# KRETOS-LAUNCHER

This is KREETOS's local build of [Elyhorizon](https://github.com/kkerneldev/Elyhorizon)
(base commit `faf034f`, itself a fork of [Horizon](https://github.com/horizon-rust/horizon)).
It is not affiliated with Mojang, Microsoft, the Horizon team or Ely.by. The
original MIT license and credits in [LICENSE](../LICENSE) and the README still apply.

GitHub: https://github.com/KREETOS/KRETOS-LAUNCHER

## Local changes

| Change | Files |
| --- | --- |
| Public sign-in IDs bundled, so both providers work in a source build | `launcher/src/account.rs`, `launcher/src/account/ely/oauth.rs` |
| Sign-in gate shows a Microsoft / Ely.by provider chooser | `launcher/src/owner_page.rs` |
| Settings has a visible **Local offline play** toggle (offline names with existing files) | `launcher/src/dev_options.rs` |
| Legacy-style **Install and play** panel: account, version and loader picker with one button | `launcher/src/install_play.rs`, `launcher/src/main.rs` |
| Recent installed versions as chips; new Minecraft release notice on startup | `launcher/src/install_play.rs`, `launcher/src/backend.rs`, `launcher/src/main.rs` |
| Memory presets: Auto / 2 GB / 4 GB / 8 GB | `launcher/src/launch_page.rs` |
| Java preflight in Install-and-play: needed major shown, one-click Mojang runtime install | `launcher/src/install_play.rs` |
| Unified **Get mods & packs** panel (Mods / Resource packs / Data packs), auto-filtered and one-click install | `launcher/src/mods_page.rs`, `launcher/src/main.rs` |
| **Data packs** tab on the full Mods page (Modrinth + world picker) | `launcher/src/mods_page.rs` |
| Registry index accepts a native mod published as `.hmod` (upstream registry bug) | `crates/horizon-mods/src/index.rs` |
| Java-only UI: Rust tab and Horizon instances hidden; no Rust versions installed | `launcher/src/glass_home.rs`, `launcher/src/frame.rs`, `launcher/src/instances_page.rs`, `launcher/src/main.rs` |
| Update badge on Java Home with one-click Update all | `launcher/src/main.rs`, `launcher/src/mods_page.rs` |
| First-run setup wizard (version → account → instance), replayable from Settings | `launcher/src/wizard.rs`, `launcher/src/main.rs` |
| Crash assistant: plain reason + fix button from `latest.log` / crash report | `launcher/src/main.rs` |
| Launcher never self-updates (protects local patches) | `launcher/src/backend.rs` |
| `version list` / `version download <id>` CLI, with SHA-1 checks and authorization gate | `launcher/src/cli.rs`, `launcher/src/minecraft_versions.rs` |

## Paths (Windows)

- Launcher: `C:\Users\Admin\Desktop\KRETOS LAUNCHER\launcher\target\release\horizon-launcher.exe`
- Game client source build: `C:\Users\Admin\Desktop\KRETOS LAUNCHER\target\fast\horizon-client.exe`
- Launcher data: `C:\Users\Admin\AppData\Roaming\horizon-launcher`
- Minecraft files: `%APPDATA%\horizon-launcher\minecraft\`
- Installed Rust builds: `%APPDATA%\horizon-launcher\versions\`

## Rebuild

```powershell
cargo build --profile fast -p horizon-client --bin horizon-client
cargo build --manifest-path launcher/Cargo.toml --release --locked --bin horizon-launcher
```

Toolchain: Rust 1.100, Python 3.14, Temurin JDK 25, Visual Studio C++ tools.

## Playing

- **Java Edition (any version):** `+` next to Instances → **Vanilla** (or **Modded**
  for Fabric/Quilt) → pick a version → **Play**. The launcher downloads the
  client, libraries, assets and Java runtime from Mojang, with SHA-1 checks.
- From the terminal: `horizon-launcher version list` and
  `horizon-launcher version download 1.20.4`. Downloads require a fresh
  Microsoft or Ely.by sign-in; offline names work with files already on disk.
- **Rust Edition:** pinned to Minecraft 26.3. The selected imported build
  (`imported-2026-11-22-624baf7`) was built from this source tree and avoids the
  "cache leases" crash seen with the newer Nov 21 Nightly clients. Do not use
  those nightlies with this launcher.

## Git

```sh
origin     https://github.com/KREETOS/KRETOS-LAUNCHER.git   # your project
upstream   https://github.com/kkerneldev/Elyhorizon.git     # original fork
```
