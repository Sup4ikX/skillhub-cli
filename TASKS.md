*Scroll down for English version / Прокрутите вниз для английской версии*

---

# TASKS.md — SkillHub Sprint 1 & 2

## English (for English description scroll down)

---

## Russian

---

## Спринт 1

| # | Задача | Описание | Тесты |
|---|---|---|---|
| **1** | `--json` флаг для всех команд | Каждая команда (`search`, `list`, `info`, `install`, `uninstall`, `update`, `doctor`, `agents`, `upgrade`) принимает `--json` и вместо цветного terminal-output выводит структурированный JSON в stdout. Флаг добавляется в `Cli` как глобальный. | `search --json` парсится как валидный JSON с полями `{"skills":[...], "count":N}`. `list --json` с пустым списком → `{"skills":[]}`. `list --json` с установленными → все поля Skill присутствуют. `--json` не ломается при пустом кэше. |
| **2** | `skillhub upgrade` | Сравнивает версии установленных навыков с registry. Если версия в registry новее — скачивает и заменяет. `--dry-run` показывает что обновится без изменений на диске. Флаг `--all` пропускает подтверждение. | Установлена v1.0.0, в registry v2.0.0 → upgrade обновляет файлы. Установлена latest версия → upgrade ничего не делает. `--dry-run` логирует но не пишет файлы. Установлено 3 навыка, registry новее для 2 → `--dry-run` показывает только 2. |
| **3** | `skillhub doctor` | Диагностика: GitHub токен (валидность/пусто), registry cache (свежесть), директории агентов существуют, права на запись, config синтаксис. `--fix` чинит: создаёт папки, перезапрашивает токен, пересоздаёт кэш. | Config не существует → доктор пишет «not configured». Config без `skills_dir` → доктор создаёт. Кэш старше 24ч → предупреждение. `--fix` при отсутствующей `cache_dir` → создаёт. |
| **4** | Shell completions | `skillhub completions bash/zsh/fish/powershell` генерирует скрипт автодополнения в stdout. Пользователь делает `source <(skillhub completions zsh)` или редиректит в файл. | `completions bash` начинается с `#compdef skillhub`. `completions zsh` начинается с `#compdef _skillhub`. `completions fish` содержит `complete -c skillhub`. `completions powershell` содержит `Register-ArgumentCompleter`. Скрипты парсятся без ошибок (shellcheck/fish -n). |
| **5** | `skillhub share` / `restore` | `share` собирает список установленных навыков + версий, генерирует URL (gist или прямую ссылку) и однострочную команду `skillhub restore <url>`. `restore <url>` скачивает JSON-список, устанавливает всё чего нет. | `share` корректно сериализует установленные навыки (имя + версия). `restore` из валидного списка устанавливает недостающие, пропускает уже установленные. `restore` с битым URL → понятная ошибка. share+restore roundtrip даёт тот же набор. |
| **6** | Progress bar (`indicatif`) | `install`, `upgrade`, `update` показывают прогресс-бар + ETA. Автоотключается если stdout не TTY, если `--json`, если `--no-progress`, или если `NO_COLOR`/`CI` в окружении. Единый `ProgressBar::new_spinner()` для трекинга. | `--no-progress` отключает progress bar. `--json` автоматически отключает. В CI-окружении (`CI=1`) не падает и не выводит escape-коды. |
| **7** | `skillhub suggest` | 1 рекомендация в день, только по явному вызову команды. Анализирует: какие агенты установлены, какие навыки популярны в registry, какие тэги пересекаются. Хранит `last_suggest_date` в `~/.skillhub/config.toml`. | Без установленных агентов → рекомендует самый популярный навык в registry. Вызвано дважды за день → сообщение «already suggested today, come back tomorrow». Агенты обнаружены → рекомендация учитывает `compatibility`. `last_suggest_date` правильно сохраняется/загружается. |
| **8** | `skillhub publish` | Публикация навыка в registry (через создание GitHub Issue/PR в репозиторий registry, либо pull request). Проверяет: валидность SKILL.md (заголовок, описание, тэги), уникальность имени, заполненность обязательных полей. `--force` пропускает валидацию. | Публикация с пустым `name` → ошибка. Публикация с дублирующимся именем → сообщение «already exists». Валидный навык → создаёт PR/выводит URL. |
| **9** | SkillTest lite (security scan) | Базовый security-скан при `install`: regex-проверка на prompt injection (`ignore previous instructions`, `system prompt`), data exfiltration (`curl | base64`, `ssh -i`), скрытые unicode-символы (zero-width joiners, RTL override). `--no-scan` отключает. | Навык с `ignore previous instructions` → warning. Навык с zero-width space → warning. `.md` без подозрительного контента → «scan passed». `--no-scan` не запускает сканер. |
| **10** | Homebrew tap + Scoop + winget | Homebrew: создать Formula в `skillhub/homebrew-tap` на GitHub, `brew tap skillhub/tap && brew install skillhub`. Scoop: манифест `bucket/skillhub.json`. winget: публикация через GitHub Releases манифест. Все указывают на GitHub Release бинарники. | (e2e, руками) `brew install skillhub --HEAD` → бинарник работает. scoop install не падает. |

## Спринт 2

| # | Задача | Описание | Тесты |
|---|---|---|---|
| **11** | VS Code extension | Extension: TreeView с реестром навыков, кнопки install/uninstall/search, статус-бар с количеством установленных. TypeScript, минимальный bundle, вызов `skillhub` CLI через child_process. README со скриншотами. | Extension активируется при открытии папки. TreeView показывает список из `skillhub list --json`. Install через UI → навык появляется в списке. |
| **12** | `skillhub sync` | Синхронизация набора навыков между машинами: `export` генерирует JSON/TOML со списком навыков + опционально их содержимым (base64). `import <file>` восстанавливает. Можно указать GitHub Gist как remote. | `export` создаёт файл где каждая запись содержит `name` + `version`. `import` устанавливает недостающие, обновляет устаревшие, не трогает свежие. Roundtrip export→import даёт идентичный набор. |
| **13** | `skillhub stats` | Статистика: сколько навыков установлено, каких авторов, сколько агентов, какие навыки самые популярные в registry. Аналоговый «процент пользователей» (локально, без сервера). | Пустой реестр → «no data». 3 навыка от 2 авторов → корректная статистика. |
| **14** | Skill badges | Генерация shield.io-совместимого URL: `![skillhub](https://img.shields.io/badge/skills-12-blue)`. `skillhub badge --format=markdown` выводит markdown-строку для README. | `badge --format=markdown` выводит `![skillhub]...`. `badge --format=url` выводит чистый URL. С 0 навыков → `skills-0-lightgrey`. |
| **15** | `skillhub import` | Импорт навыков из сторонних форматов: `.claude/skills/`, `.cursor/rules/`, `awesome-claude-skills` формат, `.windsurf/rules`. Автоматическая конвертация в `SKILL.md` + `meta.json`. `--dry-run` показывает что будет импортировано. | Импорт из пустой директории → ничего. Импорт `.cursor/rules/` с одним .mdc → корректный SKILL.md. Импорт `awesome-claude-skills` README → парсинг ссылок. |
| **16** | Auto-detect project (`--project`) | `skillhub install <name> --project` — вместо установки в `~/.skillhub/skills/` создаёт файл/ссылку прямо в текущем проекте (например `CLAUDE.md`). Определяет какой агент активен по наличию `.opencode`, `.cursor/rules`, `CLAUDE.md` в корне. | `--project` в папке без агентов → ошибка. `--project` в папке с `.opencode` → пишет в `.opencode/agents/`. `--project` в папке с `CLAUDE.md` → аппендит в него. |
| **17** | Увеличение числа поддерживаемых агентов | Исследование и добавление: исследовать AI coding agents весна-лето 2026 (новые ~10-15 агентов). Проверить: домашняя директория, структура навыков, project-файлы. Meta-агенты (агенты внутри агентов). Добавить в `agent.rs`, обновить таблицу в README, тесты. | Каждый новый агент имеет label, detect_home, skills_dir, project_file. Имена уникальны (тест). label не пустой (тест). detect_home не None (тест). |
| **18** | Docker образ | `Dockerfile` на основе scratch/alpine + skillhub бинарник. `docker run ghcr.io/skillhub/cli search "x"`. GitHub Actions собирает и пушит образ. | `docker run skillhub --help` завершается с 0. `docker run skillhub version` выводит версию. |
| **19** | Пакеты для дистрибутивов (`.deb`, `.rpm`, `.apk`) | GitHub Actions workflow с `cargo-deb`, `cargo-rpm`, Alpine APK. Публикация в GitHub Release. | `.deb` устанавливается через `dpkg -i`. `skillhub` доступен в PATH после установки. |
| **20** | `skillhub migrate` | Миграция конфигурации и навыков при обновлении версий. Если структура `~/.skillhub/` изменилась — автоматический перенос. `--rollback` откатывает. | Миграция с v0 на v1 (пустой config → новый формат). Миграция с v1 на v2 (skills переехали из `~/.skillhub/` в `~/.skillhub/skills/`). |

## Правила реализации

- **Никаких заглушек.** Каждая функция — полная реализация. Никаких `todo!()`, `unimplemented!()`, пустых match-веток или `Default::default()` где нужна реальная логика.
- **Все тесты реальны.** Никаких `assert!(true)`, `assert_eq!(1, 1)`. Каждый тест проверяет конкретное поведение, граничный случай или поток ошибок и может упасть если код сломается.
- **Тесты не прикрывают баги.** Если тест сложно написать — архитектура неправильная. Рефакторить, а не пропускать.
- **Код читаемый.** Минимум комментариев. Естественные имена (`s`, `c`, `p`, `n` и осмысленные длинные имена). Никаких шаблонных «AI-комментариев».
- **User-facing строки только на английском.** Сообщения об ошибках, help, промпты — всё на английском.
- **Никаких скелетонов.** Нельзя «зарезервировать» функцию с пустым телом на потом.
- **Порядок реализации** определяется зависимостями: `--json` → всё остальное. `skilltest-lite` + progress-bar → install/upgrade. Параллельные задачи (apt/rpm/docker, VS Code extension) независимы.
- **Каждый PR/коммит** проходит 3 code review перед мержем.

---

*Scroll down for English version / Прокрутите вверх для английской версии*
