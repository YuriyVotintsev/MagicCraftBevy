# ТЗ: Система баланса для Magic Craft Bevy из xlsx

## Обзор

Система загрузки игрового баланса из локального файла `assets/balance.xlsx`. В dev-режиме (`--features dev`) файл читается с диска при старте и перечитывается по F5 (горячая перезагрузка). В сборке без `feature = "dev"` файл вшивается в бинарник через `include_bytes!`.

Система заменяет текущий RON-пайплайн балансных файлов (`assets/balance.ron`, `assets/mobs.ron`), загружаемых через `RonAssetLoader` в `src/loading/`. Структурные ассеты (`assets/particles/*.particle.ron`, `assets/palette.ron`) остаются в RON и продолжают загружаться через `AssetLoader`/`LoadedFolder`.

Редактирование баланса — в Excel или LibreOffice Calc. Claude Code при необходимости правит тот же файл через Excel MCP-сервер (см. раздел «Инструментарий»).

---

## Контекст архитектуры (важно)

После удаления data-driven blueprint-системы, FSM, DSL выражений и генерик-абилок (коммиты `9034dac`, `0a85951`, `ed03379`, `7586174`, `c1d36c6`) в RON остались **только числа**. Структура мобов (компоненты, AI, визуалы, коллайдеры) и стат-система (`enum Stat` в `src/stats/registry.rs`) — хардкод в Rust. Абилки мобов сколокированы с самими мобами в `src/actors/mobs/{ghost,tower,slime,jumper,spinner,melee_attack}.rs`: числа атаки/выстрела лежат в per-mob `*Stats`-структурах (например, `TowerStats` содержит `flight_duration`/`arc_height`/`explosion_radius` и т.д., раньше живших в `tower_shot.ability.ron`). Абилка игрока (`fireball`) — хардкод-константы в `src/actors/player.rs` (`FIREBALL_BASE_DAMAGE`/`FIREBALL_COOLDOWN`/...), в xlsx не выносится.

Соответственно xlsx покрывает **весь числовой баланс мобов + глобалы + волны**. После миграции `balance.ron` и `mobs.ron` удаляются, `GameBalance`/`MobsBalance` как `RonAsset` — тоже.

### Что в xlsx, что в RON/Rust

| Область | Источник | Комментарий |
|---------|----------|-------------|
| Числовой баланс мобов + их абилок (`assets/mobs.ron` целиком) | **xlsx** (MobsSpecific) | Типизированные per-mob структуры `GhostStats`/`TowerStats`/`SlimeSmallStats`/`JumperStats`/`SpinnerStats` в `src/actors/mobs/{ghost,tower,slime,jumper,spinner}.rs`. Числа абилки моба лежат прямо в его `*Stats`. |
| Числа абилки игрока (`fireball`) | Rust (const) | `FIREBALL_*` в `src/actors/player.rs`. В xlsx **не выносится** — игрок пока ровно один, балансится правкой кода. Если появятся варианты/герои — тогда пересмотр. |
| Структура мобов (компоненты, AI, визуалы, эффекты) | Rust (`src/actors/`) | xlsx не описывает. Изменение — правка кода. |
| Стат-система (`enum Stat`, калькуляторы) | Rust (`src/stats/`) | Хардкод. RON-конфига статов больше нет. |
| Волны (сегменты рампа + пул мобов + рамп арены) | **xlsx** (Waves) | См. раздел Waves. На момент миграции код использует единый рамп (`balance.ron::wave`/`arena`); переход на per-segment — часть миграции. |
| Глобальные параметры (кроме полей из `wave`/`arena`, ставших per-segment) | **xlsx** (Globals) | См. раздел Globals. |
| Визуалы частиц (`assets/particles/*.particle.ron`) | RON | Не трогается. |
| Палитра (`assets/palette.ron`) | RON | Не трогается. |

---

## Структура xlsx

Один файл `assets/balance.xlsx` содержит четыре листа: **MobsSpecific**, **MobsCommon**, **Waves**, **Globals**. Парсер читает три из них — MobsSpecific, Waves, Globals. MobsCommon — вспомогательный вид для дизайнера, в код не подгружается.

### Лист "MobsSpecific" (source of truth)

Числовой баланс мобов и их абилок (после `7586174` числа абилки моба живут в его `*Stats`). Вертикальный формат в две колонки: `key | value`. Мобы описываются блоками. Блок начинается со строки, где `key = id` и `value` — идентификатор моба (`ghost`, `tower`, `slime_small`, `jumper`, `spinner`, совпадает с `MobKind::id()` в `src/actors/mobs/mod.rs`). Остальные строки блока — пары `key | value` для полей его `*Stats`-структуры (в любом порядке). Следующая строка `key = id` начинает новый блок. Пустые строки между блоками допустимы и игнорируются.

Пример:

| key | value |
|-----|-------|
| id | ghost |
| hp | 10 |
| damage | 3 |
| speed | 150 |
| size | 40 |
| mass | 1 |
| melee_range | 60 |
| melee_cooldown | 1.2 |
| visible_distance | 400 |
| invisible_distance | 600 |
|  |  |
| id | tower |
| hp | 25 |
| damage | 5 |
| size | 48 |
| shot_cooldown | 2.0 |
| flight_duration | 1.5 |
| arc_height | 200 |
| start_elevation | 30 |
| spread | 10 |
| projectile_size | 20 |
| explosion_radius | 80 |
| explosion_duration | 0.4 |
| indicator_duration | 0.6 |
|  |  |
| id | spinner |
| hp | 40 |
| damage | 8 |
| size | 56 |
| mass | 2 |
| spike_length | 1.2 |
| idle_duration | 1.5 |
| windup_duration | 1.0 |
| charge_duration | 1.2 |
| cooldown_duration | 1.0 |
| charge_speed | 600 |

**Ключи внутри блока** соответствуют полям per-mob `*Stats`-структуры в `src/actors/mobs/{ghost,tower,slime,jumper,spinner}.rs`. Неизвестный для данного `id` ключ — ошибка валидации. Отсутствие обязательного поля — ошибка валидации. Дубликат ключа в одном блоке — ошибка.

**Отладочные ключи** с префиксом `_` (`_name`, `_notes`) игнорируются парсером.

**Почему именно так:** блочный key-value формат даёт нулевую разреженность — у каждого моба только его поля, без широкой таблицы с пустыми ячейками. Новое поле для моба = одна строка; новый моб = новый блок; новая `*Stats`-структура в Rust задаёт список допустимых ключей автоматически.

### Лист "MobsCommon" (авто-производный, не парсится)

Горизонтальная таблица для удобства балансировки общих параметров (hp/damage/size/mass и любых других, которые дизайнер хочет видеть колонкой для сравнения между мобами).

**Строится формулами из MobsSpecific**, руками не заполняется. Парсер его **игнорирует** — source of truth это MobsSpecific.

Пример структуры (конкретный набор колонок — на усмотрение дизайнера):

| id | hp | damage | size | mass |
|----|-----|--------|------|------|
| ghost | =формула | =формула | =формула | =формула |
| tower | =формула | =формула | =формула | =формула |
| slime_small | ... | ... | ... | ... |
| jumper | ... | ... | ... | ... |
| spinner | ... | ... | ... | ... |

Колонка `id` — `=UNIQUE(FILTER(MobsSpecific!B:B; MobsSpecific!A:A="id"))` (Excel 365 / LibreOffice 7+) или эквивалент через `INDEX`/`SMALL`.

Каждая value-колонка — лукап «в блоке с данным id найти строку с нужным key и вернуть value». Рабочий рецепт: вспомогательная колонка в MobsSpecific, разворачивающая текущий id вниз по блоку (last-non-empty-above через `LOOKUP`), и `XLOOKUP` по паре (id, key) из MobsCommon. Добавить новую общую колонку = скопировать формулу с новым именем ключа.

**Побочный бонус**: колонка `id` из MobsCommon — удобный источник для Data Validation у `enemy_id` на листе Waves.

### Лист "Waves"

Сегменты боя. Каждый сегмент задаёт параметры рампа врагов, пул мобов и параметры рампа арены. Сегменты идут последовательно, переключаются по истечении `duration_sec`.

На момент миграции код держит единый рамп на всю игру: `balance.wave.start_enemies/max_enemies/ramp_duration_secs` и `balance.arena.start_width/start_height/width/height`. Переход на сегменты — часть миграции (см. «Миграция потребителей»).

Лист содержит два типа строк, различаемых колонкой `kind`: `segment` и `pool`.

| Колонка | Тип | Описание |
|---------|-----|----------|
| wave | int | Номер сегмента (начиная с 1). |
| kind | string (dropdown) | `segment` или `pool`. |
| duration_sec | float | (segment) сколько длится сегмент до переключения на следующий. |
| start_enemies | int | (segment) начальное число одновременных врагов в сегменте. |
| max_enemies | int | (segment) максимум к концу сегмента. |
| ramp_duration_sec | float | (segment) за сколько `start_enemies` → `max_enemies`. |
| arena_start_width | float | (segment) ширина арены в начале сегмента. |
| arena_start_height | float | (segment) высота арены в начале сегмента. |
| arena_end_width | float | (segment) ширина арены к концу сегмента. |
| arena_end_height | float | (segment) высота арены к концу сегмента. |
| enemy_id | string (dropdown) | (pool) ссылка на `id` из MobsCommon (т.е. из MobsSpecific). |
| weight | float | (pool) вес моба в пуле сегмента. |
| _enemy_name | string (формула) | **Отладочная.** `=XLOOKUP(enemy_id; MobsCommon!A:A; MobsCommon!A:A)` или любая подпись. |

**Правила:**
- Для каждого `wave` должна быть ровно одна строка `kind = segment` и ≥1 строк `kind = pool`.
- Пул сегмента активен всё время сегмента; спавн выбирает моба случайно по весам (сейчас в `src/wave/spawn.rs::spawn_enemies` выбор равновероятный из `EnemySpawnPool.active_kinds()`; миграция добавляет веса).
- Арена плавно интерполируется от `arena_start_*` к `arena_end_*` внутри сегмента (линейно). При переходе на следующий сегмент арена начинается с `arena_start_*` следующего сегмента (возможен скачок, если дизайнер не выровняет значения).
- `delay_sec` (интервал между спавнами отдельных мобов) не используется — темп спавна задаётся только рампом `start_enemies → max_enemies` и лимитом одновременных врагов.

**Взаимодействие с dev-меню.** При `--features dev` есть меню `EnemySpawnPool` с toggles на каждый тип моба (`src/wave/spawn.rs`). Если dev-меню активно использовалось (пользователь менял тогглы), оно **полностью замещает** пул из текущего сегмента Waves — сегмент в этот момент диктует только параметры рампа и арены, а выбор мобов идёт из dev-оверрайда. Флаг `EnemySpawnPool.dev_override: bool` взводится первым кликом в меню.

### Лист "Globals"

Глобальные настройки, не привязанные к волнам/аренам. Вертикальный формат ключ-значение.

Из `assets/balance.ron` сюда переезжают все поля **кроме** тех, что стали per-segment: `start_enemies`, `max_enemies`, `ramp_duration_secs`, `start_width`, `start_height`, `width`, `height` ушли в Waves. Остаются:

- `safe_spawn_radius` (секция `wave`)
- `shop_delay` (секция `wave`)
- `coins_per_kill` (секция `run`)
- `hp_scale_per_sec`, `dmg_scale_per_sec` (секция `run`)
- `coin_attraction_duration` (секция `run`)

Колонки:

| Колонка | Тип | Описание |
|---------|-----|----------|
| key | string | Уникальный ключ параметра. |
| value | string | Значение (строка, тип диктуется кодом-потребителем). |
| _description | string | **Отладочная.** Описание для дизайнера. |

---

## Валидация в Excel (UX для дизайнера)

### Data Validation (Dropdown)

- `enemy_id` на Waves: Data Validation → List → source `=MobsCommon!$A$2:$A`, стиль Stop. MobsCommon автоматически содержит полный список id из MobsSpecific, валидация всегда актуальна.
- `kind` на Waves: Data Validation → List → `segment,pool`.
- Значение в строке `key = id` на MobsSpecific (колонка B) — свободный ввод; подсветка незнакомых id через Conditional Formatting (ниже).

### Conditional Formatting (подсветка битых ссылок)

На Waves, колонка `enemy_id`, формула красной заливки:

```
=ISNA(MATCH(B2; MobsCommon!$A$2:$A; 0))
```

На MobsSpecific, колонка B в строках где `A="id"`: заливка если `value` не совпадает ни с одним `MobKind` (список можно захардкодить формулой или держать на скрытом листе `MobKinds`).

### Рекомендация по удалению мобов

Вместо удаления блока из MobsSpecific добавить в блок строку `deprecated | TRUE`. При экспорте такой блок исключается, но id остаётся в MobsCommon и в dropdown-ах (опционально можно скрыть в MobsCommon дополнительной формулой).

---

## Инструментарий

### Редактирование дизайнером

Основной путь — Excel или LibreOffice Calc. Файл `assets/balance.xlsx` хранится в репозитории под git (бинарный — diff не работает, merge conflict не резолвится штатно; для соло-работы это приемлемо, для командной — договариваться о монопольной правке).

### Редактирование Claude Code через Excel MCP-сервер

Для того чтобы Claude Code мог править баланс напрямую, а не через одноразовые скрипты, подключается community MCP-сервер для локальных xlsx (например, [`haris-musa/excel-mcp-server`](https://github.com/haris-musa/excel-mcp-server) — Python, без сети/OAuth). После установки у Claude появляются тулы уровня `read_sheet`/`write_cells`/`apply_formula`/`create_sheet` над локальными файлами.

**Установка (пример для `haris-musa/excel-mcp-server`):**

1. `pip install excel-mcp-server` (или `uvx excel-mcp-server`, зависит от способа запуска в проекте сервера).
2. Добавить запись в `~/.claude.json` (или `.claude/mcp.json` в корне проекта) по образцу:
   ```json
   {
     "mcpServers": {
       "excel": {
         "command": "uvx",
         "args": ["excel-mcp-server", "--root", "D:/Projects/MagicCraftBevy/assets"]
       }
     }
   }
   ```
3. Перезапустить Claude Code. Появятся `mcp__excel__*` тулы.

Точные имена тулов и флагов зависят от выбранного сервера — сверяться с его README. Кандидаты помимо `haris-musa/excel-mcp-server`: `excel-mcp`, `openpyxl-mcp` и т.п.

**Use case:** через этот же сервер выполняется и **первоначальное создание** `assets/balance.xlsx` — Claude Code читает текущие `assets/mobs.ron` + `assets/balance.ron` и через MCP-тулы создаёт файл с четырьмя листами, формулами MobsCommon и Data Validation на Waves. Отдельной утилиты-мигратора нет. Дальнейшие точечные правки (добавление нового поля в `*Stats` после рефакторинга и т.п.) тоже идут через MCP; массовые правки значений дизайнер делает руками в Excel.

---

## Загрузка и парсинг

### Зависимости рантайма

```toml
[dependencies]
calamine = "0.26"   # чтение xlsx

# build-dependencies не нужны — нет сетевой загрузки
```

`calamine` читает xlsx из `&[u8]` (`Cursor<Vec<u8>>`) либо из пути на диске — одинаково работает и для `include_bytes!`, и для F5-перечитывания.

### Встраивание в бинарник (release)

```rust
#[cfg(not(feature = "dev"))]
const BALANCE_XLSX: &[u8] = include_bytes!("../../assets/balance.xlsx");
```

Гейтинг по `feature = "dev"` — симметрично dev-рантайму.

### Структуры данных

```rust
use std::collections::HashMap;

// Один блок из MobsSpecific: id моба + его поля.
#[derive(Debug, Clone)]
struct MobBlock {
    id: String,
    fields: HashMap<String, String>, // key → raw value (числа тоже как строки, парсятся на resolve-стадии)
}

#[derive(Debug, Clone)]
struct WaveRow {
    wave: u32,
    kind: String, // "segment" | "pool"
    duration_sec: Option<f32>,
    start_enemies: Option<u32>,
    max_enemies: Option<u32>,
    ramp_duration_sec: Option<f32>,
    arena_start_width: Option<f32>,
    arena_start_height: Option<f32>,
    arena_end_width: Option<f32>,
    arena_end_height: Option<f32>,
    enemy_id: Option<String>,
    weight: Option<f32>,
}

#[derive(Debug, Clone)]
struct GlobalRow {
    key: String,
    value: String,
}

#[derive(Debug, Clone)]
struct WaveSegment {
    number: u32,
    duration_sec: f32,
    start_enemies: u32,
    max_enemies: u32,
    ramp_duration_sec: f32,
    arena_start: Vec2, // width, height
    arena_end: Vec2,
    pool: Vec<WaveSpawn>,
}

#[derive(Debug, Clone)]
struct WaveSpawn {
    enemy_id: String,
    weight: f32,
}
```

### Алгоритм парсинга MobsSpecific

1. Открыть workbook через `calamine::open_workbook_from_rs` (release) или `calamine::open_workbook` (dev, из файла).
2. Получить лист `MobsSpecific` как `Range<DataType>`.
3. Пройти по строкам, извлекая пару `(key, value)` из колонок A и B как строки (числа конвертируются через `DataType::to_string()`).
4. Пропустить пустые строки и строки с `key`, начинающимся на `_`.
5. На строке `key == "id"` — закрыть текущий блок (если был) и открыть новый с `id = value`. Дубликат id — ошибка.
6. Иначе — добавить пару в `fields` текущего блока. Дубликат ключа в блоке — ошибка. Ключ до первой `id`-строки — ошибка.
7. По исчерпании ввода — вернуть `Vec<MobBlock>`.

### Алгоритм парсинга Waves / Globals

Первая строка листа — заголовок. Колонки с заголовком, начинающимся на `_`, отбрасываются. Оставшиеся колонки мапятся по имени на поля `WaveRow`/`GlobalRow`. Значения конвертируются через `DataType` (числа → `f32`/`u32` напрямую, строки — как есть, `Empty` → `None` для `Option<_>`).

---

## Сборка данных (resolve)

1. **MobsBalance.** Для каждого `MobKind` ищем блок с соответствующим `id` (`ghost`/`tower`/`slime_small`/`jumper`/`spinner`). Из `HashMap` блока заполняем соответствующую `*Stats`-структуру (числа — `str::parse::<f32>()`/`parse::<u32>()`; ошибка парсинга или отсутствие обязательного поля — валидационная ошибка). Неизвестный ключ в блоке — ошибка.
2. **Сегменты волн.** Группировка `WaveRow` по полю `wave`. Из строки `kind=segment` — параметры сегмента и арены; из строк `kind=pool` — пул.
3. **Globals.** Строки листа Globals → `HashMap<String, String>` → типизированная `Globals`-структура.

---

## Валидация в коде (runtime)

Вся балансная валидация живёт в `src/balance/loader.rs` и исполняется при загрузке и F5-перезагрузке.

### Проверки

1. **Уникальность id блоков** в MobsSpecific.
2. **id блока — валидный `MobKind`.** Незарегистрированный id — ошибка. Непокрытый `MobKind` (нет соответствующего блока) — тоже ошибка (у игры есть моб без статов).
3. **Полнота и корректность полей блока** для каждого `MobKind`: все обязательные поля заполнены, все ключи известны соответствующей `*Stats`, дубликатов ключей нет, значения парсятся в нужный тип.
4. **Ссылки Waves → MobsSpecific.** `enemy_id` существует среди id блоков и не помечен `deprecated`.
5. **Структура Waves.** Для каждого `wave` — ровно одна строка `kind=segment` с заполненными `duration_sec`/`start_enemies`/`max_enemies`/`ramp_duration_sec` и четырьмя `arena_*`; ≥1 строка `kind=pool` с непустыми `enemy_id`/`weight > 0`. Номера сегментов идут без разрывов, начиная с 1.
6. **Globals.** Все ожидаемые ключи (`safe_spawn_radius`, `shop_delay`, `coins_per_kill`, `hp_scale_per_sec`, `dmg_scale_per_sec`, `coin_attraction_duration`) присутствуют и парсятся в ожидаемые типы.
7. **Фильтрация deprecated.** Блоки с ключом `deprecated = TRUE` исключаются; ссылки на них в Waves — предупреждение.

### Поведение при ошибках

- **`--features dev`**: panic с полным списком ошибок на старте; F5-перезагрузка при ошибке оставляет прежний `Balance` и логирует `error!`.
- **Release без dev**: `error!()`, пропуск невалидных записей, игра не крашится.

---

## Bevy-интеграция

### Ресурс

```rust
#[derive(Resource)]
struct Balance {
    mobs: MobsBalance,   // переиспользуем типы из src/actors/mobs/{ghost,tower,slime,jumper,spinner}.rs
    waves: Vec<WaveSegment>,
    globals: Globals,    // типизированная структура с полями из листа Globals
}
```

### Потребление

Текущие потребители переключаются на `Balance` (и на текущий сегмент волн):

- **`src/wave/mod.rs`** (`WaveState::new`, `reset_wave_state`): `shop_delay` из Globals; `start_enemies`/`max_enemies`/`ramp_duration` теперь диктуются текущим `WaveSegment`.
- **`src/wave/spawn.rs`** (`reset_arena_size`, `update_arena_size`, `update_target_count`, `spawn_enemies`): `arena_start/arena_end`, `start_enemies`, `max_enemies`, `ramp_duration_sec` берутся из текущего сегмента; `safe_spawn_radius`, `hp_scale_per_sec`, `dmg_scale_per_sec` — из Globals. `MobsBalance` берётся из `Balance.mobs` вместо отдельного ресурса.
- **`src/run/coin.rs`**, **`src/run/money.rs`**: `coins_per_kill`, `coin_attraction_duration` — из Globals.
- **`src/arena/mod.rs`** (`spawn_arena`, `update_walls`, `update_floor_mesh`): читает `CurrentArenaSize` (рантайм-ресурс, пересчитывается из текущего сегмента в `src/wave/spawn.rs::update_arena_size`). Первоначальный размер арены при `MainMenu` тоже берётся из первого сегмента.
- **`src/actors/mobs/mod.rs::spawn_mob`** и per-mob `spawn_*` (`ghost.rs`/`tower.rs`/`slime.rs`/`jumper.rs`/`spinner.rs`) — получают `&MobsBalance` через `&Balance.mobs`. Числа абилок моба (tower-shot/jumper-shot/melee) теперь часть его `*Stats` и не требуют отдельного ресурса.

Добавляется новая система-оркестратор сегментов: `advance_wave_segment` тикает `run_state.elapsed_in_segment`, переключает `current_segment_index` по истечении `duration_sec`, обновляет `WaveState.max_concurrent`, `CurrentArenaSize` и активный пул.

### Загрузка

Загрузка идёт **вне Bevy `AssetServer`** и не использует `AssetLoader`/`LoadedFolder`. Это отдельный путь, параллельный `src/loading/`.

Система `setup_balance` на `Startup`:

- **Release** (без `feature = "dev"`): `calamine::open_workbook_from_rs(Cursor::new(BALANCE_XLSX))`.
- **Dev** (`--features dev`): `calamine::open_workbook("assets/balance.xlsx")`.

### Горячая перезагрузка (только `--features dev`)

Система `reload_balance` в `Update`:

- Отслеживает `F5`.
- Перечитывает `assets/balance.xlsx` с диска, парсит, валидирует.
- При успехе — заменяет `Balance`, логирует `info!("Balance reloaded")`.
- При ошибке — `error!(...)`, старый `Balance` сохраняется.

Гейтинг: `#[cfg(feature = "dev")]`.

Альтернатива F5 — автоматический watch файла через `notify` crate. На первом этапе не делаем, чтобы не тянуть лишнюю зависимость; F5 даёт явный контроль.

### Регистрация в App

```rust
app.add_systems(Startup, setup_balance);

#[cfg(feature = "dev")]
app.add_systems(Update, reload_balance);
```

### Удаляется из кодовой базы при миграции

- `assets/balance.ron`, `assets/mobs.ron`.
- `src/balance.rs` (`GameBalance`, `WaveBalance`, `ArenaBalance`, `RunBalance`) — замещается модулем `src/balance/`.
- Реализации `RonAsset for GameBalance / MobsBalance` в `src/loading/assets.rs`; соответствующие `.init_asset::<…>()` и `.register_asset_loader(…)` в `src/loading/mod.rs`.
- Поля `balance_handle`/`mobs_balance_handle` в `src/loading/systems.rs::LoadingState` и их загрузка/poll в `start_loading`/`check_loaded`.
- Структуры `MobsBalance`/`*Stats` остаются на месте в `src/actors/mobs/{mod,ghost,tower,slime,jumper,spinner}.rs` и переиспользуются модулем `balance/` как чистые типы — с `MobsBalance` снимаются `Asset`/`Resource`/`TypePath`-деривы (теперь это поле внутри `Balance`, а не отдельный Bevy-ресурс).

---

## Файловая структура проекта

```
assets/
  balance.xlsx              — SSOT баланса (бинарный, хранится в git)
src/
  balance/
    mod.rs                  — публичный API модуля, ре-экспорт
    types.rs                — структуры данных (MobBlock, WaveRow, GlobalRow, Globals, ...)
    parser.rs               — парсинг листов xlsx через calamine
    resolve.rs              — сборка MobsBalance (block → *Stats), Vec<WaveSegment>, Globals
    loader.rs               — загрузка (embedded / disk), валидация, F5-перезагрузка
    plugin.rs               — BalancePlugin, ресурс Balance, advance_wave_segment
```

`CurrentArenaSize` остаётся в `src/arena/size.rs` (рантайм-ресурс, не часть Balance).

---

## Порядок реализации

1. **Excel MCP-сервер.** Поставить (`haris-musa/excel-mcp-server` или аналог), прописать в `~/.claude.json`, проверить что появились тулы `mcp__excel__*`. Разовая настройка.
2. **Создание `assets/balance.xlsx`.** Claude Code через MCP читает текущие `assets/{mobs,balance}.ron` и создаёт файл с четырьмя листами (MobsSpecific блоками из мобов, MobsCommon с формулами, Waves с одним стартовым сегментом из `balance.ron::wave+arena`, Globals). Настраивает Data Validation и Conditional Formatting. Коммитим.
3. `types.rs` — `MobBlock`, `WaveRow`, `GlobalRow`, `WaveSegment`, `WaveSpawn`, `Globals`.
4. `parser.rs` — парсинг трёх листов через calamine (MobsSpecific блоками, Waves/Globals построчно по шапке, фильтрация `_`-ключей/колонок).
5. `resolve.rs` — сборка `MobsBalance` (block → per-mob `*Stats`), `Vec<WaveSegment>`, `Globals`.
6. `loader.rs` — загрузка (embedded / disk) + валидация + F5-перезагрузка.
7. `plugin.rs` — `BalancePlugin`, ресурс `Balance`, система `advance_wave_segment`.
8. **Миграция потребителей** (`src/wave/`, `src/arena/`, `src/run/`, `src/actors/mobs/`): `GameBalance`/`MobsBalance`-ресурсы → `Balance` + текущий сегмент.
9. **Удаление** `assets/balance.ron`, `assets/mobs.ron`, `src/balance.rs`, `RonAsset`-имплементаций и регистраций в `src/loading/`.
10. Unit-тесты парсинга и резолва на зафиксированном тестовом xlsx (в `src/balance/*.rs` через `#[cfg(test)]`; тестовый файл лежит в `src/balance/testdata/`).

---

## Известные TODO

- **Fireball → xlsx, когда появятся варианты.** Сейчас `FIREBALL_*` — `const` в `src/actors/player.rs`. Если появятся герои/билды с разным балансом `fireball`, вынести числа в отдельный лист/секцию с `variant_id` (конкретная схема не зафиксирована).
- **Веса спавна vs dev-меню.** Полное замещение пулов dev-оверрайдом — сознательный выбор; в будущем можно дать pooling-слайдер вместо бинарных тогглов.
- **Формулы MobsCommon.** Точная форма лукапа (через helper-колонку или чистым массивным `FILTER`) выбирается при первой настройке листа мигратором — это деталь UX, код на неё не завязан.
- **Auto-watch вместо F5.** Опционально подключить `notify` для авто-перезагрузки по сохранению файла, если F5 окажется неудобным.
- **Командная работа с xlsx.** Если появится второй редактор — договориться о монопольной правке или включить в xlsx ведение ревизий (Excel track-changes), либо переключиться на текстовый формат (CSV per lsheet) с потерей live-формул MobsCommon.
