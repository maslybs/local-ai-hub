# План робіт (етапи) для `local-ai-hub`

Джерело вимог: `/Users/bogdanmasliy/Desktop/Робота/Власні-проекти/local-ai-hub/docs/SPEC_UA.md`.

Мета цього документу: перетворити ТЗ на послідовні етапи з чіткими deliverables і “definition of done” (DoD) для кожного кроку.

## Мілстоуни (великі шматки)

1) **UI shell + IPC база**: етапи 0–1  
2) **Персист конфігу + секрети**: етапи 2–3  
3) **Telegram connector (керування + команди)**: етапи 4–5  
4) **Codex connector**: етап 6 (це і є MVP)  
5) **Actions + Jobs/Logs як продуктова функція**: етап 7 (MVP+)  
6) **Полірування + збірки**: етап 8  

## Етап 0: Scaffold + UI Base

**Ціль:** підняти UI-оболонку з навігацією та базовими патернами, як у `my-local-memo-hub`.

**Scope:**
- Vite + React + TypeScript.
- TailwindCSS + Radix primitives + lucide-react.
- Layout: Sidebar + Header + Views.
- Views у MVP: `Overview`, `Telegram`, `Codex`, `Logs/Jobs`.
- Toasts + базова система статусів (поки мок).

**Deliverables:**
- Запуск dev-сервера фронтенду.
- Стабільна навігація між views.

**DoD (перевірка):**
- UI стартує без помилок.
- Навігація працює.

## Етап 1: Backend Skeleton (Tauri v2) + IPC

**Ціль:** мінімальний Tauri бекенд і гарантований канал взаємодії UI ↔ Rust.

**Scope:**
- Tauri v2 scaffold.
- Мінімальний `invoke("ping")`.
- Базовий health/connection статус у UI.

**Deliverables:**
- Команда `ping` у бекенді.
- UI показує “Backend connected” або читабельну помилку.

**DoD (перевірка):**
- `invoke` працює.
- Помилки (якщо є) видно і зрозумілі.

## Етап 2: Config Store (персист налаштувань)

**Ціль:** зберігати/читати налаштування з диска, щоб UI переживав перезапуск.

**Scope:**
- `get_config/save_config`.
- `config.json` у `app_data_dir`.
- Міграції на цьому етапі не потрібні, але формат має бути стабільним.

**Deliverables:**
- Сторінка Settings (мінімальна) або секції на Telegram/Codex views, що реально змінюють конфіг.

**DoD (перевірка):**
- Зміна налаштувань зберігається і після перезапуску лишається.

## Етап 3: Secrets / Telegram Token

**Ціль:** надійне збереження Telegram token (keychain/credential manager) і безпечний UX.

**Scope:**
- `telegram_set_token`, `telegram_delete_token`, `telegram_token_status`.
- Read-back перевірка після запису.
- UI ніколи не показує токен повністю після збереження.
- Явний контрольований fallback (тільки якщо keychain недоступний).

**Deliverables:**
- UX “Save token” + статус `stored/missing`.

**DoD (перевірка):**
- Після `Save token` статус стає `stored`.
- Старт бота не падає з “missing token”.

## Етап 4: Telegram Connector Minimal (polling loop)

**Ціль:** запустити/зупинити бота, бачити стабільний polling і діагностику.

**Scope:**
- Long polling через `getUpdates`.
- `start/stop/status`, `getMe`.
- Персист `offset` у `bot-state.json`.
- Логи подій polling’у (включно з помилками).

**Deliverables:**
- Кнопки Start/Stop в UI.
- Статус: `running/stopped`, `lastPoll`, `lastError`.

**DoD (перевірка):**
- При `running` оновлюється `lastPoll`.
- Помилки видно у Logs.
- Після Stop бот не продовжує працювати у фоні.

## Етап 5: Telegram Commands + Access Control

**Ціль:** реалізувати команди, allowlist і “безпечний” контроль доступу.

**Scope:**
- Доступні завжди: `/start`, `/whoami`.
- Тільки allowlisted `chat_id`: `/ping`, `/help`, `/codex <prompt>`, `/review [prompt]`, `/actions`, `/run <action>`.
- Підтримка `/cmd@BotName` (групи).
- `allowedChatIds` редагується у UI.
- Chunking відповідей (4096 символів Telegram).

**Deliverables:**
- Telegram view з allowlist менеджментом.
- Команди `/whoami` і `/ping` повністю працюють.

**DoD (перевірка):**
- `/whoami` відповідає в приваті і як `/whoami@BotName` в групі.
- `/ping` працює тільки після додавання `chat_id` в allowlist.

## Етап 6: Codex Connector (MVP core)

**Ціль:** викликати Codex через CLI локально і повертати результат в UI та Telegram.

**Scope:**
- Запуск зовнішнього процесу: `codex exec`, `codex review`.
- Налаштування: `workspaceDir`, `binaryPath` (опц.), `model`, `reasoningEffort` (опц.), `timeoutMs`.
- Читабельні помилки: stderr + exit code + timeout.
- Відповідь: UI output panel + Telegram (chunked).

**Deliverables:**
- Codex view: тестовий запуск Exec/Review.
- Telegram: `/codex`, `/review`.

**DoD (перевірка):**
- Відповідь приходить і в UI, і в Telegram.

## Етап 7: Actions + Jobs/Logs (MVP+)

**Ціль:** керований запуск allowlisted локальних команд і видима історія виконань.

**Scope:**
- Actions (конфіг): `name`, `cmd[]`, `timeoutMs`, `env`.
- UI: CRUD actions + Run.
- Telegram: `/actions`, `/run <name>`.
- Jobs: `id/kind/status/timestamps/exitCode/summary`.
- Logs: `info/warn/error`, фільтр + пошук, ліміти.

**Deliverables:**
- Actions UI + Jobs list.

**DoD (перевірка):**
- Запуск action створює job.
- Видно stdout/stderr.
- Нема “довільного shell-рядка” за замовчуванням (тільки allowlist).

## Етап 8: Полірування + Релізні збірки

**Ціль:** довести додаток до стану “можна користуватись щодня”.

**Scope:**
- Tray: show/hide/quit.
- Автооновлення статусів у UI.
- Документація “How to” (швидкий старт, troubleshooting).
- Перевірка стабільності (бот не “зникає”, логи завжди пояснюють, що сталось).

**Deliverables:**
- `tauri build` для macOS + Windows.
- Документація.

**DoD (перевірка):**
- Стабільна робота без “зникання”.
- Ключові сценарії з ТЗ проходять руками.

