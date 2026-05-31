*Scroll down for Russian version / Прокрутите вниз для русской версии*

---

# SkillHub

**Universal Skill Registry for AI Agents**

> npm, but for AI skills. Fully local. Zero servers. Registry lives on GitHub, skills live on your disk. And searches GitHub too.

## Install

### Windows (PowerShell)
```powershell
powershell -c "irm https://skillhub.sh/install.ps1 | iex"
# or locally:
.\install.ps1
```

### macOS / Linux
```bash
curl -fsSL https://skillhub.sh/install.sh | bash
# or locally:
chmod +x install.sh && ./install.sh
```

### Homebrew
```bash
brew tap skillhub/tap
brew install skillhub
```

### Scoop (Windows)
```powershell
scoop bucket add skillhub https://github.com/skillhub/scoop
scoop install skillhub
```

### From source (requires Rust)
```bash
cargo install --git https://github.com/skillhub/skillhub
# or locally:
cargo build --release && ./target/release/skillhub
```

## Quick Start

```bash
# First run — auto-configures GitHub token
skillhub

# Search community registry
skillhub search "code review"

# Also search GitHub repos (opts into skillhub-skill topic)
skillhub search "testing" --github

# Install a skill
skillhub install @anthropic/code-review

# List installed
skillhub list

# Show skill details
skillhub info @anthropic/code-review

# Refresh registry
skillhub update

# Upgrade installed skills to latest versions
skillhub upgrade

# See what upgrades without doing them
skillhub upgrade --dry-run

# Uninstall
skillhub uninstall @anthropic/code-review

# Diagnose your setup
skillhub doctor

# Fix common issues automatically
skillhub doctor --fix

# Snapshot your installed skills
skillhub share

# Restore from snapshot
skillhub restore <url-or-file>

# One suggestion per day
skillhub suggest

# Generate shell completions
skillhub completions bash
skillhub completions zsh
skillhub completions fish
skillhub completions powershell

# JSON output for scripting
skillhub list --json
skillhub search "code" --json
skillhub info @anthropic/code-review --json
```

## First Run

On first launch, `skillhub` will guide you through setting up a GitHub token:

1. Go to https://github.com/settings/tokens
2. Create a token (no scopes needed — public repos only)
3. Paste it when prompted

Without a token: 60 requests/hour. With a token: 5,000/hour.

To reconfigure later: `skillhub setup`

## Commands

| Command | Description |
|---|---|
| `skillhub` | Show help (first run starts setup) |
| `skillhub agents` | Detect AI agents on this machine |
| `skillhub setup` | Configure GitHub token and registry URL |
| `skillhub search <query>` | Search skills in community registry |
| `skillhub search <query> --github` | Search registry + GitHub (topic: skillhub-skill) |
| `skillhub install <name>` | Install a skill (runs security scan) |
| `skillhub install <name> --no-scan` | Skip security scan |
| `skillhub uninstall <name>` | Remove installed skill |
| `skillhub list` | List installed skills |
| `skillhub update` | Refresh registry cache from GitHub |
| `skillhub upgrade` | Upgrade all installed skills |
| `skillhub upgrade --dry-run` | Show what would upgrade |
| `skillhub upgrade --all` | Upgrade without confirmations |
| `skillhub info <name>` | Show skill details |
| `skillhub doctor` | Diagnose configuration and environment |
| `skillhub doctor --fix` | Fix common issues |
| `skillhub share` | Snapshot installed skills as a manifest |
| `skillhub restore <url>` | Install skills from a manifest |
| `skillhub suggest` | One skill recommendation (1/day) |
| `skillhub publish <path>` | Submit a skill to the registry |
| `skillhub completions <shell>` | Generate shell completions |

### Global flags

| Flag | Effect |
|---|---|
| `--json` | All commands output JSON |

## Supported Agents

skillhub auto-detects these AI coding agents and deploys skills to their directories:

| Agent | Home Dir | Skills Dir | Project File |
|---|---|---|---|
| **Claude Code** | `~/.claude/` | `skills/` | `CLAUDE.md` |
| **Codex** (OpenAI) | `~/.codex/` | `memories/` | `AGENTS.md` |
| **Cursor** | `~/.cursor/` | `skills/` | `.cursor/rules` |
| **Copilot** (GitHub) | `~/.config/github-copilot/` | `instructions/` | `.github/copilot-instructions.md` |
| **Windsurf** (Codeium) | `~/.windsurf/` | `skills/` | `.windsurf/rules` |
| **Gemini CLI** (Google) | `~/.gemini/` | `commands/` | `GEMINI.md` |
| **Aider** | `~/.aider/` | `(root)` | `AGENTS.md` |
| **Cline** | `~/.cline/` | `data/skills/` | `.clinerules` |
| **OpenCode** | `~/.config/opencode/` | `agents/` | `.opencode` |
| **OpenClaw** | `~/.openclaw/` | `skills/` | `.openclaw` |
| **Roo Code** | `~/.roo/` | `rules/` | `.roo/rules` |
| **Kilo Code** | `~/.kilo/` | `rules/` | `.kilo/rules` |
| **Crush** (Charm) | `~/.crush/` | `skills/` | `CRUSH.md` |
| **Factory** | `~/.factory/` | `skills/` | `.factory` |
| **Continue.dev** | `~/.continue/` | `prompts/` | `.continue` |
| **Qwen Code** (Alibaba) | `~/.qwen/` | `commands/` | `QWEN.md` |
| **Pi** | `~/.pi/` | `skills/` | `.pi` |
| **Augment Code** | `~/.augment/` | `rules/` | `.augment/rules` |
| **Trae** (ByteDance) | `~/.trae/` | `skills/` | `.trae` |
| **Codeium** | `~/.codeium/` | `skills/` | `.codeium` |
| **Zed** | `~/.config/zed/` | `skills/` | `.zed` |
| **Warp** | `~/.warp/` | `skills/` | `AGENTS.md` |
| **Tabby** | `~/.tabby/` | `skills/` | `.tabby` |
| **Antigravity** | `~/.antigravity/` | `skills/` | `ANTIGRAVITY.md` |
| **Kimi Code** | `~/.kimi/` | `commands/` | `KIMI.md` |
| **Kiro IDE** | `~/.kiro/` | `agents/` | `.kiro` |
| **Goose** (Block) | `~/.config/goose/` | `skills/` | `GOOSE.md` |
| **Amazon Q** | `~/.config/amazon-q/` | `instructions/` | `.amazon-q` |
| **Cody** (Sourcegraph) | `~/.cody/` | `skills/` | `.cody` |
| **Devin** (Cognition) | `~/.devin/` | `skills/` | `.devin` |
| **Poolside** | `~/.poolside/` | `skills/` | `.poolside` |
| **Cosine** | `~/.cosine/` | `skills/` | `.cosine` |

Run `skillhub agents` to see which are detected on your machine.

## Architecture

```
~/.skillhub/
├── config.toml           # User config (registry URL, GitHub token)
├── cache/
│   ├── registry.json     # Cached registry (from GitHub)
│   └── github_search_*.json  # Cached GitHub search results
└── skills/
    ├── anthropic/
    │   ├── code-review/
    │   │   ├── SKILL.md
    │   │   └── meta.json
    │   └── research-assistant/
    │       ├── SKILL.md
    │       └── meta.json
    └── karpathy/
        └── debug/
            ├── SKILL.md
            └── meta.json
```

## Search Architecture (Hybrid)

```
skillhub search "testing"
├── 1. Community registry (JSON cache) ─── fast, offline
└── 2. skillhub search "testing" --github
    └── GitHub Search API ─── repos with topic skillhub-skill
```

- **Community registry**: curated JSON on GitHub (fast, always works)
- **GitHub search**: finds repos with `skillhub-skill` topic (opt-in, discoverable)
- **Caching**: both results cached locally (registry: forever; GitHub: 1h)

## Security

`skillhub install` automatically scans skills for:
- **Prompt injection** (`ignore previous instructions`, etc.)
- **Data exfiltration** (`curl | bash`, `eval $(curl`, etc.)
- **Hidden unicode** (zero-width spaces, RTL override, etc.)

Skip with `--no-scan`.

## How It Works

```
┌──────────────┐      fetch       ┌──────────────────┐
│  GitHub API  │ ───────────────> │  ~/.skillhub/    │
│  (registry)  │                  │  cache/          │
└──────────────┘                  │  skills/         │
         │                        │  config.toml     │
         └── GitHub Search ─────> │ (cached results) │
                                  └──────────────────┘
      No servers.
      No cloud.
      Just GitHub + your disk.
```

- **Registry** = JSON file hosted in a GitHub repo (like Homebrew taps)
- **GitHub Search** = repos with `skillhub-skill` topic
- **Skills** = markdown files stored in `~/.skillhub/skills/`
- **Cache** = registry + GitHub search cached locally
- **Everything else** = Rust CLI, single binary, no runtime deps

## Tech Stack

- **CLI:** Rust (fast, single binary, no runtime)
- **Registry:** JSON on GitHub (like Homebrew taps)
- **HTTP:** ureq (lightweight, sync, no async overhead)
- **Config:** TOML (~/.skillhub/config.toml)
- **Progress:** indicatif (progress bars, auto-hidden in CI)
- **Paths:** directories crate (cross-platform)

## Development

```bash
cargo build
cargo test
cargo build --release
```

## License

MIT

---

# SkillHub

**Универсальный реестр навыков для AI-агентов**

## Установка

### Windows (PowerShell)
```powershell
powershell -c "irm https://skillhub.sh/install.ps1 | iex"
```

### macOS / Linux
```bash
curl -fsSL https://skillhub.sh/install.sh | bash
```

### Homebrew
```bash
brew tap skillhub/tap
brew install skillhub
```

### Scoop (Windows)
```powershell
scoop bucket add skillhub https://github.com/skillhub/scoop
scoop install skillhub
```

### Из исходников (требуется Rust)
```bash
cargo install --git https://github.com/skillhub/skillhub
cargo build --release && ./target/release/skillhub
```

## Быстрый старт

```bash
# Первый запуск — настройка GitHub токена
skillhub

# Поиск в community реестре
skillhub search "code review"

# Поиск по GitHub (репозитории с топиком skillhub-skill)
skillhub search "testing" --github

# Установка навыка (автоматический security scan)
skillhub install @anthropic/code-review

# Список установленных
skillhub list

# Информация о навыке
skillhub info @anthropic/code-review

# Обновление реестра
skillhub update

# Обновление всех навыков
skillhub upgrade

# Диагностика
skillhub doctor
skillhub doctor --fix

# Слепок установленных навыков
skillhub share

# Восстановление из слепка
skillhub restore <url-или-файл>

# Одна рекомендация в день
skillhub suggest

# Генерация автодополнений для оболочки
skillhub completions bash

# JSON-вывод для скриптов
skillhub list --json
```

## Команды

| Команда | Описание |
|---|---|
| `skillhub` | Справка (первый запуск → настройка) |
| `skillhub agents` | Детектирование AI-агентов |
| `skillhub setup` | Настройка токена и URL реестра |
| `skillhub search <query>` | Поиск в community реестре |
| `skillhub search <query> --github` | Поиск + GitHub |
| `skillhub install <name>` | Установка (со сканированием) |
| `skillhub install <name> --no-scan` | Установка без сканирования |
| `skillhub list` | Список установленных |
| `skillhub update` | Обновление кэша реестра |
| `skillhub upgrade` | Обновление навыков |
| `skillhub info <name>` | Детали навыка |
| `skillhub doctor` | Диагностика |
| `skillhub doctor --fix` | Диагностика + исправление |
| `skillhub share` | Экспорт манифеста навыков |
| `skillhub restore <url>` | Импорт из манифеста |
| `skillhub suggest` | Рекомендация (1/день) |
| `skillhub publish` | Публикация навыка |
| `skillhub completions <shell>` | Автодополнения |

### Глобальные флаги

| Флаг | Эффект |
|---|---|
| `--json` | Все команды выводят JSON |

## Поддерживаемые агенты

32 AI-агента: Claude Code, Codex, Cursor, Copilot, Windsurf, Gemini CLI, Aider, Cline, OpenCode, OpenClaw, Roo Code, Kilo Code, Crush, Factory, Continue.dev, Qwen Code, Pi, Augment Code, Trae, Codeium, Zed, Warp, Tabby, Antigravity, Kimi Code, Kiro IDE, Goose, Amazon Q, Cody, Devin, Poolside, Cosine.

Запусти `skillhub agents` чтобы увидеть, какие обнаружены на твоей машине.

## Архитектура поиска (гибрид)

- **Community реестр** — курируемый JSON на GitHub (быстро, всегда работает)
- **GitHub Search** — репозитории с топиком `skillhub-skill` (opt-in, обнаруживаемые)
- **Кэширование** — оба результата кэшируются локально

## Безопасность

`skillhub install` автоматически сканирует навыки на:
- **Prompt injection** (`ignore previous instructions` и т.д.)
- **Data exfiltration** (`curl | bash`, `eval $(curl` и т.д.)
- **Скрытый unicode** (zero-width символы, RTL override и т.д.)

Отключить: `--no-scan`.

## Разработка

```bash
cargo build
cargo test
cargo build --release
```

## Лицензия

MIT
