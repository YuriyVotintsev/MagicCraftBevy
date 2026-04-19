# ТЗ: Система баланса для Magic Craft Bevy из xlsx

## Обзор

Система загрузки игрового баланса из локального файла `assets/balance.xlsx`. В dev-режиме (`--features dev`) файл читается с диска при старте и перечитывается по F5 (горячая перезагрузка). В сборке без `feature = "dev"` файл вшивается в бинарник через `include_bytes!`.

Система заменяет текущий RON-пайплайн балансных файлов (`assets/balance.ron`, `assets/mobs.ron`, `assets/runes.ron`, `assets/waves.ron`), загружаемых через `RonAssetLoader` в `src/loading/`. Структурные ассеты (`assets/particles/*.particle.ron`, `assets/palette.ron`) остаются в RON и продолжают загружаться через `AssetLoader`/`LoadedFolder`.

Редактирование баланса — в Excel или LibreOffice Calc. Claude Code при необходимости правит тот же файл через Excel MCP-сервер (см. раздел «Инструментарий»).

---

## Контекст архитектуры

После удаления data-driven blueprint-системы, FSM, DSL выражений и генерик-абилок (коммиты `9034dac`, `0a85951`, `ed03379`, `7586174`, `c1d36c6`), а затем после коллапса continuous-scaling-моделей (коммит `1d57db6` — «remove arena expansion and enemy scaling») в RON-балансе остались только **общие числа**. Структура мобов (компоненты, AI, визуалы, коллайдеры), их уникальные per-mob тайминги/радиусы абилок, стат-система (`enum Stat` в `src/stats/registry.rs`) — хардкод в Rust.

Соответственно xlsx покрывает **только общие, пригодные для кросс-сравнения числа**, а не полный per-mob профиль. Основная ценность файла — горизонтальная таблица мобов, где hp/damage/speed/size/mass всех мобов видно одновременно и тюнится бок-о-бок; плюс прогрессия волн, глобалы и цены рун.

### Что в xlsx, что в Rust

| Область | Источник | Комментарий |
|---------|----------|-------------|
| Общие балансные числа мобов (hp, damage, speed, size, mass) | **xlsx** (лист Mobs) | Горизонтальная таблица, одна строка на моба. Только параметры, встречающиеся у ≥2 мобов и имеющие смысл для сравнения. |
| Уникальные per-mob параметры поведения (тайминги абилок, радиусы, проджектайл-параметры, AI-фазы) | Rust (`const` в `src/actors/mobs/<mob>.rs`) | Например, `TowerShooter.flight_duration`, `SpinnerStats.charge_speed`, `JumperStats.jump_distance`. Уникально → код. Попало в ≥2 моба и захотелось тюнить рядом → поднимается в Mobs. |
| Визуальные числа моба (цвета, формы, высоты y-дуги снаряда, длины шипов, fade-дистанции, warning-индикаторы, колыхание/подскок) | Rust (`const`) | Визуал жёстко хардкожен. В xlsx не едет ни при каких условиях. |
| Числа абилки игрока (fireball) | Rust (`const` в `src/actors/player.rs`) | `FIREBALL_*`. Игрок один, варианты не нужны; если появятся — отдельный разговор. |
| Прогрессия волн (анлоки мобов + per-wave разнообразие/лимит/множители) | **xlsx** (лист Waves) | См. раздел Waves. Модель дискретная: одна строка = одна волна. |
| Per-rune параметры (сейчас только цена) | **xlsx** (лист Runes) | Горизонтальная таблица, одна строка на `RuneKind`. |
| Параметры рунного шопа (тир-веса, вероятность джокера, реролл-цены) | **xlsx** (лист Globals) | Key-value, префикс `rune_`. Не per-rune. |
| Глобальные настройки (safe-spawn-radius, арена, монеты) | **xlsx** (лист Globals) | Key-value список. |
| Визуалы частиц (`assets/particles/*.particle.ron`) | RON | Не трогается. |
| Палитра (`assets/palette.ron`) | RON | Не трогается. |

---

## Структура xlsx

Один файл `assets/balance.xlsx` содержит четыре листа: **Mobs**, **Waves**, **Runes**, **Globals**. Все четыре парсятся кодом; source of truth для каждого листа — он сам, никаких derived-views-из-формул между листами (в отличие от изначального плана с MobsCommon-XLOOKUP-из-MobsSpecific).

### Лист «Mobs» — общая балансная таблица

Горизонтальная таблица: по строке на моба, по колонке на общий параметр. Это та самая **общая таблица** — сидишь над ней, видишь всех мобов рядом, тюнишь числа бок-о-бок.

| id | hp | damage | speed | size | mass |
|----|-----|--------|-------|------|------|
| ghost | 5 | 8 | 120 | 150 | 1 |
| tower | 25 | 6 | | 150 | |
| slime_small | 3 | 5 | 300 | 90 | 1 |
| jumper | 10 | 8 | 400 | 200 | 10 |
| spinner | 15 | 10 | | 100 | 20 |

**Правила:**

- В Mobs попадают **только параметры, общие для нескольких мобов** и пригодные для кросс-сравнения. Текущий набор — `hp`, `damage`, `speed`, `size`, `mass`.
- Пустая ячейка = параметр у этого моба отсутствует (например, у башни нет `speed`/`mass`, потому что она статическая). Парсер маппит пустую ячейку в `None` для соответствующего `Option<f32>` в `MobCommonStats`.
- Колонка `id` — идентификатор моба, совпадает с `MobKind::id()` в `src/actors/mobs/mod.rs` (`ghost`, `tower`, `slime_small`, `jumper`, `spinner`). Валидация требует: каждый `MobKind` покрыт ровно одной строкой; неизвестный id — ошибка.
- Новое общее поле = новая колонка. Новый моб = новая строка.
- Отладочные колонки с заголовком, начинающимся на `_` (`_name`, `_notes`), игнорируются парсером. Полезны для подписей/заметок дизайнера.

**Если параметр есть только у одного моба** (например, `lunge_duration` у `slime_small`) — он **не едет в Mobs**, а живёт `const` в `src/actors/mobs/slime.rs`. Если со временем lunge появится у второго моба и захочется тюнить синхронно — поднимаем в Mobs как новую колонку.

### Лист «Waves» — прогрессия волн

Модель прогрессии: дискретные волны, одна строка = одна волна. Анлоки мобов живут отдельной колонкой прямо в этой же таблице — ячейка содержит id моба, который становится доступен **начиная с этой волны**.

Горизонтальная таблица:

| wave | unlocks | enemy_variety | max_concurrent | hp_multiplier | damage_multiplier |
|------|---------|---------------|----------------|---------------|-------------------|
| 1 | slime_small | 1 | 5 | 1.0 | 1.0 |
| 2 |  | 1 | 6 | 2.0 | 1.2 |
| 3 | ghost | 2 | 7 | 4.0 | 1.44 |
| 4 |  | 2 | 8 | 8.0 | 1.728 |
| 5 | tower | 3 | 8 | 16.0 | 2.0736 |
| … |  | … | … | … | … |
| 8 | jumper | 4 | 11 | 128.0 | 3.5832 |
| … |  | … | … | … | … |
| 13 | spinner | 5 | 15 | 4096.0 | 8.9161 |

Семантика (как в `src/wave/config.rs::WaveDef`):

- **`unlocks`** — id моба, разблокируемого начиная с этой волны; пустая ячейка = на этой волне никто не разблокируется. Парсер собирает из этой колонки `HashMap<MobKind, u32>` (мапу `моб → номер волны, на которой он появился`), которая дальше ведёт себя идентично прежнему `mob_unlocks`: `resolve_pool` гарантированно включает мобов, разблокированных на `wave` или `wave - 1`, и добирает остальными разблокированными до `enemy_variety`.
- **`enemy_variety`** — сколько разных типов мобов попадает в активный пул на этой волне.
- **`max_concurrent`** — одновременный лимит живых врагов на арене. Единственный регулятор темпа: волна заканчивается, когда игрок успевает всех убить. Ни `start_enemies`, ни `ramp_duration_sec` не поддерживаются (были удалены вместе с continuous-рампом).
- **`hp_multiplier` / `damage_multiplier`** — множители к базовому `hp`/`damage` моба на этой волне. Применяются при спавне через `WaveModifiers` в `src/wave/summoning.rs`.

**Правило хвоста:** если текущая волна превышает `waves.len()`, используется последняя строка — бесконечная прогрессия по последним значениям. Парсер требует ≥1 строки.

**Правила валидации:**

- Номера волн в колонке `wave` идут без разрывов, начиная с 1.
- Каждый `MobKind` встречается в колонке `unlocks` не более одного раза. Моб, не упомянутый ни в одной строке, считается никогда не разблокируемым (эквивалент `unlock_wave: 0` в старой модели) — такие мобы не появляются сами, только через dev-меню.
- Ячейка `unlocks` либо пустая, либо содержит id из `MobKind`. Неизвестный id — ошибка.
- `enemy_variety > 0`, `max_concurrent > 0`, множители > 0.
- `enemy_variety <= (число уникальных unlocks на этой и предыдущих волнах)` — иначе пул физически не соберётся; парсер предупреждает.

**Взаимодействие с dev-меню.** При `--features dev` есть меню `EnemySpawnPool` с toggles на каждый тип моба (`src/wave/spawn.rs`). Если дизайнер активно пользовался тогглами, `EnemySpawnPool` **полностью замещает** пул, построенный из unlocks/variety — волна в этот момент диктует только `max_concurrent` и множители, а выбор мобов идёт из dev-оверрайда.

### Лист «Runes» — per-rune параметры

Горизонтальная таблица: по строке на `RuneKind`, по колонке на per-rune параметр. Сейчас единственный такой параметр — `cost`.

| id | cost |
|----|------|
| spike | 3 |
| heart_stone | 8 |
| resonator | 10 |

`id` соответствует варианту `RuneKind` в `src/rune/content.rs`. Новая руна = новая строка + вариант в enum. Новый per-rune параметр (например, тир, базовый модификатор, какой-то per-rune cap) = новая колонка.

Всё, что **не** per-rune (глобальные параметры рунного шопа — тир-веса, вероятность джокера, стоимость реролла) живёт в Globals, а не здесь.

### Лист «Globals»

Глобальные настройки, не привязанные к per-mob/per-wave/per-rune таблицам. Key-value.

| key | value | _description |
|-----|-------|--------------|
| safe_spawn_radius | 200.0 | минимум дистанции моба до игрока при спавне |
| arena_width | 1200.0 | фиксированный размер арены |
| arena_height | 1200.0 | |
| coins_per_kill | 1 | номинал дропающейся с моба монеты |
| coin_attraction_duration | 0.5 | длительность полёта монеты в игрока |
| rune_joker_probability | 0.25 | шанс, что слот шопа выпадет джокером |
| rune_tier_weight_common | 60 | вес тира common при ролле шопа |
| rune_tier_weight_rare | 30 | вес тира rare |
| rune_reroll_base_cost | 2 | стоимость первого реролла за сессию шопа |
| rune_reroll_cost_step | 2 | рост стоимости за каждый следующий реролл |

Колонки:

| Колонка | Тип | Описание |
|---------|-----|----------|
| key | string | Уникальный ключ. |
| value | string | Значение (тип диктуется консьюмером). |
| _description | string | **Отладочная.** Описание для дизайнера. |

---

## Валидация в Excel (UX для дизайнера)

### Data Validation (Dropdown)

- Колонка `id` в Mobs, `id` в unlocks-блоке Waves, `id` в per-rune блоке Runes: Data Validation → List → жёсткий список значений `MobKind`/`RuneKind` (вводится при создании файла MCP-сервером). Стиль Stop.

### Conditional Formatting

- Mobs/Waves unlocks: если `id` не совпадает ни с одним `MobKind` — красная заливка (`ISNA(MATCH(...))`).
- Runes prices: то же для `RuneKind`.
- Mobs: подсветка ячеек, где обязательный для моба параметр оставлен пустым (решается по каждой колонке — например, `hp`/`damage`/`size` обязательны всегда, `speed`/`mass` опциональны для статических мобов).

### Deprecated мобы/руны

Вместо удаления строки — колонка `deprecated` (TRUE/FALSE) в Mobs/Runes. При парсинге такие строки исключаются, но id остаётся в dropdown-ах (для хвоста в Waves проще — ссылки через unlock-блок; при наличии TRUE в `deprecated` unlocks-запись на него просто игнорируется).

---

## Инструментарий

### Редактирование дизайнером

Основной путь — Excel или LibreOffice Calc. Файл `assets/balance.xlsx` хранится в репозитории под git (бинарный — diff не работает, merge conflict не резолвится штатно; для соло-работы это приемлемо, для командной — договариваться о монопольной правке).

### Редактирование Claude Code через Excel MCP-сервер

Для того чтобы Claude Code мог править баланс напрямую, а не через одноразовые скрипты, подключается community MCP-сервер для локальных xlsx (например, [`haris-musa/excel-mcp-server`](https://github.com/haris-musa/excel-mcp-server) — Python, без сети/OAuth). После установки у Claude появляются тулы уровня `read_sheet`/`write_cells`/`apply_formula`/`create_sheet` над локальными файлами.

**Установка (пример для `haris-musa/excel-mcp-server`):**

1. `pip install excel-mcp-server` (или `uvx excel-mcp-server`).
2. Добавить запись в `~/.claude.json` (или `.claude/mcp.json` в корне проекта):
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

**Use case:** через этот же сервер выполняется и **первоначальное создание** `assets/balance.xlsx` — Claude Code читает текущие `assets/{mobs,balance,runes,waves}.ron` и через MCP-тулы создаёт файл с четырьмя листами и Data Validation. Отдельной утилиты-мигратора нет. Дальнейшие точечные правки (добавление колонки в Mobs после добавления общего поля и т.п.) тоже идут через MCP; массовые правки значений дизайнер делает руками в Excel.

---

## Загрузка и парсинг

### Зависимости рантайма

```toml
[dependencies]
calamine = "0.26"   # чтение xlsx
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
// Общие (кросс-сравнимые) числа мобов из листа Mobs.
#[derive(Debug, Clone)]
pub struct MobCommonStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: Option<f32>,
    pub size: f32,
    pub mass: Option<f32>,
}

#[derive(Debug, Clone, Resource)]
pub struct MobsBalance {
    pub ghost: MobCommonStats,
    pub tower: MobCommonStats,
    pub slime_small: MobCommonStats,
    pub jumper: MobCommonStats,
    pub spinner: MobCommonStats,
}

#[derive(Debug, Clone)]
pub struct WaveDef {
    pub enemy_variety: u32,
    pub max_concurrent: u32,
    pub hp_multiplier: f32,
    pub damage_multiplier: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct WavesConfig {
    pub mob_unlocks: HashMap<MobKind, u32>,
    pub waves: Vec<WaveDef>,
}

#[derive(Debug, Clone, Resource)]
pub struct RuneCosts { /* поля по RuneKind */ }

#[derive(Debug, Clone, Resource)]
pub struct Globals {
    pub safe_spawn_radius: f32,
    pub arena_width: f32,
    pub arena_height: f32,
    pub coins_per_kill: u32,
    pub coin_attraction_duration: f32,
    pub rune_joker_probability: f32,
    pub rune_tier_weight_common: u32,
    pub rune_tier_weight_rare: u32,
    pub rune_reroll_base_cost: u32,
    pub rune_reroll_cost_step: u32,
}

#[derive(Debug, Clone, Resource)]
pub struct Balance {
    pub mobs: MobsBalance,
    pub waves: WavesConfig,
    pub rune_costs: RuneCosts,
    pub globals: Globals,
}
```

Разделение на под-ресурсы внутри `Balance` — так текущим консьюмерам (шоп использует `RuneCosts`, волна — `WavesConfig`, и т.д.) не нужно тянуть жирный `Balance` целиком; можно дублировать поля как отдельные `Resource`-клоны (как сейчас делается для RON) или держать только `Balance` и переписать консьюмеров — решается на этапе миграции.

### Алгоритм парсинга Mobs

1. Открыть workbook через `calamine::open_workbook_from_rs` (release) или `calamine::open_workbook` (dev, из файла).
2. Лист Mobs: первая строка — заголовок. Колонки с заголовком на `_` отбрасываются. Ожидаемые: `id`, `hp`, `damage`, `speed`, `size`, `mass`.
3. Для каждой строки: `id` → `MobKind`; числовые колонки → `Option<f32>` (пустая ячейка = `None`, непустая → `parse::<f32>()`).
4. Обязательные поля (`hp`/`damage`/`size`) должны быть заполнены для каждого моба; опциональные (`speed`/`mass`) могут отсутствовать у статических мобов — парсер не валится, консьюмер решает что делать с `None`.
5. Валидация: все `MobKind::iter()` покрыты ровно одной строкой; дубликатов нет.

### Алгоритм парсинга Waves

Один блок. Заголовок: `wave | unlocks | enemy_variety | max_concurrent | hp_multiplier | damage_multiplier`.

1. Каждая строка → `WaveDef` + опциональный `(MobKind, wave_number)` из колонки `unlocks`.
2. Пустая ячейка `unlocks` = ничего не разблокируется. Непустая → маппится в `MobKind` (неизвестный id — ошибка).
3. Строки сортируются по `wave`, номера проверяются на непрерывность начиная с 1.
4. Из собранных пар `(MobKind, wave)` строится `HashMap<MobKind, u32>` — это и есть `mob_unlocks` для `WavesConfig`. `MobKind`, не встретившийся ни в одной строке, в мапу не попадает (эквивалент «никогда не разблокируется»).

### Алгоритм парсинга Runes

Один блок. Заголовок: `id | cost | ...` (любые per-rune колонки). `id` → `RuneKind`; числовые колонки → соответствующие поля; из всех строк собирается `RuneCosts` (и другие per-rune ресурсы, если появятся).

### Алгоритм парсинга Globals

Key-value. Заголовок `key | value | _description` (последняя отбрасывается). Строки → `HashMap<String, String>` → типизированный `Globals`.

---

## Валидация в коде (runtime)

Вся балансная валидация живёт в `src/balance/loader.rs` и исполняется при загрузке и F5-перезагрузке.

### Проверки

1. **Mobs**: все `MobKind` покрыты; `id` уникальны; обязательные поля (`hp`/`damage`/`size`) заполнены; значения парсятся.
2. **Waves**: ≥1 строка; номера `wave` идут без разрывов, начиная с 1; все множители > 0; `enemy_variety > 0`; `max_concurrent > 0`. Значения в колонке `unlocks` либо пустые, либо валидные `MobKind`; каждый `MobKind` встречается в колонке `unlocks` не более одного раза. Предупреждение (не ошибка), если `enemy_variety` на волне превышает число мобов, разблокированных к этому моменту.
3. **Runes**: все `RuneKind` покрыты; `id` уникальны; обязательные per-rune поля заполнены.
4. **Globals**: все ожидаемые ключи присутствуют и парсятся (включая `rune_joker_probability`, `rune_tier_weight_{common,rare}`, `rune_reroll_base_cost`, `rune_reroll_cost_step`).

### Поведение при ошибках

- **`--features dev`**: panic с полным списком ошибок на старте; F5-перезагрузка при ошибке оставляет прежний `Balance` и логирует `error!`.
- **Release без dev**: `error!()`, пропуск невалидных записей, игра не крашится.

---

## Bevy-интеграция

### Регистрация в App

```rust
app.add_systems(Startup, setup_balance);

#[cfg(feature = "dev")]
app.add_systems(Update, reload_balance);
```

### Загрузка

Идёт **вне Bevy `AssetServer`**, не использует `AssetLoader`/`LoadedFolder`. Отдельный путь, параллельный `src/loading/`.

Система `setup_balance` на `Startup`:

- **Release**: `calamine::open_workbook_from_rs(Cursor::new(BALANCE_XLSX))`.
- **Dev**: `calamine::open_workbook("assets/balance.xlsx")`.

Парсит все четыре листа, валидирует, вставляет `Balance` (и/или отдельные под-ресурсы) как `Resource`.

### Горячая перезагрузка (только `--features dev`)

Система `reload_balance` в `Update`:

- Отслеживает `F5`.
- Перечитывает `assets/balance.xlsx` с диска, парсит, валидирует.
- При успехе — заменяет `Balance`, логирует `info!("Balance reloaded")`.
- При ошибке — `error!(...)`, старый `Balance` сохраняется.

Гейтинг: `#[cfg(feature = "dev")]`. Альтернатива — авто-watch через crate `notify`; на первом этапе не делаем, F5 даёт явный контроль.

### Потребление

Текущие консьюмеры:

- **`src/wave/spawn.rs`** (`spawn_enemies`, `apply_wave_config`): `safe_spawn_radius` из Globals; `arena_width`/`arena_height` из Globals (через `CurrentArenaSize`); `max_concurrent`/`enemy_variety` из текущей `WaveDef`; пул мобов — из unlocks + `resolve_pool`.
- **`src/wave/summoning.rs`** (`animate_summoning`): `hp_multiplier`/`damage_multiplier` из текущей `WaveDef` → `WaveModifiers` → `spawn_mob`.
- **`src/arena/spawn.rs`**: `arena_width`/`arena_height` из Globals.
- **`src/run/coin.rs`**: `coins_per_kill`, `coin_attraction_duration` из Globals.
- **`src/rune/shop_gen.rs`**: `rune_joker_probability`, `rune_tier_weight_*` из `Globals`.
- **`src/rune/scene.rs`**, **`src/ui/shop_hud.rs`**: `rune_reroll_base_cost`, `rune_reroll_cost_step` из `Globals`.
- **`src/rune/cost.rs`**: `RuneCosts` как сейчас.
- **`src/actors/mobs/*`** — per-mob `spawn_*`-функции берут `MobCommonStats` из `MobsBalance`. Уникальные числа (`shot_cooldown`, `flight_duration`, `jump_distance`, `charge_speed`, и т.д.) читают из модульных `const` в том же файле.

### Удаляется из кодовой базы при миграции

- `assets/balance.ron`, `assets/mobs.ron`, `assets/runes.ron`, `assets/waves.ron`.
- `src/balance.rs` (`GameBalance`, `WaveBalance`, `ArenaBalance`, `RunBalance`, `RuneBalance`, `TierWeights`) — замещается модулем `src/balance/`.
- Реализации `RonAsset` для `GameBalance` / `MobsBalance` / `RuneCosts` / `WavesConfig` в `src/loading/assets.rs`; соответствующие `.init_asset::<…>()` и `.register_asset_loader(…)` в `src/loading/mod.rs`.
- Поля `balance_handle`/`mobs_balance_handle`/`rune_costs_handle`/`waves_handle` в `src/loading/systems.rs::LoadingState` и их загрузка/poll в `start_loading`/`check_loaded`.
- В per-mob `*Stats`-структурах остаются только общие поля (или они вообще исчезают в пользу прямого чтения `MobCommonStats` + модульных `const`). Конкретная форма определяется на этапе рефакторинга.

---

## Файловая структура проекта

```
assets/
  balance.xlsx              — SSOT баланса (бинарный, хранится в git)
src/
  balance/
    mod.rs                  — публичный API, плагин, ресурс Balance
    types.rs                — MobCommonStats, MobsBalance, WaveDef, WavesConfig, RuneCosts, Globals
    parser.rs               — парсинг четырёх листов через calamine
    loader.rs               — загрузка (embedded / disk), валидация, F5-перезагрузка
```

---

## Порядок реализации

1. **Excel MCP-сервер.** Поставить (`haris-musa/excel-mcp-server` или аналог), прописать в `~/.claude.json`, проверить что появились тулы `mcp__excel__*`.
2. **Создание `assets/balance.xlsx`.** Claude Code через MCP читает текущие `assets/{mobs,balance,runes,waves}.ron` и создаёт файл с четырьмя листами. На этапе создания из mobs-блоков отбрасываются все уникальные per-mob поля (идут в `const` в Rust на шаге 7); в Mobs-лист попадают только `hp`/`damage`/`speed`/`size`/`mass`. Настраивается Data Validation и Conditional Formatting.
3. `src/balance/types.rs` — структуры данных.
4. `src/balance/parser.rs` — парсинг четырёх листов через calamine.
5. `src/balance/loader.rs` — загрузка (embedded / disk), валидация, F5.
6. `src/balance/mod.rs` — плагин, ресурс `Balance` (и/или под-ресурсы).
7. **Рефакторинг мобов.** Уникальные per-mob поля выносятся из `*Stats`-структур и из `spawn_*`-аргументов в `const` в `src/actors/mobs/<mob>.rs`. `*Stats` сжимается до `MobCommonStats` (или исчезает совсем — `spawn_*` читает `MobCommonStats` напрямую из `MobsBalance`).
8. **Миграция потребителей** (`src/wave/`, `src/arena/`, `src/run/`, `src/rune/`, `src/ui/shop_hud.rs`): `GameBalance`/`MobsBalance`/`WavesConfig`/`RuneCosts`-ресурсы → ресурсы из `Balance`.
9. **Удаление** RON-файлов баланса, `src/balance.rs`, `RonAsset`-имплементаций и регистраций в `src/loading/`.
10. Unit-тесты парсинга и валидации на зафиксированном тестовом xlsx (в `src/balance/*.rs` через `#[cfg(test)]`; тестовый файл — `src/balance/testdata/`).

---

## Известные TODO

- **Fireball → xlsx, когда появятся варианты.** Сейчас `FIREBALL_*` — `const` в `src/actors/player.rs`. Если появятся герои/билды с разным балансом fireball, вынести в отдельный лист с `variant_id`.
- **Веса спавна vs dev-меню.** Полное замещение пула dev-оверрайдом — сознательный выбор; в будущем можно дать слайдер вместо бинарных тогглов.
- **Веса мобов внутри пула волны.** `WaveDef` не хранит весов; выбор из активного пула равновероятен. При необходимости расширить модель (вес на моба per-wave или на моба глобально).
- **Auto-watch вместо F5.** Опционально подключить `notify` для авто-перезагрузки по сохранению файла.
- **Командная работа с xlsx.** Бинарный diff — проблема при коллаборации. Варианты: монопольная правка, track-changes в Excel, либо экспорт в CSV per-sheet как вторичный формат для code review.
- **Поднятие уникального параметра в общую таблицу.** Когда какой-то `const` (например, `melee_range`) понадобится у второго моба и захочется тюнить синхронно — добавляется колонка в Mobs, константы из соответствующих `.rs` удаляются, консьюмеры читают из `MobCommonStats`.
