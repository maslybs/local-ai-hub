# Технічне завдання (ТЗ)
**Проєкт:** `local-ai-hub`  
**Формат:** локальний десктоп-хаб для інтеграції AI-інструментів  
**Платформи (мінімум):** macOS + Windows  
**Технології:** Tauri v2 + Rust backend, Vite + React + TypeScript frontend

---

## 1) Контекст і проблема
Є дві ідеї, які треба об’єднати:

- **`my-local-memo-hub`**: хаб з гарним UI/UX та “конекторною” архітектурою (інструменти як модулі/сервіси).
- **`codex-tg-hub`**: ідея використовувати **Telegram як інтерфейс** до **локальних** дій та Codex.

Поточний прототип має проблеми стабільності й UX (зокрема токен/старт бота). Тому робимо **новий проєкт у новій папці**, поетапно, з перевіркою кожного кроку.

---

## 2) Цілі
- Зробити **локальний AI Hub-додаток** з приємним сучасним UI (перенести підхід/візуальну мову з `my-local-memo-hub`).
- Перший “конектор” (MVP): **Telegram Bot Connector** (long polling, без webhook, без сервера).
- Другий “конектор” (частина MVP): **Codex Connector** (виклик Codex через CLI).
- Забезпечити **надійне збереження токена** (keychain/credential manager як основний шлях, контрольований fallback за потреби).
- Модульність: чітка структура, мінімум дублювання, окремі фічі/модулі.

---

## 3) Нецілі (на MVP)
- Проксі/перехоплення внутрішніх мережевих запитів Codex Desktop UI.
- “App-server” Codex як інтеграція на рівні приватних API.
- Віддалений хостинг або постійний бекенд-сервер.
- “Маркетплейс” конекторів. На старті тільки базова архітектура + 1–2 конектори.

---

## 4) Терміни та визначення
- **Hub**: десктоп-додаток, що керує локальними AI-інструментами/конекторами.
- **Connector**: модуль інтеграції (Telegram, Codex, далі інші).
- **Job**: одиниця виконання (команда, `codex exec/review`), з статусом і результатом.
- **Workspace**: каталог (cwd), де виконуються локальні команди і де Codex читає контекст.

---

## 5) Ключові сценарії (user stories)
- Як користувач, я додаю токен Telegram, запускаю бота, пишу `/whoami` і отримую `chat_id`.
- Як користувач, я allowlist’ю свій `chat_id`, після чого `/ping` працює.
- Як користувач, я пишу в Telegram `/codex ...` і отримую відповідь від Codex.
- Як користувач, я бачу у UI статус бота, останні помилки, історію jobs і логи.
- Як користувач, я можу зупинити бота і бути впевненим, що він не працює у фоні.
- Як користувач, я хочу поступово перевіряти фічі крок за кроком.

---

## 6) Функціональні вимоги

### 6.1 UI/UX (взяти як базу `my-local-memo-hub`)
- Патерн: **Dashboard з Sidebar + Header + Views** (як у `my-local-memo-hub`).
- Мінімум 4 views у MVP:
  - **Overview**: загальний стан (Hub, Telegram, Codex).
  - **Telegram**: керування ботом і налаштування доступу.
  - **Codex**: налаштування і тестовий запуск (exec/review) з UI.
  - **Logs/Jobs**: стрічка логів та історія виконань.
- Налаштування мають бути структуровані (не “все в одному блоці”).
- Токен ніколи не показується повністю після збереження.
- Стани явні: `running/stopped`, `token stored/missing`, `last error`, `last poll`.
- Кожна дія (Save token / Start / Stop / Exec) дає toast + запис у logs.

### 6.2 Telegram Connector (MVP)
- Режим: **long polling** через `getUpdates`.
- Команди:
  - Доступні завжди: `/start`, `/whoami` (повертають `chat_id` і інструкцію, як додати в allowlist).
  - Тільки allowlisted `chat_id`: `/ping`, `/help`, `/codex <prompt>`, `/review [prompt]`, `/actions`, `/run <action>`.
- Підтримка `/cmd@BotName` (важливо для груп).
- `allowedChatIds`: список `i64`, редагується у UI.
- `pollTimeoutSec`: налаштовується у UI.
- `offset` зберігається на диск (щоб не отримувати старі апдейти повторно).

### 6.3 Codex Connector (MVP)
- Виклик Codex як зовнішнього процесу:
  - `codex exec` з отриманням “останнього повідомлення” як відповіді.
  - `codex review` для рев’ю репозиторію/змін (опціонально з промптом).
- Налаштування:
  - `workspaceDir` (cwd).
  - `binaryPath` (опціонально).
  - `model`, `reasoningEffort` (опціонально, advanced).
  - `timeoutMs`.
- Відповідь повертається:
  - у Telegram (chunked, якщо довга).
  - у UI (output panel).
- Помилки: читабельні (stderr + exit code + timeout).

### 6.4 Actions (allowlisted локальні команди) (MVP або MVP+)
- Список actions в конфігу:
  - `name`
  - `cmd` (масив аргументів)
  - `timeoutMs`
  - `env`
- Telegram: `/actions`, `/run <name>`.
- UI: вибір action і `Run`.
- Безпека: тільки allowlist (без довільного shell-рядка за замовчуванням).

### 6.5 Jobs + Logs
- Jobs:
  - `id`, `kind`, `status`, `createdAt`, `startedAt`, `finishedAt`, `exitCode`, `summary`.
- Логи:
  - рівні `info/warn/error`
  - події Telegram polling, команди, старт/стоп, запуск процесів, помилки
  - UI view з фільтром і пошуком
- Ліміти: logs ~ 1000–2000, jobs ~ 50–200.
- (Опціонально пізніше) персист jobs на диск.

### 6.6 Сховище конфігів і секретів
- `config.json` у `app_data_dir`.
- Telegram token:
  - Основний шлях: OS keychain / Windows Credential Manager.
  - UI після `Save token` робить read-back перевірку.
  - Якщо keychain недоступний: показати явну помилку і запропонувати fallback режим.
- `bot-state.json`: offset.

---

## 7) Нефункціональні вимоги
- Платформи: **macOS + Windows** (мінімум), збірка `tauri build` для обох.
- Надійність: бот не має “зникати” без зрозумілої причини.
- Дебажність: будь-яка помилка видима в Logs.
- Безпека: токени не логуються; plaintext fallback лише в контрольованому режимі.
- UX: прозорі статуси, без “магії”.

---

## 8) Техстек і архітектура (пропозиція)
**Frontend:**
- Vite + React + TypeScript
- TailwindCSS
- Radix UI primitives
- lucide-react icons
- View-based структура `src/features/...`, reusable компоненти `src/components/...`

**Backend (Tauri v2):**
- Rust модулі:
  - `connectors/telegram/*`
  - `connectors/codex/*`
  - `core/config_store.rs`
  - `core/secrets.rs`
  - `core/process_runner.rs`
  - `core/jobs.rs`
  - `core/logbus.rs` (еміт подій в UI)

**IPC (приблизно):**
- `get_config`, `save_config`
- `telegram_token_status`, `telegram_set_token`, `telegram_delete_token`
- `telegram_start`, `telegram_stop`, `telegram_status`
- `codex_exec`, `codex_review`
- `actions_run`, `jobs_list`
- `logs_subscribe` (через Tauri events)

---

## 9) Структура нового проєкту (в новій папці)
Приклад:
- `local-ai-hub/`
- `local-ai-hub/src/` (React app)
- `local-ai-hub/src/features/telegram/...`
- `local-ai-hub/src/features/codex/...`
- `local-ai-hub/src/features/logs/...`
- `local-ai-hub/src/components/ui/...`
- `local-ai-hub/src-tauri/src/core/...`
- `local-ai-hub/src-tauri/src/connectors/telegram/...`
- `local-ai-hub/src-tauri/src/connectors/codex/...`

---

## 10) Поетапний план (кожен крок перевіряється)

### Етап 0: Scaffold + UI base
- Створити нову папку проєкту.
- Підняти Vite+React+Tailwind UI з sidebar/header/views як у `my-local-memo-hub`.
- **Критерій:** UI запускається, навігація між views працює.

### Етап 1: Backend skeleton
- Додати Tauri v2 бекенд, мінімальний `invoke("ping")`.
- Показати у UI “Backend connected”.
- **Критерій:** `invoke` працює, errors зрозумілі.

### Етап 2: Config store
- `get_config/save_config`, зберігання `config.json`.
- **Критерій:** Settings зберігаються і переживають перезапуск.

### Етап 3: Secrets/token
- `telegram_set_token` + read-back перевірка.
- UI badge: stored/missing.
- **Критерій:** після Save token badge стає stored і Start Bot не каже “missing”.

### Етап 4: Telegram bot minimal
- `start/stop/status`, `getMe`, polling loop без команд.
- **Критерій:** status running, `lastPoll` оновлюється, помилки видно.

### Етап 5: Telegram commands
- `/start` `/whoami` завжди.
- allowlist + `/ping`.
- **Критерій:** `/whoami` відповідає в приваті і як `/whoami@BotName` в групі.

### Етап 6: Codex connector
- UI: тест `Exec/Review`.
- Telegram: `/codex`, `/review`.
- **Критерій:** відповідь приходить і в UI, і в Telegram.

### Етап 7: Actions + Jobs
- CRUD actions в UI.
- `/actions` `/run`.
- Jobs list у UI.
- **Критерій:** запуск action створює job, видно stdout/stderr.

### Етап 8: Полірування
- Tray (show/hide/quit).
- Автооновлення статусу.
- Документація “How to”.
- **Критерій:** стабільна робота без “зникання”, з нормальними повідомленнями.

---

## 11) Ризики
- Keychain/Credential Manager може вести себе по-різному в dev/build: потрібна read-back перевірка і явний fallback.
- Codex CLI може бути не в PATH на Windows/macOS: потрібен `binaryPath` (+ авто-детект пізніше).
- Telegram API обмеження 4096 символів: потрібен chunking.

