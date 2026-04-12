# ТЗ: Система баланса для Magic Craft Bevy из Google Sheets

## Обзор

Система загрузки игрового баланса из Google Sheets. В dev-режиме (`--features dev`) данные загружаются из Google Sheets по сети с горячей перезагрузкой по F5. В сборке без `feature = "dev"` данные вшиваются в бинарник на этапе компиляции через `build.rs`.

Система работает **параллельно** существующему RON-пайплайну (`AssetLoader` / `LoadedFolder` для `.mob.ron`, `.ability.ron`, `.particle.ron`, `config.stats.ron` и пр.). Sheets отвечают только за числовой баланс; структурные данные (компоненты, FSM, визуалы, ability-дефиниции) остаются в RON.

---

## Разграничение: что в Sheets, что в RON

| Область | Источник | Комментарий |
|---------|----------|-------------|
| `base_stats` мобов (числа) | **Sheets** | Единственный источник. `base_stats` из `.mob.ron` удаляются при миграции. |
| Структура моба (components, FSM, visuals, abilities) | RON (`assets/mobs/*.mob.ron`) | Sheets не описывает компоненты. |
| Числовые тюнинги опциональных компонентов мобов | **Sheets** (Overrides) | Например `GhostTransparency.visible_distance`, `Size.value`. |
| Числовые поля ability-RON мобов | RON (пока) | **TODO**: в будущем расширить Overrides на `assets/mobs/abilities/*.ability.ron`. Механика ещё не определена. |
| Волны (сегменты рампа + пул мобов + рамп арены) | **Sheets** (Waves) | См. раздел Waves. |
| Глобальные параметры (кроме секций `wave`/`arena` из `balance.ron`) | **Sheets** (Globals) | См. раздел Globals. |
| Stats config (`config.stats.ron`) | RON | Не трогается. |
| Player ability RON | RON | Не трогается. |
| Player hero (`base.hero.ron`) | RON | Не трогается в рамках текущего скоупа. |

---

## Структура Google Sheets

Один документ Google Sheets содержит несколько листов. Документ должен быть опубликован (Share → Anyone with the link). Каждый лист экспортируется как CSV по URL:

```
https://docs.google.com/spreadsheets/d/{DOC_ID}/export?format=csv&gid={GID}
```

`DOC_ID` — идентификатор документа из URL таблицы. `GID` — идентификатор листа, виден в URL при переключении листов (`#gid=...`).

### Лист "Enemies"

Числовые `base_stats` мобов. Один ряд — один моб. `id` совпадает с top-level `id` из `assets/mobs/{id}.mob.ron`.

Колонки:

| Колонка | Тип | Описание |
|---------|-----|----------|
| id | string | Идентификатор моба, совпадает с `id:` в `.mob.ron`. |
| _name | string | **Отладочная.** Отображаемое имя для дизайнера. |
| _notes | string | **Отладочная.** Заметки. |

Остальные колонки — **любые имена статов из `assets/stats/config.stats.ron`**: `max_life_flat`, `physical_damage_flat`, `movement_speed_flat`, `crit_chance_flat` и т.д. Набор колонок расширяемый.

**Пустая ячейка** = «значение по умолчанию»: стат не добавляется в `base_stats`, моб получит дефолт из стат-системы. **Пустая ячейка ≠ 0.** Чтобы явно задать 0, написать `0`.

**Важно:** `base_stats: { ... }` в `.mob.ron` удаляется при миграции. Sheets — единственный источник `base_stats`. Если моб присутствует в Enemies, но `base_stats` пуст — моб запускается с одними дефолтами (валидным сценарием это не считается, но и не падает).

Отладочные колонки (имя начинается с `_`) игнорируются при парсинге.

### Лист "Overrides"

Числовые оверрайды для опциональных числовых полей компонентов мобов (из `.mob.ron`). Пример: `GhostTransparency.visible_distance`, `Size.value`, `RandomJump.distance`.

**Скоуп сейчас — только мобы.** Числовые поля в `assets/mobs/abilities/*.ability.ron` (например `MeleeStrike.range`, `MeleeStrike.damage`) Overrides пока **не покрывает**. В будущем лист будет расширен (см. TODO в разделе «Разграничение»).

Один ряд — одно поле одного моба.

| Колонка | Тип | Описание |
|---------|-----|----------|
| enemy_id | string (dropdown) | Ссылка на `id` из Enemies. Dropdown from range `Enemies!A2:A`. |
| property | string (dropdown) | Формат `ComponentName.field_name`. Dropdown из Properties. |
| value | string | Числовое значение. Тип (float/int) диктуется Properties. |

Оверрайд замещает значение в `.mob.ron` (для поля типа `Option<ScalarExpr>` — вставляется как литерал; для поля без `Option` — перекрывает дефолтный). Структурные свойства (наличие компонентов, списки, вложения) не оверрайдятся.

### Лист "Properties" (справочник)

Реестр допустимых полей для Overrides. Источник для dropdown-валидации.

| Колонка | Тип | Описание |
|---------|-----|----------|
| id | string | Формат `ComponentName.field_name`. |
| type | string | `float`, `int`. |
| description | string | Описание для дизайнера. |

### Лист "Waves"

Сегменты боя. Каждый сегмент задаёт параметры рампа врагов, пул мобов и параметры рампа арены. Сегменты идут последовательно, переключаются по истечении `duration_sec`.

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
- Пул сегмента активен всё время сегмента; спавн выбирает моба случайно по весам.
- Арена плавно интерполируется от `arena_start_*` к `arena_end_*` внутри сегмента (линейно). При переходе на следующий сегмент арена начинается с `arena_start_*` следующего сегмента (возможен скачок, если дизайнер не выровняет значения).
- `delay_sec` (интервал между спавнами отдельных мобов) не используется — темп спавна задаётся только рампом `start_enemies → max_enemies` и лимитом одновременных врагов.

**Взаимодействие с dev-меню.** При `--features dev` есть меню `EnemySpawnPool` с toggles на каждый тип моба. Если dev-меню активно использовалось (пользователь менял тогглы), оно **полностью замещает** пул из текущего сегмента Waves — сегмент в этот момент диктует только параметры рампа и арены, а выбор мобов идёт из dev-оверрайда. Как именно определять «активно использовалось» — флаг `EnemySpawnPool.dev_override: bool`, взводится первым кликом в меню.

### Лист "Globals"

Глобальные настройки, не привязанные к волнам/аренам. Вертикальный формат ключ-значение.

Из `assets/balance.ron` сюда переезжают все поля **кроме** тех, что стали per-segment: `start_enemies`, `max_enemies`, `ramp_duration_secs`, `start_width`, `start_height`, `width`, `height` ушли в Waves. Остаются:

- `safe_spawn_radius` (секция `wave`)
- `shop_delay` (секция `wave`)
- `coins_per_kill` (секция `run`)
- `hp_scale_per_sec`, `dmg_scale_per_sec` (секция `run`)
- `coin_attraction_duration` (секция `run`)

Поле `node_cost` из `run` удаляется (skill tree уже убран, поле мёртвое).

Колонки:

| Колонка | Тип | Описание |
|---------|-----|----------|
| key | string | Уникальный ключ параметра. |
| value | string | Значение (строка, тип диктуется кодом-потребителем). |
| _description | string | **Отладочная.** Описание для дизайнера. |

---

## Валидация в Google Sheets

### Dropdown-валидация

- `enemy_id` на Waves и Overrides: Data Validation → Dropdown from range → `Enemies!A2:A`, режим "Reject input".
- `property` на Overrides: Dropdown from range → `Properties!A2:A`, режим "Reject input".
- `kind` на Waves: Dropdown from list → `segment,pool`.

### Подсветка битых ссылок

На Waves и Overrides условное форматирование (красный фон) на `enemy_id`:

```
=ISNA(MATCH(B2, Enemies!$A$2:$A, 0))
```

### Рекомендация по удалению мобов

Вместо удаления строки из Enemies добавить колонку `deprecated` (bool). Моб с `deprecated = TRUE` фильтруется при экспорте, но остаётся в dropdown-ах.

---

## Реестр листов в коде

```rust
const DOC_ID: &str = "..."; // ID таблицы

struct SheetMeta {
    name: &'static str,
    gid: u32,
}

const SHEETS: &[SheetMeta] = &[
    SheetMeta { name: "enemies",    gid: 0 },
    SheetMeta { name: "overrides",  gid: XXXXXXXXX },
    SheetMeta { name: "properties", gid: XXXXXXXXX },
    SheetMeta { name: "waves",      gid: XXXXXXXXX },
    SheetMeta { name: "globals",    gid: XXXXXXXXX },
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
    pub const ENEMIES:    &str = include_str!(concat!(env!("OUT_DIR"), "/enemies.csv"));
    pub const OVERRIDES:  &str = include_str!(concat!(env!("OUT_DIR"), "/overrides.csv"));
    pub const PROPERTIES: &str = include_str!(concat!(env!("OUT_DIR"), "/properties.csv"));
    pub const WAVES:      &str = include_str!(concat!(env!("OUT_DIR"), "/waves.csv"));
    pub const GLOBALS:    &str = include_str!(concat!(env!("OUT_DIR"), "/globals.csv"));
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
// Одна строка листа Enemies. stats — произвольный набор колонок.
#[derive(Debug, Clone)]
struct EnemyRow {
    id: String,
    /// Ключ = имя стата из config.stats.ron.
    /// Присутствует только для непустых ячеек.
    /// Отсутствие ключа = дефолт стата (ячейка была пустой).
    stats: HashMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct OverrideRow {
    enemy_id: String,
    property: String, // "ComponentName.field_name"
    value: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PropertyDef {
    id: String,
    r#type: String, // "float", "int"
    description: String,
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
struct EnemyBalance {
    id: String,
    stats: HashMap<String, f32>,           // base_stats
    overrides: HashMap<String, String>,    // "Component.field" -> raw value
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

1. **EnemyBalance.** Для каждой строки Enemies — `stats` из непустых ячеек; оверрайды из Overrides по `enemy_id` → `HashMap<String, String>` (ключ = `property`).
2. **Сегменты волн.** Группировка WaveRow по полю `wave`. Из строки `kind=segment` — параметры сегмента и арены; из строк `kind=pool` — пул.
3. **Globals.** Лист Globals → `HashMap<String, String>`.

---

## Валидация в коде (runtime)

Вся балансная валидация живёт в `src/balance/loader.rs` и исполняется при загрузке и F5-перезагрузке. Существующий `src/validation_tests.rs` (`#[test]`-юнит-тесты на RON-ассеты) не расширяется.

### Проверки

1. **Уникальность id мобов** в Enemies.
2. **Ссылки Overrides → Enemies.**
3. **Ссылки Waves → Enemies.**
4. **Соответствие колонок Enemies → config.stats.ron.** Каждая нефлаговая колонка (не `_`, не `id`) — зарегистрированный стат.
5. **Валидность property в Overrides.** `property` существует в Properties; `value` парсится в указанный тип.
6. **Дубликаты оверрайдов.** Пара (`enemy_id`, `property`) не встречается дважды.
7. **Структура Waves.** Для каждого `wave` — ровно одна строка `kind=segment` с заполненными `duration_sec`/`start_enemies`/`max_enemies`/`ramp_duration_sec` и четырьмя `arena_*`; ≥1 строка `kind=pool` с непустыми `enemy_id`/`weight > 0`. Номера сегментов идут без разрывов, начиная с 1.
8. **Фильтрация deprecated.** Мобы с `deprecated = TRUE` исключаются; ссылки на них в Waves/Overrides — предупреждение.
9. **Пересечение с RON.** Каждый `id` из Enemies имеет соответствующий `.mob.ron` в `assets/mobs/`; каждый моб в `assets/mobs/` без записи в Enemies — предупреждение (моб стартует с дефолтными статами).

### Поведение при ошибках

- **`--features dev`**: panic с полным списком ошибок на старте; F5-перезагрузка при ошибке оставляет прежний `Balance` и логирует `error!`.
- **Release без dev**: `error!()`, пропуск невалидных записей, игра не крашится.

---

## Bevy-интеграция

### Ресурс

```rust
#[derive(Resource)]
struct Balance {
    enemies: HashMap<String, EnemyBalance>,
    waves: Vec<WaveSegment>,
    globals: HashMap<String, String>,
}
```

### Потребление

Текущие потребители `GameBalance` переключаются на `Balance`:

- **`src/wave.rs`** (`WaveState::new`, `reset_wave_state`): берёт `shop_delay` из Globals; `start_enemies/max_enemies/ramp_duration` теперь диктуются текущим `WaveSegment`.
- **`src/arena.rs`** (`reset_arena_size`, `update_arena_size`, `update_target_count`, `spawn_enemies`): `arena_start/arena_end`, `start_enemies`, `max_enemies`, `ramp_duration_sec` берутся из текущего сегмента; `safe_spawn_radius`, `hp_scale_per_sec`, `dmg_scale_per_sec` — из Globals.
- **`src/coin.rs`**: `coins_per_kill`, `coin_attraction_duration` — из Globals.
- **`src/blueprints/components/mob/random_jump.rs`**: использует только `CurrentArenaSize` — остаётся как есть (рантайм-ресурс пересчитывается из текущего сегмента).

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

- `assets/balance.ron`.
- `src/balance.rs` (`GameBalance`, `WaveBalance`, `ArenaBalance`, `RunBalance`) — замещается модулем `src/balance/`. `CurrentArenaSize` мигрирует в `balance/plugin.rs` как рантайм-ресурс.
- `GameBalanceAsset` и `GameBalanceLoader` в `src/loading/assets.rs`; соответствующие `.init_asset::<GameBalanceAsset>()` и `.register_asset_loader(GameBalanceLoader)` в `src/loading/mod.rs`.
- `LoadingState.balance_handle` и блок его ожидания в `src/loading/systems.rs::check_stats_loaded`.
- Поле `node_cost` в `run` секции (не переносится в Globals).
- `base_stats: { ... }` из всех `.mob.ron` (статы теперь только в Sheets).

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
    types.rs        — структуры данных (EnemyRow, OverrideRow, WaveRow, ...)
    resolve.rs      — сборка EnemyBalance, WaveSegment, Globals
    loader.rs       — загрузка (embedded / network), валидация, F5-перезагрузка
    plugin.rs       — BalancePlugin, ресурс Balance, CurrentArenaSize, advance_wave_segment
build.rs            — скачивание CSV для release-билда
```

---

## Порядок реализации

1. **Подготовка Google Sheets.** Создать документ с листами Enemies / Overrides / Properties / Waves / Globals. Перенести `base_stats` из всех `.mob.ron` в Enemies. Перенести `balance.ron` в Waves (per-segment) + Globals (остальное).
2. `types.rs` — все структуры данных.
3. `sheets.rs` — `DOC_ID`, `GID`, формирование URL.
4. `parser.rs` — парсинг CSV с фильтрацией `_`.
5. `resolve.rs` — сборка EnemyBalance, WaveSegment, Globals.
6. `loader.rs` — загрузка + валидация + F5-перезагрузка.
7. `plugin.rs` — `BalancePlugin`, ресурс `Balance`, `CurrentArenaSize`, система `advance_wave_segment`.
8. **Миграция потребителей** (`wave.rs`, `arena.rs`, `coin.rs`): `GameBalance` → `Balance` + текущий сегмент.
9. **Удаление** `assets/balance.ron`, `src/balance.rs`, `GameBalanceLoader`/`GameBalanceAsset`, `base_stats:` из всех `.mob.ron`.
10. `build.rs` — скачивание CSV при release-билде.
11. Unit-тесты парсинга и резолва на захардкоженных CSV-строках (в `src/balance/*.rs` через `#[cfg(test)]`).

---

## Известные TODO

- **Оверрайды для числовых полей ability-RON.** Сейчас числа в `assets/mobs/abilities/*.ability.ron` (например `MeleeStrike.range`, `MeleeStrike.damage`) тюнятся только через RON. В будущей итерации расширить Overrides (отдельный лист или единый `target_id` + `kind`), чтобы дизайнер мог балансить и абилки мобов через Sheets. Конкретный механизм не зафиксирован.
