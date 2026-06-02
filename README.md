# SkillHub

**Universal skill registry for AI coding agents.**

> npm, but for AI skills. Fully local. Zero servers. Registry lives on GitHub, skills live on your disk. And it searches GitHub too.

---

## Install

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/Sup4ikX/skillhub-cli/main/install.ps1 | iex
```

### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/Sup4ikX/skillhub-cli/main/install.sh | bash
```

### Homebrew
```bash
brew install skillhub/tap/skillhub
```

### Scoop (Windows)
```powershell
scoop bucket add skillhub https://github.com/skillhub/scoop-bucket
scoop install skillhub
```

### winget (Windows)
```powershell
winget install SkillHub.SkillHub
```

### From source (requires Rust 1.85+)
```bash
cargo install --git https://github.com/Sup4ikX/skillhub-cli
```

### Distro packages
```bash
# Debian / Ubuntu
sudo dpkg -i skillhub_0.1.0_amd64.deb
# Fedora / RHEL
sudo rpm -i skillhub-0.1.0-1.x86_64.rpm
# Alpine
sudo apk add --allow-untrusted skillhub-0.1.0.apk
```

### Docker
```bash
docker run --rm ghcr.io/skillhub/cli --help
```

---

## Quick Start

```bash
skillhub                    # first run: configure GitHub token
skillhub search "testing"   # search the community registry
skillhub install <name>     # install a skill
skillhub list               # what's installed
skillhub check <name>       # security + quality + compatibility score
skillhub audit <name>       # check + write a JSON report to ~/.skillhub/cache/audits/
```

All commands accept `--json` for scriptable output. Every command exits with a non-zero status if it fails — easy to chain in CI.

---

## Commands

| Command | What it does |
|---|---|
| `skillhub` | Show help. If not configured, starts setup. |
| `skillhub setup` | Configure GitHub token and registry URL. |
| `skillhub agents` | Detect AI coding agents installed on this machine. |
| `skillhub search <query>` | Search the community registry. |
| `skillhub search <query> --github` | Search the registry **and** all of GitHub. |
| `skillhub install <name>` | Install a skill. Runs a security scan first. |
| `skillhub install <name> --no-scan` | Install without scanning. |
| `skillhub install <name> --project` | Install into the project's CLAUDE.md / .opencode file. |
| `skillhub uninstall <name>` | Remove an installed skill. |
| `skillhub list` | List installed skills. |
| `skillhub info <name>` | Show details about a skill. |
| `skillhub update` | Refresh the registry cache from GitHub. |
| `skillhub upgrade` | Upgrade installed skills to the latest versions. |
| `skillhub upgrade --dry-run` | Show what would upgrade without changing anything. |
| `skillhub upgrade --all` | Upgrade without confirmation. |
| `skillhub doctor` | Diagnose configuration and environment. |
| `skillhub doctor --fix` | Diagnose and fix common issues. |
| `skillhub share` | Snapshot installed skills as a manifest. |
| `skillhub restore <url-or-file>` | Install skills from a manifest. |
| `skillhub suggest` | Suggest one skill (1/day). |
| `skillhub publish <path>` | Submit a skill to the registry (creates a GitHub issue). |
| `skillhub import <source>` | Import skills from `.claude`, `.cursor`, `.windsurf`, or `awesome-claude-skills`. |
| `skillhub import <source> --dry-run` | Show what would import. |
| `skillhub stats` | Show installation statistics. |
| `skillhub badge --format=markdown` | Print a shields.io badge for your skill set. |
| `skillhub sync export [file]` | Export installed skills to a manifest file. |
| `skillhub sync import <file-or-url>` | Import from a manifest file or gist. |
| `skillhub migrate` | Migrate config/skills between format versions. |
| `skillhub migrate --rollback` | Roll back to a previous format. |
| `skillhub check <name-or-path>` | Run security, quality, and compatibility checks. Exit code: 0=pass, 1=warn, 2=fail. |
| `skillhub audit <name-or-path>` | Run `check` and persist the full report as JSON. |
| `skillhub completions <shell>` | Generate shell completions (`bash`, `zsh`, `fish`, `powershell`). |

### Global flags

| Flag | Effect |
|---|---|
| `--json` | Output structured JSON instead of colored terminal output. |
| `-y`, `--yes` | Skip confirmation prompts (install, upgrade, uninstall). |

---

## Security

`skillhub install` runs a local security scanner on every skill before writing it to disk. The scanner flags:

- **Prompt injection** — `ignore previous instructions`, `system prompt`, `override your instructions`, etc.
- **Data exfiltration** — `curl | bash`, `eval $(curl`, `nc -e`, `ssh -i`, etc.
- **Hidden Unicode** — zero-width spaces, RTL/LTR overrides, BOM, word joiners.

Findings are surfaced as a coloured warning. You can still proceed with `-y` or by answering the prompt. Skip scanning entirely with `--no-scan`.

The **audit** subcommand goes further: it grades the skill on security, quality (frontmatter, sections, examples, length, "use this when" guidance), and per-agent compatibility. Reports are saved to `~/.skillhub/cache/audits/<name>.json`.

---

## First Run

1. `skillhub` prompts for a GitHub token.
2. Create one at <https://github.com/settings/tokens> — no scopes needed; public read-only access is enough.
3. Paste it. You're done.

Without a token you get 60 requests/hour. With one: 5,000/hour.

Reconfigure later: `skillhub setup`.

---

## Supported Agents

skillhub auto-detects these AI coding agents and deploys skills to their native directories.

| Agent | Home | Skills | Project file |
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

Run `skillhub agents` to see which are detected on this machine.

---

## Architecture

```
~/.skillhub/
├── config.toml            # registry URL, GitHub token, deploy list
├── cache/
│   ├── registry.json      # cached registry
│   ├── github_search_*.json
│   └── audits/            # JSON reports from `skillhub audit`
└── skills/
    ├── anthropic/
    │   ├── code-review/
    │   │   ├── SKILL.md
    │   │   └── meta.json
    │   └── research-assistant/
    └── karpathy/
        └── debug/
```

---

## Search

```
skillhub search "testing"
  ├─ 1. community registry (JSON cache) — fast, offline
  └─ 2. --github flag → GitHub Search API — all matching repos
```

Both are cached locally (registry: forever; GitHub search: 1 hour).

---

## Tech Stack

- **CLI:** Rust, single static binary, no runtime dependencies.
- **Registry:** JSON file on GitHub (Homebrew-tap model).
- **HTTP:** `ureq` (sync, lightweight, no async overhead).
- **Config:** TOML at `~/.skillhub/config.toml`.
- **Progress:** `indicatif` (auto-hidden under CI / `NO_COLOR` / `--json`).

---

## Development

```bash
git clone https://github.com/Sup4ikX/skillhub-cli
cd skillhub-cli
cargo build
cargo test
cargo build --release
```

The release binary is `target/release/skillhub` (or `skillhub.exe` on Windows).

---

## License

MIT.

---

---

# SkillHub

**Универсальный реестр навыков для AI-агентов.**

> npm, но для AI-навыков. Полностью локально. Без своих серверов. Реестр на GitHub, навыки — на диске. Плюс поиск по GitHub.

---

## Установка

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/Sup4ikX/skillhub-cli/main/install.ps1 | iex
```

### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/Sup4ikX/skillhub-cli/main/install.sh | bash
```

### Homebrew
```bash
brew install skillhub/tap/skillhub
```

### Scoop (Windows)
```powershell
scoop bucket add skillhub https://github.com/skillhub/scoop-bucket
scoop install skillhub
```

### winget (Windows)
```powershell
winget install SkillHub.SkillHub
```

### Из исходников (нужен Rust 1.85+)
```bash
cargo install --git https://github.com/Sup4ikX/skillhub-cli
```

### Пакеты дистрибутивов
```bash
# Debian / Ubuntu
sudo dpkg -i skillhub_0.1.0_amd64.deb
# Fedora / RHEL
sudo rpm -i skillhub-0.1.0-1.x86_64.rpm
# Alpine
sudo apk add --allow-untrusted skillhub-0.1.0.apk
```

### Docker
```bash
docker run --rm ghcr.io/skillhub/cli --help
```

---

## Быстрый старт

```bash
skillhub                    # первый запуск: настройка GitHub-токена
skillhub search "testing"   # поиск в community-реестре
skillhub install <name>     # установить навык
skillhub list               # что уже стоит
skillhub check <name>       # оценка безопасности / качества / совместимости
skillhub audit <name>       # check + сохранение JSON-отчёта в ~/.skillhub/cache/audits/
```

Все команды принимают `--json` для скриптов. Каждая команда возвращает ненулевой код при ошибке — удобно для CI.

---

## Команды

| Команда | Что делает |
|---|---|
| `skillhub` | Справка. Если не настроено — запускает setup. |
| `skillhub setup` | Настройка GitHub-токена и URL реестра. |
| `skillhub agents` | Детектирование AI-агентов на машине. |
| `skillhub search <query>` | Поиск в community-реестре. |
| `skillhub search <query> --github` | Поиск в реестре **и** по всему GitHub. |
| `skillhub install <name>` | Установка навыка. Сначала прогон security-сканера. |
| `skillhub install <name> --no-scan` | Установка без сканирования. |
| `skillhub install <name> --project` | Установка в `CLAUDE.md` / `.opencode` текущего проекта. |
| `skillhub uninstall <name>` | Удаление навыка. |
| `skillhub list` | Список установленных навыков. |
| `skillhub info <name>` | Подробности о навыке. |
| `skillhub update` | Обновить кэш реестра из GitHub. |
| `skillhub upgrade` | Обновить установленные навыки. |
| `skillhub upgrade --dry-run` | Показать, что обновится, без изменений. |
| `skillhub upgrade --all` | Обновить без подтверждений. |
| `skillhub doctor` | Диагностика окружения. |
| `skillhub doctor --fix` | Диагностика + авто-исправление. |
| `skillhub share` | Снимок установленных навыков (манифест). |
| `skillhub restore <url-или-файл>` | Установка навыков из манифеста. |
| `skillhub suggest` | Одна рекомендация (1/день). |
| `skillhub publish <path>` | Отправить навык в реестр (создаёт GitHub issue). |
| `skillhub import <source>` | Импорт из `.claude`, `.cursor`, `.windsurf`, `awesome-claude-skills`. |
| `skillhub import <source> --dry-run` | Показать, что будет импортировано. |
| `skillhub stats` | Статистика установок. |
| `skillhub badge --format=markdown` | Ссылка на shields.io-бейдж для вашего набора. |
| `skillhub sync export [file]` | Экспорт установленных навыков в файл-манифест. |
| `skillhub sync import <file-или-url>` | Импорт из манифеста. |
| `skillhub migrate` | Миграция конфига/навыков между версиями формата. |
| `skillhub migrate --rollback` | Откат на предыдущий формат. |
| `skillhub check <name-или-path>` | Проверки безопасности, качества, совместимости. Код возврата: 0=ok, 1=warn, 2=fail. |
| `skillhub audit <name-или-path>` | `check` + сохранение полного отчёта в JSON. |
| `skillhub completions <shell>` | Генерация автодополнений (`bash`, `zsh`, `fish`, `powershell`). |

### Глобальные флаги

| Флаг | Эффект |
|---|---|
| `--json` | JSON вместо цветного вывода в терминал. |
| `-y`, `--yes` | Пропустить подтверждения (install, upgrade, uninstall). |

---

## Безопасность

`skillhub install` локально сканирует навык перед записью на диск. Сканер ловит:

- **Prompt injection** — `ignore previous instructions`, `system prompt`, `override your instructions` и т.п.
- **Data exfiltration** — `curl | bash`, `eval $(curl`, `nc -e`, `ssh -i` и т.п.
- **Скрытый Unicode** — zero-width, RTL/LTR override, BOM, word joiner.

Найденные проблемы показываются как цветные предупреждения. Продолжить можно через `-y` или ответив «да» на запрос. Полностью отключить — `--no-scan`.

Подкоманда `audit` идёт дальше: выставляет оценки безопасности, качества (frontmatter, секции, примеры, длина, секция «use this when») и совместимости с каждым агентом. Отчёты сохраняются в `~/.skillhub/cache/audits/<name>.json`.

---

## Первый запуск

1. `skillhub` спросит GitHub-токен.
2. Создайте его на <https://github.com/settings/tokens> — никакие scopes не нужны, достаточно публичного read-only.
3. Вставьте. Готово.

Без токена — 60 запросов/час. С токеном — 5,000/час.

Перенастроить: `skillhub setup`.

---

## Поддерживаемые агенты

skillhub автоматически находит эти AI-агенты и раскладывает навыки в их нативные директории.

| Агент | Home | Skills | Project-файл |
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

Запустите `skillhub agents`, чтобы увидеть, что обнаружено у вас.

---

## Архитектура

```
~/.skillhub/
├── config.toml            # URL реестра, GitHub-токен, список агентов
├── cache/
│   ├── registry.json      # кэш реестра
│   ├── github_search_*.json
│   └── audits/            # JSON-отчёты `skillhub audit`
└── skills/
    ├── anthropic/
    │   ├── code-review/
    │   │   ├── SKILL.md
    │   │   └── meta.json
    │   └── research-assistant/
    └── karpathy/
        └── debug/
```

---

## Поиск

```
skillhub search "testing"
  ├─ 1. community-реестр (JSON-кэш) — быстро, офлайн
  └─ 2. --github → GitHub Search API — все подходящие репозитории
```

Оба результата кэшируются локально (реестр: бессрочно; GitHub search: 1 час).

---

## Стек

- **CLI:** Rust, один статический бинарник, без рантайм-зависимостей.
- **Реестр:** JSON на GitHub (модель Homebrew-tap).
- **HTTP:** `ureq` (sync, лёгкий, без async-оверхеда).
- **Конфиг:** TOML в `~/.skillhub/config.toml`.
- **Прогресс:** `indicatif` (авто-скрывается в CI / при `NO_COLOR` / `--json`).

---

## Разработка

```bash
git clone https://github.com/Sup4ikX/skillhub-cli
cd skillhub-cli
cargo build
cargo test
cargo build --release
```

Релизный бинарник — `target/release/skillhub` (или `skillhub.exe` на Windows).

---

## Лицензия

MIT.
