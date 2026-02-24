# Спецификация: Генерируемое дерево навыков (Passive Skill Tree)

## 1. Обзор

Генерируемый на каждый забег планарный граф пассивных навыков (~400 нод), визуально похожий на дерево из Path of Exile. Чем дальше нода от центра — тем более редкий навык на ней. Игрок получает +3 skill points в каждом магазине и тратит их на изучение нод, прокладывая путь от центра.

## 2. RON-формат: пул нод

**Файл:** `assets/skill_tree/passives.ron`

```ron
(
    nodes: [
        (
            name: "Sharp Claws",
            rarity: "common",
            modifiers: [(stats: [Fixed(stat: "physical_damage_flat", value: 3.0)])],
        ),
        (
            name: "Thick Skin",
            rarity: "common",
            modifiers: [(stats: [Fixed(stat: "max_life_flat", value: 10.0)])],
        ),
        (
            name: "Eagle Eye",
            rarity: "rare",
            modifiers: [
                (stats: [Fixed(stat: "crit_chance_flat", value: 0.05)]),
                (stats: [Fixed(stat: "crit_multiplier", value: 0.25)]),
            ],
        ),
        // ... ~100-200 определений нод
    ],
    rarity_order: ["common", "uncommon", "rare", "epic"],
)
```

**Текстовый id не нужен** — ноды нигде не адресуются по имени. Индекс в массиве достаточен.

**Имя — флейворное** ("Sharp Claws", "Eagle Eye"). Конкретные эффекты ноды отображаются через существующую `display` систему из `config.stats.ron` — тот же механизм, что у артефактов и аффиксов.

**Формат модификаторов** — тот же `ModifierDefRaw`/`StatRangeRaw`, что и в артефактах. `Fixed` и `Range` поддерживаются. Значения `Range` роллятся при генерации графа (каждая вершина получает конкретные числа). Игрок в тултипе видит уже заролленные значения, а не диапазон.

**`rarity_order`** — задаёт порядок редкости от центра к краям. Генератор распределяет ноды по кольцам в этом порядке.

**Ноды могут повторяться в графе** — одно определение может быть размещено на нескольких вершинах графа (как в PoE, где "+10 str" встречается десятки раз). Пул определений (~100-200) раскидывается по ~400 вершинам.

## 3. Модуль `src/skill_tree/`

```
src/skill_tree/
├── mod.rs              # SkillTreePlugin, регистрация систем
├── types.rs            # PassiveNodeDef, PassiveNodeDefRaw, Rarity, PassiveNodePool
├── graph.rs            # SkillGraph, GraphNode, рёбра, аллокация
├── generation.rs       # Алгоритм генерации графа
├── systems.rs          # grant_skill_points, handle_allocate_node
└── loader.rs           # AssetLoader для passives.ron

src/ui/
└── skill_tree_view.rs  # UI рендер: 2D сущности, камера, overlay, тултипы, pan/zoom
```

## 4. Типы данных

### 4.1. Определения нод (types.rs)

```rust
// Rarity — порядок определяется rarity_order из RON
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Rarity(u8);

// Определение ноды (resolved)
struct PassiveNodeDef {
    name: String,
    rarity: Rarity,
    modifiers: Vec<ModifierDef>,        // из stats::ModifierDef
}

// Определение ноды (RON raw)
struct PassiveNodeDefRaw {
    name: String,
    rarity: String,
    modifiers: Vec<ModifierDefRaw>,     // из stats::ModifierDefRaw
}

// Пул всех определений, загруженный из RON
struct PassiveNodePool {
    defs: Vec<PassiveNodeDef>,
    by_rarity: HashMap<Rarity, Vec<usize>>,  // rarity → индексы в defs[]
    rarity_order: Vec<Rarity>,               // от центра к краям
}
```

### 4.2. Граф (graph.rs)

```rust
// Вершина сгенерированного графа
struct GraphNode {
    def_index: usize,           // Индекс в PassiveNodePool.defs[]
    position: Vec2,             // Позиция для рендера UI (логические координаты)
    rarity: Rarity,             // Редкость (определена при генерации)
    rolled_values: Vec<(StatId, f32)>,  // Значения модификаторов, заролленные при генерации
    allocated: bool,            // Изучена ли
}

// Ребро графа
struct GraphEdge {
    a: usize,                   // Индекс вершины
    b: usize,                   // Индекс вершины
}

// Весь сгенерированный граф — Resource
#[derive(Resource)]
struct SkillGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    adjacency: Vec<Vec<usize>>,  // adjacency[node_idx] = [neighbor_indices]
    start_node: usize,           // Центральная нода (авто-изучена)
    seed: u64,                   // Сид генерации (для воспроизводимости)
}
```

**Методы `SkillGraph`:**

| Метод | Описание |
|-------|----------|
| `is_allocatable(node_idx)` | Нода не изучена И хотя бы один сосед изучен |
| `allocate(node_idx)` | Пометить ноду как изученную |
| `allocated_count()` | Кол-во изученных нод |
| `allocatable_nodes()` | Итератор по нодам, доступным для изучения |
| `neighbors(node_idx)` | Соседи вершины |

### 4.3. Ресурсы (resources в mod.rs или systems.rs)

```rust
// Очки навыков игрока
#[derive(Resource, Default)]
struct SkillPoints(u32);

// Пул определений (загружен из RON)
#[derive(Resource)]
struct PassiveNodePool { ... }  // см. 4.1
```

## 5. Алгоритм генерации (generation.rs)

### 5.1. Общий план

```
Seed → Poisson Disk Sampling → точки с позициями
     → Delaunay Triangulation → рёбра (планарный граф)
     → Назначение колец по расстоянию от центра
     → Назначение нод из пула по кольцу/редкости
     → SkillGraph
```

### 5.2. Размещение точек: Poisson Disk Sampling

- Область: круг радиуса `R`
- Минимальное расстояние: `d_min` (подбирается чтобы получить ~400 точек)
- Центральная точка `(0, 0)` размещается первой (стартовая нода)
- Результат: ~400 равномерно распределённых точек

Равномерное распределение гарантирует, что граф не вытягивается в ниточку.

### 5.3. Связи: Delaunay Triangulation + прореживание

- Триангуляция Делоне по полученным точкам
- Результат — планарный граф (по определению), средняя степень ~6

**Прореживание до ~3-4 связей на вершину:**
Delaunay слишком плотный (~1200 рёбер на 400 нод). Прореживаем до ~700-800 рёбер:
1. Удалить рёбра длиннее порога (например, > 1.5× медианной длины)
2. При удалении проверять: граф остаётся связным (не разрывать мосты)
3. Результат: средняя степень ~3-4, граф визуально ближе к PoE (не сплошная сетка, но с циклами)

### 5.4. Назначение редкости (вероятностная модель)

Редкость — не жёсткие зоны, а вероятностные веса. Редкая нода может появиться у центра (маловероятно), а common — на краю (маловероятно).

**Расстояние → веса редкости:**

Для каждой вершины вычисляется нормализованное расстояние `t ∈ [0, 1]` от центра. Затем для каждой редкости считается вес через кривую:

```
weight(rarity, t) — функция, задающая вероятность редкости на расстоянии t

Пример распределения (4 редкости):
t=0.0 (центр):  common=90%, uncommon=8%,  rare=1.5%, epic=0.5%
t=0.5 (середина): common=30%, uncommon=40%, rare=25%,  epic=5%
t=1.0 (край):   common=5%,  uncommon=15%, rare=40%,  epic=40%
```

Конкретные кривые/веса хранятся в `passives.ron`:

```ron
(
    // ...nodes, rarity_order...
    rarity_weights: (
        center: [90, 8, 2, 0],   // веса для t=0 (индексы соответствуют rarity_order)
        edge:   [5, 15, 40, 40], // веса для t=1
    ),
)
```

Для промежуточных `t` — линейная интерполяция между `center` и `edge`.

### 5.5. Назначение нод из пула

Для каждой вершины графа:
1. Вычислить `t` (нормализованное расстояние от центра)
2. Рассчитать веса редкостей по `t` (линейная интерполяция center↔edge)
3. Weighted random — выбрать rarity
4. Случайно выбрать определение ноды из `PassiveNodePool.by_rarity[rarity]`
5. Заролить `rolled_values` из `ModifierDef` (аналогично `ArtifactDef::roll_values`)
6. Одно определение может быть выбрано многократно (дубли допустимы)

### 5.6. Стартовая нода

Центральная вершина (`start_node`):
- Не имеет бонусов (пустая нода, не из пула, `def_index` = специальное значение)
- Авто-изучена при генерации
- Соседи стартовой ноды сразу доступны для изучения
- Визуально: уникальный вид (золотой/белый, фиксированный размер), отличается от обычных нод

## 6. Системы (systems.rs)

### 6.1. Генерация дерева

**`generate_skill_tree`**
- Запускается: `OnEnter(GameState::Playing)` (или при создании нового забега)
- Входы: `PassiveNodePool`, сид (из RNG или фиксированный)
- Выходы: вставляет `SkillGraph` как Resource, вставляет `SkillPoints(0)`

### 6.2. Выдача очков

**`grant_skill_points`**
- Запускается: `OnEnter(WavePhase::Shop)`
- Действие: `skill_points.0 += 3`

### 6.3. Изучение ноды

**Сообщение:**
```rust
#[derive(Message)]
struct AllocateNodeRequest {
    node_index: usize,
}
```

**`handle_allocate_node`**
- Schedule: `Update`, `in_set(ShopSet::Process)`, `run_if(in_state(WavePhase::Shop))`
- Проверяет: `skill_points >= 1` И `graph.is_allocatable(node_index)`
- Действия:
  1. `graph.allocate(node_index)`
  2. `skill_points.0 -= 1`
  3. Для каждого `(stat, value)` в `graph.nodes[idx].rolled_values` — `modifiers.add(stat, value, None)`

### 6.4. Очистка при завершении забега

**`cleanup_skill_tree`**
- Запускается: `OnExit(GameState::Playing)`
- Действия:
  1. Удалить `SkillGraph` resource
  2. Сбросить `SkillPoints` в 0
  3. Модификаторы от нод (с `source: None`) — удаляются вместе с игроком при game over
  4. 2D сущности дерева (ноды/рёбра) — `DespawnOnExit(WavePhase::Shop)` или явный cleanup

## 7. Загрузка (loader.rs)

**Asset type:** `PassiveNodePoolAsset`
- Загружается из `passives.ron` через Bevy `AssetLoader`
- При загрузке: `PassiveNodeDefRaw` → `resolve()` → `PassiveNodeDef` (через `StatRegistry`)
- Результат вставляется как `Resource<PassiveNodePool>`

Паттерн идентичен текущему `ArtifactRegistry` — `ModifierDefRaw.resolve(stat_registry)`.

## 8. UI

### 8.1. Размещение в игровом потоке

Вкладка/секция экрана магазина (`WavePhase::Shop`). Магазин получает кнопку-переключатель между Shop и Skill Tree (или обе видны одновременно — магазин слева, дерево справа).

### 8.2. Архитектура рендера: два слоя

**Слой 1 — 2D World (граф):**
Ноды и рёбра рендерятся как 2D-сущности (спрайты/мешы), а не через Bevy UI, потому что:
- ~400 нод + рёбра — свободное 2D-размещение, не flex-layout
- Нужен pan/zoom (камера)
- Bevy UI не имеет примитива линий для рёбер

Используется выделенная `SkillTreeCamera` (Camera2d с отдельным order/layer), активная только во время `WavePhase::Shop` + активной вкладки дерева.

**Слой 2 — Bevy UI (overlay):**
Поверх графа стандартным Bevy UI рисуются:
- Счётчик skill points
- Тултип при наведении
- Кнопка переключения вкладок (Shop / Tree)

### 8.3. Модуль

```
src/ui/skill_tree_view.rs   # или src/ui/skill_tree_view/ если разрастётся
```

### 8.4. Визуальное кодирование: редкость + состояние одновременно

**Редкость → цвет заливки ноды:**

| Rarity   | Цвет                |
|----------|---------------------|
| Common   | серый `#A0A0A0`     |
| Uncommon | зелёный `#40C040`   |
| Rare     | синий `#4080FF`     |
| Epic     | фиолетовый `#A040E0`|

(Цвета конфигурируемы — в константах или RON.)

**Состояние → яркость + обводка:**

| Состояние  | Яркость заливки | Обводка              | Доп. эффект        |
|------------|-----------------|----------------------|--------------------|
| Locked     | ~30% (тусклый)  | нет                  | —                  |
| Available  | ~80%            | белая/золотая рамка  | —                  |
| Allocated  | 100%            | тонкая рамка в цвет  | заливка полная     |

Таким образом **цвет** читается как редкость, а **яркость + рамка** — как состояние. Обе характеристики видны одновременно.

Пример:
- Тусклый фиолетовый кружок без рамки = epic + locked
- Яркий зелёный кружок с белой рамкой = uncommon + available
- Полностью синий кружок = rare + allocated

### 8.5. Рёбра (edges)

Рёбра рендерятся как тонкие линии (Mesh2d или Gizmos). Яркость ребра определяется состоянием его концов:

| Концы ребра              | Яркость линии |
|--------------------------|---------------|
| Allocated ↔ Allocated    | яркая         |
| Allocated ↔ Available    | средняя       |
| Иначе (хотя бы 1 locked)| тусклая       |

### 8.6. Размеры нод

Размер ноды зависит от редкости — редкие ноды крупнее, что дополнительно подчёркивает их значимость:

| Rarity   | Радиус (логич. координаты) |
|----------|---------------------------|
| Common   | ~16                       |
| Uncommon | ~20                       |
| Rare     | ~26                       |
| Epic     | ~32                       |

(Конкретные значения подбираются при тюнинге.)

### 8.7. Взаимодействие

**Pan:** зажать ПКМ (или среднюю) и двигать → перемещение камеры.

**Zoom:** колёсико мыши → масштаб камеры (clamp min/max).

**Клик по ноде (ЛКМ):**
1. Экранные координаты курсора → мировые через `SkillTreeCamera`
2. Найти ближайшую ноду в радиусе
3. Если `Available` и `skill_points >= 1` → `AllocateNodeRequest`

**Hover:**
- При наведении (nearest node в радиусе) — показать тултип

### 8.8. Тултип

Появляется рядом с курсором (Bevy UI, `PositionType::Absolute`). Содержит:

1. **Имя ноды** (флейворное) — золотой цвет, аналогично артефактам
2. **Модификаторы** — через существующий `StatLineBuilder::spawn_line()` + `StatDisplayRegistry::get_format()`. Тот же механизм что в `artifact_tooltip.rs`

Тултип не показывается для allocated нод (опционально — можно показывать с пометкой "Learned").

### 8.9. Overlay UI (Bevy UI поверх графа)

```
┌──────────────────────────────────────────┐
│  [Shop] [Skill Tree]     Skill Points: 5 │  ← верхняя панель
│                                          │
│        (граф нод — 2D world)             │
│                                          │
│                    [тултип у курсора]     │
│                                          │
│                          [Next Wave]      │  ← нижняя кнопка
└──────────────────────────────────────────┘
```

**Компоненты overlay:**

| Компонент             | Описание |
|-----------------------|----------|
| `SkillTreeTab`        | Кнопка переключения на вкладку дерева |
| `ShopTab`             | Кнопка переключения на вкладку магазина |
| `SkillPointsText`     | "Skill Points: N" — обновляется при изменении `SkillPoints` |
| `SkillTreeTooltip`    | Тултип ноды (по аналогии с `ArtifactTooltip`) |

### 8.10. Состояние вкладки

```rust
#[derive(Resource, Default, PartialEq, Eq)]
enum ShopView {
    #[default]
    Shop,
    SkillTree,
}
```

- Переключается кнопками `ShopTab` / `SkillTreeTab`
- При `ShopView::Shop` — показан магазин (текущий UI), камера дерева выключена
- При `ShopView::SkillTree` — магазин скрыт, камера дерева активна, overlay видим
- Сбрасывается в `Shop` при `OnEnter(WavePhase::Shop)`

### 8.11. Камера и видимость

```rust
#[derive(Component)]
struct SkillTreeCamera;
```

- Отдельная `Camera2d` с `RenderLayers::layer(1)` (или аналогичный механизм)
- Ноды и рёбра тоже на этом layer
- Камера активна только когда `ShopView::SkillTree`
- Pan/zoom — модифицируют `Transform` камеры

### 8.12. Сущности графа (2D world)

Каждая нода — Entity:
```rust
#[derive(Component)]
struct SkillTreeNode {
    graph_index: usize,     // индекс в SkillGraph.nodes[]
}
```

Компоненты ноды-entity: `SkillTreeNode`, `Sprite` (или `Mesh2d`), `Transform`, `RenderLayers`.

Рёбра — отдельные Entity с `Mesh2d` (thin quad) или рисуются через `Gizmos`.

### 8.13. Системы UI

| Система                        | Когда                | Что делает |
|-------------------------------|----------------------|------------|
| `spawn_skill_tree_view`       | При переключении на вкладку Tree | Спавнит 2D сущности нод/рёбер + overlay UI |
| `despawn_skill_tree_view`     | При переключении на вкладку Shop / выходе из магазина | Чистит 2D сущности |
| `update_node_visuals`         | Update, когда `SkillGraph` changed | Обновляет цвета/яркость нод и рёбер по текущему состоянию |
| `skill_tree_pan_zoom`         | Update, вкладка Tree | Pan (ПКМ drag) + zoom (scroll) камеры |
| `skill_tree_click`            | ShopSet::Input, вкладка Tree | Screen→world, find node, send AllocateNodeRequest |
| `skill_tree_hover`            | Update, вкладка Tree | Находит ноду под курсором, показывает/обновляет тултип |
| `update_skill_points_text`    | Update, вкладка Tree | Обновляет текст "Skill Points: N" |

## 9. Зависимости от крейтов

| Крейт | Зачем |
|-------|-------|
| (встроенный или свой) | Poisson disk sampling |
| `delaunator` или `spade` | Delaunay triangulation |
| `rand` | Уже используется в проекте |

Альтернатива: реализовать Poisson disk и Delaunay вручную (~200 строк каждый) чтобы не тянуть зависимости.

## 10. Интеграция с существующими системами

| Система | Интеграция |
|---------|-----------|
| `Modifiers` / `ComputedStats` | Изученные ноды добавляют модификаторы к `Player` entity через `modifiers.add()` |
| `ModifierDef` / `StatRange` | RON-формат нод переиспользует тот же формат что артефакты |
| `WavePhase::Shop` | Skill points выдаются при входе в магазин |
| `ShopSet` | `AllocateNodeRequest` обрабатывается в `ShopSet::Process` |
| `StatRegistry` | Ноды ссылаются на те же stat_id что и артефакты |
| `GameState::Playing` | Граф генерируется при входе в Playing |

## 11. Наполнение пула нод

Ноды строятся на существующих статах из `config.stats.ron`. Ниже — ориентировочный пул. Имена флейворные, эффект — через модификаторы.

### Common (~60 определений)

Одностатовые, небольшие бонусы. Составляют основную массу графа.

| Имя | Модификаторы |
|-----|-------------|
| Sharp Claws | physical_damage_flat +3 |
| Iron Fist | physical_damage_flat +4 |
| Thick Skin | max_life_flat +8 |
| Endurance | max_life_flat +12 |
| Light Step | movement_speed_increased +0.04 |
| Quick Feet | movement_speed_increased +0.03 |
| Keen Edge | crit_chance_flat +0.01 |
| Steady Aim | projectile_speed_flat +15 |
| Wide Swing | area_of_effect_flat +5 |
| Lingering Force | duration_flat +0.15 |
| Toughness | max_life_increased +0.03 |
| Fury | physical_damage_increased +0.04 |
| Focus | max_mana_flat +5 |
| Concentration | max_mana_increased +0.03 |
| Precision Shot | projectile_speed_increased +0.04 |
| Scatter | projectile_count +1 |
| ... | (ещё ~45 вариаций, микшируя статы и числа) |

### Uncommon (~40 определений)

Чуть сильнее и/или двухстатовые.

| Имя | Модификаторы |
|-----|-------------|
| Predator's Mark | physical_damage_flat +6, crit_chance_flat +0.01 |
| Fortified Mind | max_life_flat +15, max_mana_flat +8 |
| Wind Runner | movement_speed_increased +0.06, projectile_speed_increased +0.04 |
| Battle Hardened | max_life_increased +0.05, physical_damage_flat +3 |
| Hawk Eye | crit_chance_flat +0.02, crit_multiplier +0.10 |
| Broad Reach | area_of_effect_flat +10, duration_flat +0.1 |
| Vital Surge | max_life_flat +20 |
| Heavy Strike | physical_damage_flat +8 |
| Arcane Flow | max_mana_flat +12, max_mana_increased +0.04 |
| ... | (ещё ~30 вариаций) |

### Rare (~25 определений)

Заметные бонусы, часто мультистатовые.

| Имя | Модификаторы |
|-----|-------------|
| Butcher's Glee | physical_damage_increased +0.12, crit_multiplier +0.15 |
| Ironclad | max_life_flat +30, max_life_increased +0.06 |
| Tempest | projectile_speed_increased +0.10, projectile_count +1 |
| Death's Precision | crit_chance_flat +0.04, crit_multiplier +0.20 |
| Titan's Blood | max_life_flat +40, max_life_more +0.05 |
| Berserker's Fury | physical_damage_flat +10, physical_damage_increased +0.08 |
| Cataclysm | area_of_effect_increased +0.12, duration_increased +0.08 |
| ... | (ещё ~18 вариаций) |

### Epic (~15 определений)

Сильные мультистатовые, редкие.

| Имя | Модификаторы |
|-----|-------------|
| Avatar of War | physical_damage_increased +0.15, physical_damage_more +0.10, movement_speed_increased +0.05 |
| Undying Will | max_life_flat +50, max_life_increased +0.10, max_life_more +0.08 |
| Storm of Steel | projectile_count +2, projectile_speed_increased +0.12 |
| Inevitable End | crit_chance_flat +0.06, crit_multiplier +0.35 |
| Eternal Torrent | area_of_effect_increased +0.15, duration_increased +0.15, physical_damage_increased +0.08 |
| ... | (ещё ~10 вариаций) |

**Итого:** ~140 определений → раскидываются по ~400 вершинам (дубли common нод — норма).

Конкретные числа — черновые, балансируются после плейтеста. Главный принцип: common = маленькие одностатовые, epic = большие мультистатовые.

## 12. Ограничения и будущие расширения

**Не входит в текущую спеку:**
- Респек (снятие изученных нод) — потребует source-entity для remove_by_source
- Визуальные кластеры / sub-regions (тематические зоны: "огонь", "защита")
- Keystones (ноды с уникальным эффектом + трейдоффом)
- Jewel sockets (вставляемые ноды)
