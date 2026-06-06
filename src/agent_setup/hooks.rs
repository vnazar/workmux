//! Shared JSON hook utilities for agent status tracking setup.
//!
//! Agents like Claude Code, Codex, and Gemini all use the same pattern:
//! hooks are stored as JSON objects under a "hooks" key, with workmux
//! commands in the "command" fields. This module provides shared helpers.

use serde_json::Value;

/// Check if a parsed JSON value contains any workmux status hooks.
pub fn has_workmux_hooks(settings: &Value) -> bool {
    let Some(hooks) = settings.get("hooks").and_then(|v| v.as_object()) else {
        return false;
    };
    for (_event, groups) in hooks {
        let Some(groups_arr) = groups.as_array() else {
            continue;
        };
        for group in groups_arr {
            let Some(hook_list) = group.get("hooks").and_then(|v| v.as_array()) else {
                continue;
            };
            for hook in hook_list {
                if let Some(cmd) = hook.get("command").and_then(|v| v.as_str())
                    && cmd.contains("workmux set-window-status")
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Remove workmux hook commands from `settings` in place.
///
/// Removes individual hook entries whose command contains
/// `workmux set-window-status`, then cleans up empty groups
/// and events. Returns true if anything was removed.
pub fn remove_workmux_hooks(settings: &mut Value) -> bool {
    let Some(hooks) = settings.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return false;
    };

    let mut modified = false;
    let mut events_to_remove: Vec<String> = Vec::new();

    for (event, groups) in hooks.iter_mut() {
        let Some(groups_arr) = groups.as_array_mut() else {
            continue;
        };

        // For each group, remove only workmux hooks from its inner hooks array
        for group in groups_arr.iter_mut() {
            if let Some(hooks_list) = group.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                let len_before = hooks_list.len();
                hooks_list.retain(|e| {
                    !e.get("command")
                        .and_then(|c| c.as_str())
                        .is_some_and(|c| c.contains("workmux set-window-status"))
                });
                if hooks_list.len() < len_before {
                    modified = true;
                }
            }
        }

        // Remove groups that now have empty hooks arrays
        groups_arr.retain(|group| {
            group
                .get("hooks")
                .and_then(|h| h.as_array())
                .is_some_and(|h| !h.is_empty())
        });

        if groups_arr.is_empty() {
            events_to_remove.push(event.clone());
        }
    }

    for event in &events_to_remove {
        hooks.remove(event);
    }

    modified
}

/// Remove workmux-status plugin entries from enabledPlugins.
pub fn remove_workmux_plugins(settings: &mut Value) -> bool {
    let Some(plugins) = settings
        .get_mut("enabledPlugins")
        .and_then(|v| v.as_object_mut())
    else {
        return false;
    };
    let keys: Vec<String> = plugins
        .keys()
        .filter(|k| k.starts_with("workmux-status@"))
        .cloned()
        .collect();
    let modified = !keys.is_empty();
    for key in &keys {
        plugins.remove(key);
    }
    modified
}

/// Remove empty wrapper objects from the JSON tree.
/// E.g., if "hooks" is now an empty object, remove the "hooks" key.
pub fn remove_empty_hooks_wrapper(settings: &mut Value) -> bool {
    let root = settings.as_object_mut().map(|o| {
        let mut modified = false;
        if let Some(hooks) = o.get("hooks")
            && hooks.as_object().is_some_and(|m| m.is_empty())
        {
            o.remove("hooks");
            modified = true;
        }
        if let Some(plugins) = o.get("enabledPlugins")
            && plugins.as_object().is_some_and(|m| m.is_empty())
        {
            o.remove("enabledPlugins");
            modified = true;
        }
        modified
    });
    root.unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_has_workmux_hooks_empty() {
        let settings = json!({});
        assert!(!has_workmux_hooks(&settings));
    }

    #[test]
    fn test_has_workmux_hooks_present() {
        let settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [{
                        "type": "command",
                        "command": "workmux set-window-status done"
                    }]
                }]
            }
        });
        assert!(has_workmux_hooks(&settings));
    }

    #[test]
    fn test_has_workmux_hooks_other_hooks_only() {
        let settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [{
                        "type": "command",
                        "command": "afplay /System/Library/Sounds/Glass.aiff"
                    }]
                }]
            }
        });
        assert!(!has_workmux_hooks(&settings));
    }

    #[test]
    fn test_remove_workmux_hooks_mixed() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [{ "type": "command", "command": "workmux set-window-status done" }]
                }, {
                    "hooks": [{ "type": "command", "command": "afplay /System/Library/Sounds/Glass.aiff" }]
                }]
            },
            "enabledPlugins": {
                "workmux-status@workmux": true,
                "other-plugin@1.0": true
            }
        });

        assert!(remove_workmux_hooks(&mut settings));
        assert!(remove_workmux_plugins(&mut settings));
        remove_empty_hooks_wrapper(&mut settings);

        // Workmux hook group removed, non-workmux group preserved
        let stop = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert!(
            stop[0]["hooks"][0]["command"]
                .as_str()
                .unwrap()
                .contains("Glass")
        );

        // Workmux plugin removed, other plugin preserved
        assert!(
            settings["enabledPlugins"]
                .as_object()
                .unwrap()
                .contains_key("other-plugin@1.0")
        );
        assert!(
            !settings["enabledPlugins"]
                .as_object()
                .unwrap()
                .contains_key("workmux-status@workmux")
        );
    }

    #[test]
    fn test_remove_workmux_hooks_only_workmux() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [{ "type": "command", "command": "workmux set-window-status done" }]
                }]
            }
        });

        assert!(remove_workmux_hooks(&mut settings));
        remove_empty_hooks_wrapper(&mut settings);
        // Empty hooks object should be removed
        assert!(settings.get("hooks").is_none());
    }

    #[test]
    fn test_remove_workmux_hooks_mixed_in_same_group() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [
                        { "type": "command", "command": "workmux set-window-status done" },
                        { "type": "command", "command": "afplay /System/Library/Sounds/Glass.aiff" },
                        { "type": "command", "command": "echo user-hook" }
                    ]
                }]
            }
        });

        assert!(remove_workmux_hooks(&mut settings));

        // The group should still exist with non-workmux hooks preserved
        let stop = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        let hooks = stop[0]["hooks"].as_array().unwrap();
        assert_eq!(hooks.len(), 2);
        assert!(hooks[0]["command"].as_str().unwrap().contains("Glass"));
        assert!(hooks[1]["command"].as_str().unwrap().contains("echo"));
    }

    #[test]
    fn test_remove_workmux_hooks_idempotent() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{
                    "hooks": [{ "type": "command", "command": "workmux set-window-status done" }]
                }]
            }
        });
        assert!(remove_workmux_hooks(&mut settings));
        // Second call should return false (nothing to remove)
        assert!(!remove_workmux_hooks(&mut settings));
    }

    #[test]
    fn test_remove_workmux_hooks_empty_settings() {
        let mut settings = json!({});
        assert!(!remove_workmux_hooks(&mut settings));
    }

    #[test]
    fn test_remove_workmux_plugins_empty() {
        let mut settings = json!({});
        assert!(!remove_workmux_plugins(&mut settings));
    }

    #[test]
    fn test_remove_workmux_plugins_only_workmux() {
        let mut settings = json!({
            "enabledPlugins": {
                "workmux-status@workmux": true
            }
        });
        assert!(remove_workmux_plugins(&mut settings));
        assert!(settings["enabledPlugins"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_remove_workmux_plugins_idempotent() {
        let mut settings = json!({
            "enabledPlugins": {
                "workmux-status@workmux": true
            }
        });
        assert!(remove_workmux_plugins(&mut settings));
        assert!(!remove_workmux_plugins(&mut settings));
    }

    #[test]
    fn test_remove_empty_hooks_wrapper_none() {
        let mut settings = json!({
            "hooks": {
                "Stop": [{"hooks": [{"command": "echo hi"}]}]
            }
        });
        assert!(!remove_empty_hooks_wrapper(&mut settings));
    }

    #[test]
    fn test_remove_empty_hooks_wrapper_empty_hooks() {
        let mut settings = json!({ "hooks": {} });
        assert!(remove_empty_hooks_wrapper(&mut settings));
        assert!(settings.get("hooks").is_none());
    }

    #[test]
    fn test_remove_empty_hooks_wrapper_empty_plugins() {
        let mut settings = json!({ "enabledPlugins": {} });
        assert!(remove_empty_hooks_wrapper(&mut settings));
        assert!(settings.get("enabledPlugins").is_none());
    }
}
