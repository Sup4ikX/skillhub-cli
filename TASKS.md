# TASKS.md — SkillHub Sprint 1 & 2

## English

| # | Task | Description | Tests |
|---|------|-------------|-------|
| **1** | `--json` flag for all commands | Every command (`search`, `list`, `info`, `install`, `uninstall`, `update`, `doctor`, `agents`, `upgrade`) accepts `--json` and outputs structured JSON to stdout instead of colored terminal output. Flag added to `Cli` as global. | `search --json` parses as valid JSON with `{"skills":[...], "count":N}`. `list --json` with empty list → `{"skills":[]}`. |
| **2** | `skillhub upgrade` | Compares installed skill versions with registry. If registry version is newer — downloads and replaces. `--dry-run` shows what would update without disk changes. `--all` skips confirmation. | Installed v1.0.0, registry v2.0.0 → upgrade updates files. Latest installed → upgrade does nothing. |
| **3** | `skillhub doctor` | Diagnostics: GitHub token validity, registry cache freshness, agent directories exist, write permissions, config syntax. `--fix` repairs: creates folders, re-requests token, recreates cache. | Missing config → "not configured". Missing `skills_dir` → created. Cache older than 24h → warning. |
| **4** | Shell completions | `skillhub completions bash/zsh/fish/powershell` generates completion script to stdout. | `completions bash` starts with `#compdef skillhub`. Scripts parse without errors. |
| **5** | `skillhub share` / `restore` | `share` serializes installed skills + versions, generates URL and one-liner command. `restore <url>` downloads JSON list, installs missing. | share+restore roundtrip gives identical set. |
| **6** | Progress bar (`indicatif`) | `install`, `upgrade`, `update` show progress bar + ETA. Auto-disables if stdout is not TTY, if `--json`, `--no-progress`, or `CI`/`NO_COLOR` env. | `--no-progress` disables. `--json` auto-disables. CI env works. |
| **7** | `skillhub suggest` | One recommendation per day on explicit command. Analyzes installed agents, popular skills, tag overlap. Stores `last_suggest_date` in config. | No agents → most popular skill. Called twice daily → "already suggested today". |
| **8** | `skillhub publish` | Publish skill to registry via GitHub Issue/PR. Validates SKILL.md (title, description, tags), name uniqueness. `--force` skips validation. | Empty name → error. Duplicate name → "already exists". Valid skill → creates PR. |
| **9** | SkillTest lite (security scan) | Basic security scan on `install`: regex check for prompt injection, data exfiltration, hidden unicode. `--no-scan` disables. | Malicious skill → warning. Clean skill → "scan passed". |
| **10** | Homebrew tap + Scoop + winget | Homebrew formula, Scoop manifest, winget manifest. All point to GitHub Release binaries. | (e2e, manual) |
| **11** | VS Code extension | TreeView with skill registry, install/uninstall/search buttons, status bar. TypeScript, minimal bundle, calls `skillhub` CLI. | Extension activates on folder open. TreeView shows `skillhub list --json`. |
| **12** | `skillhub sync` | Export/restore skill sets between machines. `export` generates JSON/TOML, `import <file>` restores. Supports GitHub Gist as remote. | Roundtrip export→import gives identical set. |
| **13** | `skillhub stats` | Statistics: installed skills count, authors, agents, popular skills. | Empty registry → "no data". 3 skills from 2 authors → correct stats. |
| **14** | Skill badges | Generate shields.io-compatible URL. `skillhub badge --format=markdown` outputs markdown. | `badge --format=markdown` outputs `![skillhub]...`. 0 skills → `skills-0-lightgrey`. |
| **15** | `skillhub import` | Import skills from external formats: `.claude/skills/`, `.cursor/rules/`, `awesome-claude-skills`. Auto-convert to SKILL.md + meta.json. | Empty dir → nothing. `.cursor/rules/` with .mdc → correct SKILL.md. |
| **16** | Auto-detect project (`--project`) | `skillhub install <name> --project` — installs to current project instead of `~/.skillhub/skills/`. Detects active agent by project files. | No agents → error. With `.opencode` → writes to `.opencode/agents/`. |
| **17** | Expand supported agents | Research and add AI coding agents from 2026. Add to `agent.rs`, update README table, tests. | Each agent has label, detect_home, skills_dir, project_file. Names unique. |
| **18** | Docker image | Dockerfile based on scratch/alpine + skillhub binary. GitHub Actions builds and pushes. | `docker run skillhub --help` exits 0. |
| **19** | Distro packages (`.deb`, `.rpm`, `.apk`) | GitHub Actions workflow with cargo-deb, cargo-rpm, Alpine APK. | `.deb` installs via `dpkg -i`. |
| **20** | `skillhub migrate` | Migrate config and skills on version upgrades. `--rollback` reverts. | Migration with empty config → new format. |

## Implementation Rules

- **No stubs.** Every function is a full implementation. No `todo!()`, `unimplemented!()`, empty match arms.
- **All tests are real.** No `assert!(true)`, `assert_eq!(1, 1)`. Every test checks concrete behavior.
- **Code is readable.** Minimal comments. Natural variable names. No template "AI comments".
- **User-facing strings only in English.** Error messages, help, prompts — all English.
- **No skeletons.** Cannot "reserve" a function with empty body for later.
- **Order of implementation** is determined by dependencies: `--json` → everything else.

---

## Russian (Русская версия)

| # | Задача | Описание | Тесты |
|---|--------|----------|-------|
| **1** | `--json` флаг для всех команд | Каждая команда принимает `--json` и выводит структурированный JSON в stdout. | `search --json` парсится как валидный JSON. |
| **2** | `skillhub upgrade` | Сравнивает версии установленных навыков с registry. Если версия в registry новее — скачивает и заменяет. | Установлена v1.0.0, в registry v2.0.0 → upgrade обновляет файлы. |
| **3** | `skillhub doctor` | Диагностика: GitHub токен, registry cache, директории агентов, права на запись. `--fix` чинит. | Config не существует → доктор пишет «not configured». |
| **4** | Shell completions | `skillhub completions bash/zsh/fish/powershell` генерирует скрипт автодополнения. | Скрипты парсятся без ошибок. |
| **5** | `skillhub share` / `restore` | `share` сериализует установленные навыки, `restore <url>` скачивает и устанавливает. | share+restore roundtrip даёт тот же набор. |
| **6** | Progress bar | `install`, `upgrade`, `update` показывают прогресс-бар. Автоотключается в CI/JSON/no-progress. | `--no-progress` отключает. |
| **7** | `skillhub suggest` | 1 рекомендация в день. Анализирует агентов и популярность. | Без агентов → популярный навык. |
| **8** | `skillhub publish` | Публикация навыка в registry через GitHub PR. Проверяет валидность SKILL.md. | Публикация с пустым name → ошибка. |
| **9** | SkillTest lite | Базовый security-скан при `install`. | Навык с injection → warning. |
| **10** | Homebrew/Scoop/winget | Манифесты для пакетных менеджеров. | (e2e, руками) |
| **11** | VS Code extension | TreeView с реестром навыков, кнопки install/uninstall. | Extension активируется. |
| **12** | `skillhub sync` | Синхронизация навыков между машинами. | Roundtrip export→import. |
| **13** | `skillhub stats` | Статистика: навыки, авторы, агенты. | Пустой реестр → «no data». |
| **14** | Skill badges | Генерация shield.io URL. | `badge --format=markdown` выводит markdown. |
| **15** | `skillhub import` | Импорт из сторонних форматов. | Импорт из `.cursor/rules/` → SKILL.md. |
| **16** | Auto-detect project | Установка в текущий проект. | `--project` в папке с агентом → пишет в проект. |
| **17** | Расширение агентов | Добавление новых AI coding agents 2026. | У каждого агента есть label, detect_home. |
| **18** | Docker образ | Dockerfile + GitHub Actions. | `docker run skillhub --help` → 0. |
| **19** | Пакеты дистрибутивов | .deb, .rpm, .apk через GitHub Actions. | `.deb` устанавливается. |
| **20** | `skillhub migrate` | Миграция конфигурации при обновлении. | Миграция с пустого config → новый формат. |

## Правила реализации

- **Никаких заглушек.** Каждая функция — полная реализация.
- **Все тесты реальны.** Никаких `assert!(true)`.
- **Код читаемый.** Минимум комментариев. Естественные имена.
- **User-facing строки только на английском.**
- **Никаких скелетонов.**
