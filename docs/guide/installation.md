---
description: Install workmux via Homebrew, pre-built binaries, Cargo, mise, or Nix
---

# Installation

## Bash YOLO

```bash
curl -fsSL https://raw.githubusercontent.com/raine/workmux/main/scripts/install.sh | bash
```

## Homebrew (macOS/Linux)

```bash
brew install raine/workmux/workmux
```

## Other methods

### Cargo

Requires Rust. Install via [rustup](https://rustup.rs/) if you don't have it.

```bash
cargo install workmux
```

### mise

```bash
mise use -g cargo:raine/workmux
```

### Nix

Requires [Nix with flakes enabled](https://nixos.wiki/wiki/Flakes).

```bash
nix profile install github:raine/workmux
```

Or try without installing:

```bash
nix run github:raine/workmux -- --help
```

See [Nix guide](/guide/nix) for flake integration and home-manager setup.

---

For manual installation, see [pre-built binaries](https://github.com/raine/workmux/releases/latest).

## Shell alias (recommended)

For faster typing, alias `workmux` to `wm`:

```bash
alias wm='workmux'
```

Add this to your `.bashrc`, `.zshrc`, or equivalent shell configuration file.

## Shell completions

To enable tab completions for commands and branch names, add the following to your shell's configuration file.

::: code-group

```bash [Bash]
# Add to ~/.bashrc
eval "$(workmux completions bash)"
```

```bash [Zsh]
# Add to ~/.zshrc
eval "$(workmux completions zsh)"
```

```bash [Fish]
# Add to ~/.config/fish/config.fish
workmux completions fish | source
```

:::

## Uninstalling

### Automatic (recommended)

Run the uninstall script:

```bash
curl -fsSL https://raw.githubusercontent.com/raine/workmux/main/scripts/uninstall.sh | bash
```

If installed via [Homebrew](#homebrew-macoslinux), also run:

```bash
brew uninstall workmux
brew untap raine/workmux
```

If installed via [Cargo](#cargo), also run:

```bash
cargo uninstall workmux
```

### Manual

1. Run the uninstall script or call `workmux uninstall` if the binary is still available
2. Remove the binary from `/usr/local/bin`, `~/.local/bin`, or `~/.cargo/bin`
3. Remove cache and state: `rm -rf ~/.cache/workmux ~/.local/state/workmux`
4. Optionally remove config: `rm -rf ~/.config/workmux`
5. Remove shell completions and `alias wm=workmux` from your shell config
6. Clean up worktrees with `git worktree list` / `git worktree remove`
