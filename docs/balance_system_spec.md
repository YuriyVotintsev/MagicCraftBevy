# ТЗ: Система баланса для Magic Craft Bevy из Google Sheets

## Обзор

Система загрузки игрового баланса из Google Sheets. В dev-режиме (`--features dev`) данные загружаются из Google Sheets по сети с горячей перезагрузкой по F5. В сборке без `feature = "dev"` данные вшиваются в бинарник на этапе компиляции через `build.rs`.

Система заменяет текущий RON-пайплайн балансных файлов (`assets/balance.ron`, `assets/mobs.ron`, `assets/abilities.ron`), загружаемых через `RonAssetLoader` в `src/loading/`. Структурные ассеты (`assets/particles/*.particle.ron`, `assets/palette.ron`) остаются в RON и продолжают загружаться через `AssetLoader`/`LoadedFolder`.

---

## Контекст архитектуры (важно)

После удаления data-driven blueprint-системы и FSM (коммиты `9034dac`, `0a85951`, `ed03379`) в RON остались **только числа**. Структура мобов (компоненты, AI, визуалы, коллайдеры), стат-система (`enum Stat` в `src/stats/registry.rs`) и абилки (модули в `src/actors/components/ability/` и `src/actors/tower_shot.rs`) — хардкод в Rust под `src/actors/`.

Соответственно Sheets покрывает **весь текущий числовой баланс целиком**, а не параллельный RON-поток. После миграции `balance.ron`/`mobs.ron`/`abilities.ron` удаляются, `GameBalance`/`MobsBalance`/`AbilitiesBalance` как `RonAsset` — тоже.

### Что в Sheets, что в RON

| Область | Источник | Комментарий |
|---------|----------|-------------|
| Числовой баланс мобов (`assets/mobs.ron` целиком) | **Sheets** (Enemies) | Типизированные per-mob структуры `GhostStats`/`TowerStats`/`SlimeSmallStats`/`JumperStats`/`SpinnerStats` в `src/actors/mobs.rs`. |
| Числовой баланс абилок (`assets/abilities.ron` целиком) | **Sheets** (Abilities) | Типизированные per-ability структуры в `src/actors/abilities.rs`. |
| Структура мобов (компоненты, AI, визуалы) | Rust (`src/actors/`) | Sheets не описывает. Изменение — правка кода. |
| Стат-система (`enum Stat`, калькуляторы) | Rust (`src/stats/`) | Хардкод. RON-конфига статов больше нет. |
| Волны (сегменты рампа + пул мобов + рамп арены) | **Sheets** (Waves) | См. раздел Waves. На момент миграции код использует единый рамп (`balance.ron::wave`/`arena`); переход на per-segment — часть миграции. |
| Глобальные параметры (кроме полей из `wave`/`arena`, ставших per-segment) | **Sheets** (Globals) | См. раздел Globals. |
| Визуалы частиц (`assets/particles/*.particle.ron`) | RON | Не трогается. |
| Палитра (`assets/palette.ron`) | RON | Не трогается. |

---

## Структура Google Sheets

Один документ Google Sheets содержит несколько листов. Документ должен быть опубликован (Share → Anyone with the link). Каждый лист экспортируется как CSV по URL:

```
https://docs.google.com/spreadsheets/d/{DOC_ID}/export?format=csv&gid={GID}
```

`DOC_ID` — идентификатор документа из URL таблицы. `GID` — идентификатор листа, виден в URL при переключении листов (`#gid=...`).

### Лист "Enemies"

Числовой баланс мобов. Один ряд — один моб. `id` совпадает с ключом в текущем `assets/mobs.ron` и с `MobKind::id()` в `src/actors/mobs.rs` (`ghost`, `tower`, `slime_small`, `jumper`, `spinner`).

Колонки — **объединение всех числовых полей per-mob структур** в `src/actors/mobs.rs`. Неприменимые ячейки остаются пустыми (например, у `tower` не заполняется `speed`/`mass`/`melee_range`).

| Колонка | Тип | Применимо к |
|---------|-----|-------------|
| id | string | все |
| _name | string (debug) | все |
| _notes | string (debug) | все |
| hp | float | все |
| damage | float | все |
| speed | float | ghost, slime_small, jumper |
| size | float | все |
| mass | float | ghost, slime_small, jumper, spinner |
| melee_range | float | ghost, slime_small |
| melee_cooldown | float | ghost, slime_small |
| visible_distance | float | ghost |
| invisible_distance | float | ghost |
| shot_cooldown | float | tower |
| lunge_duration | float | slime_small |
| idle_duration | float | jumper, spinner |
| jump_duration | float | jumper |
| land_duration | float | jumper |
| jump_distance | float | jumper |
| spike_length | float | spinner |
| windup_duration | float | spinner |
| charge_duration | float | spinner |
| cooldown_duration | float | spinner |
| charge_speed | float | spinner |

**Пустая ячейка** для применимого поля = валидационная ошибка (поле обязательно в структуре). Пустая ячейка для неприменимого поля — ожидаема, игнорируется.

Отладочные колонки (имя начинается с `_`) игнорируются при парсинге.

При добавлении нового моба (новой `*Stats`-структуры в `src/actors/mobs.rs`) в лист добавляются только недостающие колонки; существующие мобы оставляют их пустыми.

### Лист "Abilities"

Числовой баланс абилок. Один ряд — одна абилка. `id` совпадает с ключом в `assets/abilities.ron` и полем `AbilitiesBalance` (`melee_attack`, `jumper_shot`, `tower_shot`, `fireball`).

Колонки — объединение числовых полей per-ability структур в `src/actors/abilities.rs`.

| Колонка | Тип | Применимо к |
|---------|-----|-------------|
| id | string | все |
| _name | string (debug) | все |
| _notes | string (debug) | все |
| range | float | melee_attack |
| projectile_count | int | jumper_shot |
| projectile_speed | float | jumper_shot |
| projectile_size | float | jumper_shot, tower_shot |
| projectile_lifetime | float | jumper_shot |
| spread_degrees | float | jumper_shot |
| flight_duration | float | tower_shot |
| arc_height | float | tower_shot |
| start_elevation | float | tower_shot |
| spread | float | tower_shot |
| explosion_radius | float | tower_shot |
| explosion_duration | float | tower_shot |
| indicator_duration | float | tower_shot |
| base_damage | float | fireball |
| base_speed | float | fireball |
| cooldown | float | fireball |
| size | float | fireball |
| gap | float | fireball |

Правила валидации — те же, что у Enemies: обязательное поле-ячейка пустой быть не может.

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
| enemy_id | string (dropdown) | (pool) ссылка на `id` из Enemies. |
| weight | float | (pool) вес моба в пуле сегмента. |
| _enemy_name | string (формула) | **Отладочная.** `=VLOOKUP(enemy_id, Enemies!A:B, 2, FALSE)`. |

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

## Валидация в Google Sheets

### Dropdown-валидация

- `enemy_id` на Waves: Data Validation → Dropdown from range → `Enemies!A2:A`, режим "Reject input".
- `kind` на Waves: Dropdown from list → `segment,pool`.
- `id` на Abilities — свободный ввод с подсветкой ошибок (см. ниже).

### Подсветка битых ссылок

На Waves условное форматирование (красный фон) на `enemy_id`:

```
=ISNA(MATCH(B2, Enemies!$A$2:$A, 0))
```

### Рекомендация по удалению мобов/абилок

Вместо удаления строки из Enemies/Abilities добавить колонку `deprecated` (bool). Запись с `deprecated = TRUE` фильтруется при экспорте, но остаётся в dropdown-ах.

---

## Реестр листов в коде

```rust
const DOC_ID: &str = "..."; // ID таблицы

struct SheetMeta {
    name: &'static str,
    gid: u32,
}

const SHEETS: &[SheetMeta] = &[
    SheetMeta { name: "enemies",   gid: 0 },
    SheetMeta { name: "abilities", gid: XXXXXXXXX },
    SheetMeta { name: "waves",     gid: XXXXXXXXX },
    SheetMeta { name: "globals",   gid: XXXXXXXXX },
];
```

---

## Сборка: `build.rs`

### Release-режим (без `feature = "dev"`)

`build.rs`:

1. Итерирует по `SHEETS`.
2. Скачивает CSV по URL.
3. Сохраняет в `OUT_DIR` как `{name}.csv`.

Зависимость: `ureq` в `[build-dependencies]`.

### Dev-режим (`--features dev`)

`build.rs` ничего не скачивает. Данные загружаются в рантайме по сети.

### Встраивание в бинарник

```rust
#[cfg(not(feature = "dev"))]
mod embedded {
    pub const ENEMIES:   &str = include_str!(concat!(env!("OUT_DIR"), "/enemies.csv"));
    pub const ABILITIES: &str = include_str!(concat!(env!("OUT_DIR"), "/abilities.csv"));
    pub const WAVES:     &str = include_str!(concat!(env!("OUT_DIR"), "/waves.csv"));
    pub const GLOBALS:   &str = include_str!(concat!(env!("OUT_DIR"), "/globals.csv"));
}
```

Гейтинг по `feature = "dev"` (не `debug_assertions`) — симметрично dev-рантайму.

---

## Парсинг CSV

### Фильтрация отладочных колонок

Любая колонка, заголовок которой начинается с `_`, удаляется из CSV перед десериализацией.

Алгоритм:
1. Прочитать заголовки.
2. Оставить индексы колонок, чей заголовок НЕ начинается с `_`.
3. Пересобрать CSV только из этих колонок.
4. Десериализовать.

### Структуры данных

```rust
// Одна строка листа Enemies. Значения приходят как Option<f32> — неприменимые
// поля пусты. После резолва мапятся в конкретную *Stats-структуру по id.
#[derive(Debug, Clone, Deserialize)]
struct EnemyRow {
    id: String,
    hp: Option<f32>,
    damage: Option<f32>,
    speed: Option<f32>,
    size: Option<f32>,
    mass: Option<f32>,
    melee_range: Option<f32>,
    melee_cooldown: Option<f32>,
    visible_distance: Option<f32>,
    invisible_distance: Option<f32>,
    shot_cooldown: Option<f32>,
    lunge_duration: Option<f32>,
    idle_duration: Option<f32>,
    jump_duration: Option<f32>,
    land_duration: Option<f32>,
    jump_distance: Option<f32>,
    spike_length: Option<f32>,
    windup_duration: Option<f32>,
    charge_duration: Option<f32>,
    cooldown_duration: Option<f32>,
    charge_speed: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct AbilityRow {
    id: String,
    range: Option<f32>,
    projectile_count: Option<u32>,
    projectile_speed: Option<f32>,
    projectile_size: Option<f32>,
    projectile_lifetime: Option<f32>,
    spread_degrees: Option<f32>,
    flight_duration: Option<f32>,
    arc_height: Option<f32>,
    start_elevation: Option<f32>,
    spread: Option<f32>,
    explosion_radius: Option<f32>,
    explosion_duration: Option<f32>,
    indicator_duration: Option<f32>,
    base_damage: Option<f32>,
    base_speed: Option<f32>,
    cooldown: Option<f32>,
    size: Option<f32>,
    gap: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

---

## Сборка данных (resolve)

1. **MobsBalance.** По `EnemyRow.id` матчинг на `MobKind` (`ghost`/`tower`/`slime_small`/`jumper`/`spinner`). Заполнение соответствующей `*Stats`-структуры из колонок; недостающая обязательная колонка — ошибка валидации.
2. **AbilitiesBalance.** По `AbilityRow.id` матчинг на поле в `AbilitiesBalance` (`melee_attack`/`jumper_shot`/`tower_shot`/`fireball`); аналогично заполняем per-ability структуру.
3. **Сегменты волн.** Группировка `WaveRow` по полю `wave`. Из строки `kind=segment` — параметры сегмента и арены; из строк `kind=pool` — пул.
4. **Globals.** Лист Globals → `HashMap<String, String>`.

---

## Валидация в коде (runtime)

Вся балансная валидация живёт в `src/balance/loader.rs` и исполняется при загрузке и F5-перезагрузке.

### Проверки

1. **Уникальность id мобов** в Enemies.
2. **id из Enemies — валидный `MobKind`.** Незарегистрированный id — ошибка. Непокрытый `MobKind` в Enemies — тоже ошибка (у игры есть моб без статов).
3. **Заполненность полей Enemies** по маске применимости для каждого `MobKind`. Пустая ячейка в обязательном поле — ошибка; непустая ячейка в неприменимом поле — предупреждение.
4. **Уникальность id абилок** в Abilities и такая же маска применимости по полям `AbilitiesBalance`.
5. **Ссылки Waves → Enemies.** `enemy_id` существует в Enemies и не помечен `deprecated`.
6. **Структура Waves.** Для каждого `wave` — ровно одна строка `kind=segment` с заполненными `duration_sec`/`start_enemies`/`max_enemies`/`ramp_duration_sec` и четырьмя `arena_*`; ≥1 строка `kind=pool` с непустыми `enemy_id`/`weight > 0`. Номера сегментов идут без разрывов, начиная с 1.
7. **Globals.** Все ожидаемые ключи (`safe_spawn_radius`, `shop_delay`, `coins_per_kill`, `hp_scale_per_sec`, `dmg_scale_per_sec`, `coin_attraction_duration`) присутствуют и парсятся в ожидаемые типы.
8. **Фильтрация deprecated.** Записи с `deprecated = TRUE` исключаются; ссылки на них в Waves — предупреждение.

### Поведение при ошибках

- **`--features dev`**: panic с полным списком ошибок на старте; F5-перезагрузка при ошибке оставляет прежний `Balance` и логирует `error!`.
- **Release без dev**: `error!()`, пропуск невалидных записей, игра не крашится.

---

## Bevy-интеграция

### Ресурс

```rust
#[derive(Resource)]
struct Balance {
    mobs: MobsBalance,          // переиспользуем типы из src/actors/mobs.rs
    abilities: AbilitiesBalance, // переиспользуем типы из src/actors/abilities.rs
    waves: Vec<WaveSegment>,
    globals: Globals,            // типизированная структура с полями из листа Globals
}
```

### Потребление

Текущие потребители переключаются на `Balance` (и на текущий сегмент волн):

- **`src/wave/mod.rs`** (`WaveState::new`, `reset_wave_state`): `shop_delay` из Globals; `start_enemies`/`max_enemies`/`ramp_duration` теперь диктуются текущим `WaveSegment`.
- **`src/wave/spawn.rs`** (`reset_arena_size`, `update_arena_size`, `update_target_count`, `spawn_enemies`): `arena_start/arena_end`, `start_enemies`, `max_enemies`, `ramp_duration_sec` берутся из текущего сегмента; `safe_spawn_radius`, `hp_scale_per_sec`, `dmg_scale_per_sec` — из Globals. `MobsBalance` берётся из `Balance.mobs` вместо отдельного ресурса.
- **`src/run/coin.rs`**, **`src/run/money.rs`**: `coins_per_kill`, `coin_attraction_duration` — из Globals.
- **`src/arena/mod.rs`** (`spawn_arena`, `update_walls`, `update_floor_mesh`): читает `CurrentArenaSize` (рантайм-ресурс, пересчитывается из текущего сегмента в `src/wave/spawn.rs::update_arena_size`). Первоначальный размер арены при `MainMenu` тоже берётся из первого сегмента.
- **`src/actors/mobs.rs::spawn_mob`** и потребители `AbilitiesBalance` (спеллы игрока, `tower_shot`, `jumper_shot`, `melee_strike`) — через `&Balance.abilities`.

Добавляется новая система-оркестратор сегментов: `advance_wave_segment` тикает `run_state.elapsed_in_segment`, переключает `current_segment_index` по истечении `duration_sec`, обновляет `WaveState.max_concurrent`, `CurrentArenaSize` и активный пул.

### Загрузка

Загрузка идёт **вне Bevy `AssetServer`** и не использует `AssetLoader`/`LoadedFolder`. Это отдельный путь, параллельный `src/loading/`.

Система `setup_balance` на `Startup`:

- **Release** (без `feature = "dev"`): парсит `embedded::*`.
- **Dev** (`--features dev`): скачивает CSV через `ureq`.

### Горячая перезагрузка (только `--features dev`)

Система `reload_balance` в `Update`:

- Отслеживает `F5`.
- Перекачивает CSV, парсит, валидирует.
- При успехе — заменяет `Balance`, логирует `info!("Balance reloaded")`.
- При ошибке — `error!(...)`, старый `Balance` сохраняется.
- **Скачанные CSV не пишутся в `assets/`** — живут только в памяти.

Гейтинг: `#[cfg(feature = "dev")]`.

### Регистрация в App

```rust
app.add_systems(Startup, setup_balance);

#[cfg(feature = "dev")]
app.add_systems(Update, reload_balance);
```

### Удаляется из кодовой базы при миграции

- `assets/balance.ron`, `assets/mobs.ron`, `assets/abilities.ron`.
- `src/balance.rs` (`GameBalance`, `WaveBalance`, `ArenaBalance`, `RunBalance`) — замещается модулем `src/balance/`.
- Реализации `RonAsset for GameBalance / MobsBalance / AbilitiesBalance` в `src/loading/assets.rs`; соответствующие `.init_asset::<…>()` и `.register_asset_loader(…)` в `src/loading/mod.rs`.
- Поля `balance_handle`/`mobs_balance_handle`/`abilities_balance_handle` в `src/loading/systems.rs::LoadingState` и их загрузка/poll в `start_loading`/`check_loaded`.
- Структуры `MobsBalance`/`*Stats` и `AbilitiesBalance`/per-ability структуры либо остаются на месте в `src/actors/mobs.rs` и `src/actors/abilities.rs` и переиспользуются модулем `balance/` как чистые типы (без `#[derive(Asset)]`), либо переезжают в `src/balance/types.rs` — по вкусу. Рекомендуется оставить на месте и снять `Asset`/`RonAsset`.

---

## Зависимости (Cargo.toml)

```toml
[dependencies]
# существующие (bevy 0.18, serde, ron, ...)
csv = "1"

# Только dev — сетевая загрузка в рантайме
[target.'cfg(feature = "dev")'.dependencies]
ureq = "2"

[build-dependencies]
ureq = "2"
```

---

## Файловая структура проекта

```
src/
  balance/
    mod.rs          — публичный API модуля, ре-экспорт
    sheets.rs       — реестр листов, URL, константы DOC_ID и GID
    parser.rs       — парсинг CSV, фильтрация _-колонок
    types.rs        — структуры данных (EnemyRow, AbilityRow, WaveRow, Globals, ...)
    resolve.rs      — сборка MobsBalance, AbilitiesBalance, WaveSegment, Globals
    loader.rs       — загрузка (embedded / network), валидация, F5-перезагрузка
    plugin.rs       — BalancePlugin, ресурс Balance, advance_wave_segment
build.rs            — скачивание CSV для release-билда
```

`CurrentArenaSize` остаётся в `src/arena/size.rs` (рантайм-ресурс, не часть Balance).

---

## Порядок реализации

1. **Подготовка Google Sheets.** Создать документ с листами Enemies / Abilities / Waves / Globals. Перенести текущий `assets/mobs.ron` в Enemies, `assets/abilities.ron` в Abilities, `assets/balance.ron::wave/arena` в Waves (один стартовый сегмент), `assets/balance.ron::run` + `wave.safe_spawn_radius`/`wave.shop_delay` в Globals.
2. `types.rs` — все структуры данных.
3. `sheets.rs` — `DOC_ID`, `GID`, формирование URL.
4. `parser.rs` — парсинг CSV с фильтрацией `_`.
5. `resolve.rs` — сборка `MobsBalance`, `AbilitiesBalance`, `Vec<WaveSegment>`, `Globals`.
6. `loader.rs` — загрузка + валидация + F5-перезагрузка.
7. `plugin.rs` — `BalancePlugin`, ресурс `Balance`, система `advance_wave_segment`.
8. **Миграция потребителей** (`src/wave/`, `src/arena/`, `src/run/`, `src/actors/mobs.rs`, `src/actors/abilities.rs` и производные): `GameBalance`/`MobsBalance`/`AbilitiesBalance`-ресурсы → `Balance` + текущий сегмент.
9. **Удаление** `assets/balance.ron`, `assets/mobs.ron`, `assets/abilities.ron`, `src/balance.rs`, `RonAsset`-имплементаций и регистраций в `src/loading/`.
10. `build.rs` — скачивание CSV при release-билде.
11. Unit-тесты парсинга и резолва на захардкоженных CSV-строках (в `src/balance/*.rs` через `#[cfg(test)]`).

---

## Известные TODO

- **Per-ability оверрайды или ветвление.** Если появится потребность в разных балансах одной абилки (например, `fireball` для разных геров/билдов), Sheets-схема `Abilities` расширится id-ветвлением (`fireball.hero_a`, `fireball.hero_b`) или отдельным листом `AbilityVariants`. Конкретный механизм не зафиксирован.
- **Веса спавна vs dev-меню.** Полное замещение пулов dev-оверрайдом — сознательный выбор; в будущем можно дать pooling-слайдер вместо бинарных тогглов.
