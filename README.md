# viscosity-cli

Small macOS CLI for [Viscosity](https://www.viscosityvpn.com/) (via `osascript` / AppleScript).

## Install

1. Install [Rust](https://rustup.rs/) (includes `cargo`).
2. Clone or copy this folder, then from inside it:

   ```bash
   cargo install --path .
   ```

3. Put Cargo’s binaries on your `PATH` if needed (the installer may print a hint). Typical fix for zsh:

   ```bash
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

4. The binary is `viscosity-cli`. You need Viscosity installed; macOS may ask to allow **Automation** for your terminal the first time.

## Use

```bash
viscosity-cli list          # table of connections (#, name, state)
viscosity-cli connect 3     # connect by 1-based index from list
viscosity-cli disconnect 3
viscosity-cli connect "My VPN Name"
viscosity-cli disconnect "My VPN Name"
```

If the target is **only digits**, it is treated as an index. Anything else is treated as the connection name. For options, run `viscosity-cli --help`.
