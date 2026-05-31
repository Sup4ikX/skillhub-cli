# SkillHub for VS Code

Universal skill registry for AI coding agents — install, search, and manage skills directly from VS Code.

## Features

- **TreeView** — browse installed skills and search the registry
- **Install / Uninstall** — one-click skill management
- **Search** — find skills by keyword from the community registry
- **Status Bar** — shows installed skill count at a glance
- **Info Panel** — detailed skill information in a webview

![SkillHub TreeView](https://img.shields.io/badge/screenshot-coming_soon-blue)

## Requirements

- [SkillHub CLI](https://github.com/skillhub/cli) installed and configured (`skillhub` in PATH)
- VS Code 1.85+

## Usage

1. Open the **SkillHub Skills** view in the Explorer sidebar
2. Click refresh (↻) to load installed skills
3. Click search (🔍) to find skills in the registry
4. Right-click a skill to install or uninstall
5. Click the status bar to jump to installed skills

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `skillhub.enabled` | `true` | Enable SkillHub views |
| `skillhub.binaryPath` | `skillhub` | Path to the CLI binary |

## Commands

| Command | Description |
|---------|-------------|
| `SkillHub: Refresh Skills` | Reload installed skills |
| `SkillHub: Search Registry` | Search for skills |
| `SkillHub: Install Skill` | Install selected skill |
| `SkillHub: Uninstall Skill` | Uninstall selected skill |
| `SkillHub: Show Skill Info` | Show detailed info |
| `SkillHub: List Installed Skills` | Show installed list |

## Known Issues

- CLI must be pre-installed separately
- Search queries the community registry (no GitHub fallback from extension)

## Release Notes

### 0.1.0

Initial release: TreeView, install/uninstall, search, status bar, info panel.
