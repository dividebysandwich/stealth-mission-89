# The player's files

Everything the game writes for the player is in one folder, with the same layout on every
system (`crates/stealth-host/src/store.rs`). Nothing is written next to the game, which stays
read-only (Program Files, the AppImage, the macOS bundle, Steam's install folder).

| System | Folder |
|---|---|
| Windows | `Documents\My Games\Stealth Mission 89` (the Documents folder wherever it was moved, OneDrive included) |
| macOS | `~/Library/Application Support/stealth-mission-89` |
| Linux, Steam Deck, Steam Frame | `$XDG_DATA_HOME/stealth-mission-89`, by default `~/.local/share/stealth-mission-89` |
| Browser | The browser's local storage for the site |

Windows without a Documents folder falls back to `%APPDATA%\stealth-mission-89`. macOS keeps
`Application Support` because macOS asks the player's permission before a game reads Documents;
Linux has no "My Games" convention and Documents may not exist (SteamOS).

## Layout

| Path | What | Synced by Steam Cloud |
|---|---|---|
| `saves/*.json` | Single missions saved in flight (SAVE MISSION) | yes |
| `pilots/*.json` | Career pilots and their tours | yes |
| `campaigns/*.json` | Dynamic campaigns (and a dedicated server's `dedicated.json`) | yes |
| `config/prefs.json` | Options and the menu's choices | no: this computer's |
| `config/input.json` | Key and joystick bindings | no: this computer's devices |
| `config/route.json`, `config/loadout.json` | Handed to the game when FLY IN VR restarts it | no |
| `tacview/*.acmi` | Tacview recordings | no: exports, large |
| `stealth-mission-89.log`, `stealth-mission-89-vr.log` | The log of a game started from Explorer, Steam or a launcher | no |
| `assets/` (Linux AppImage only) | The game's shaders and data, copied there on every start | no |

File names are lower case ASCII, digits, `-` and `_` only, so they are the same file on every
system. Saves, pilots, campaigns and settings are written next to themselves as `<name>.tmp` and
then renamed over the old file: a crash or a full disk keeps the previous contents, and a Steam
Cloud pattern of `*.json` never syncs a half-written file.

## Steam Cloud

Steam Auto-Cloud needs no code in the game (Steamworks partner site, App Admin > Cloud). With
the layout above:

* **Byte quota per user**: 100 MB. **Number of files allowed per user**: 1000. A single mission
  save is about 100 kB plus about 50 kB per minute flown; pilots and campaigns are smaller.
* **Root paths**, one row per synced folder (subdirectory relative to the root, pattern `*.json`,
  not recursive, OS Windows):

| Root | Subdirectory | Pattern |
|---|---|---|
| `WinMyDocuments` | `My Games/Stealth Mission 89/saves` | `*.json` |
| `WinMyDocuments` | `My Games/Stealth Mission 89/pilots` | `*.json` |
| `WinMyDocuments` | `My Games/Stealth Mission 89/campaigns` | `*.json` |

* **Root overrides**, so a save made on one system loads on the others:

| Original root | OS | New root | Add/Replace path | Replace path |
|---|---|---|---|---|
| `WinMyDocuments` | macOS | `MacAppSupport` | `My Games/Stealth Mission 89` | `stealth-mission-89` |
| `WinMyDocuments` | Linux + SteamOS | `LinuxXdgDataHome` | `My Games/Stealth Mission 89` | `stealth-mission-89` |

The settings (`config/`) are left out on purpose, as Steam advises for what depends on the
computer: graphics options and VR, and bindings that name this computer's joysticks. A Steam Deck
running the Windows build through Proton keeps its files in Proton's own Documents folder, which
the `WinMyDocuments` rows cover.

All the Steam accounts of one system account share the folder: the game does not use the
Steamworks API, so it doesn't know which Steam account plays.
