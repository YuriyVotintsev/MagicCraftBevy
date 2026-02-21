# Спецификация: Единая система выражений и шаблоны calc()

## 1. Текущее состояние

### 1.1. Два типа выражений

В проекте существуют два независимых типа выражений:

**`stats::Expression`** — используется в калькуляторах статов.
Записывается в RON как дерево:
```ron
formula: Clamp(
    value: Mul(Stat("crit_chance_base"), Add(Constant(1.0), Stat("crit_chance_increased"))),
    min: 0.0, max: 1.0,
)
```
Варианты: `Constant`, `Stat`, `ModifierSum`, `ModifierProduct`, `Add`, `Sub`, `Mul`, `Div`, `Min`, `Max`, `Clamp`.

**`blueprints::ScalarExpr`** — используется в параметрах абилок.
Записывается в RON как строка, парсится в AST:
```ron
speed: "600 * (1 + stat(projectile_speed_increased))"
```
Варианты: `Literal`, `Stat`, `Index`, `Count`, `Add`, `Sub`, `Mul`, `Div`, `Neg`, `Min`, `Max`, `Length`, `Distance`, `Dot`, `X`, `Y`, `Angle`, `Recalc`.

Оба типа дублируют арифметику и работу со статами, но живут в разных модулях и имеют разный синтаксис записи.

### 1.2. Дублирование формул в абилках

Формулы в RON-файлах абилок повторяются и отличаются только базовым значением:
```ron
// fireball
DamagePayload((amount: "(15 + stat(physical_damage_base)) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)"))
// caustic_arrow
DamagePayload((amount: "(2 + stat(physical_damage_base) * 0.15) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)"))
```

Таких паттернов четыре: physical_damage, projectile_speed, area_of_effect, duration. Каждый повторяется от 2 до 8 раз.

### 1.3. Суффикс `_base`

Статы вида `physical_damage_base`, `max_life_base` и т.д. используют суффикс `_base`. В терминологии PoE корректнее `_flat`.

---

## 2. Целевое состояние

### 2.1. Единый тип выражений

Один `ScalarExpr` содержит **все** варианты — математику, статы и blueprint-контекст.

Raw-версии (`ScalarExprRaw`, `VecExprRaw`) используют `String` для имён статов. Resolved-версии (`ScalarExpr`, `VecExpr`) используют `StatId`. Два отдельных enum — без generic-параметризации, как и в остальных Raw→Resolved парах проекта.

Существующий `ExprFamily` trait удаляется полностью. Все типы, которые сейчас параметризованы через `ExprFamily` (`EntityDef<F>`, `StateDef<F>` и т.д.), переделываются на прямые Raw/Resolved пары (`EntityDefRaw`/`EntityDef`, `StateDefRaw`/`StateDef`) — без generic-параметризации.

```rust
// src/expr/

pub enum ScalarExpr {
    // Математика
    Literal(f32),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Neg(Box<Self>),
    Min(Box<Self>, Box<Self>),
    Max(Box<Self>, Box<Self>),
    Clamp { value: Box<Self>, min: Box<Self>, max: Box<Self> },
    // Статы
    Stat(StatId),
    // Blueprint-контекст (spawn)
    Index,
    Count,
    Recalc(Box<Self>),
    // Vec → Scalar
    Length(Box<VecExpr>),
    Distance(Box<VecExpr>, Box<VecExpr>),
    Dot(Box<VecExpr>, Box<VecExpr>),
    X(Box<VecExpr>),
    Y(Box<VecExpr>),
    Angle(Box<VecExpr>),
}

// ScalarExprRaw — идентичный enum, но Stat(String) вместо Stat(StatId),
// и VecExprRaw вместо VecExpr. Аналогично для VecExpr/VecExprRaw.

pub enum VecExpr {
    // Математика
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Scale(Box<Self>, Box<ScalarExpr>),
    Normalize(Box<Self>),
    Rotate(Box<Self>, Box<ScalarExpr>),
    Lerp(Box<Self>, Box<Self>, Box<ScalarExpr>),
    Vec2Expr(Box<ScalarExpr>, Box<ScalarExpr>),
    FromAngle(Box<ScalarExpr>),
    // Blueprint-контекст (spawn)
    CasterPos,
    SourcePos,
    SourceDir,
    TargetPos,
    TargetDir,
    Recalc(Box<Self>),
}

pub enum EntityExpr {
    Caster, Source, Target, Recalc(Box<Self>),
}
```

`StatId` — тривиальный newtype, живёт в `src/expr/`:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StatId(pub usize);
```

### 2.2. EvalCtx и StatProvider

`expr/` не зависит ни от `stats/`, ни от `blueprints/`. Доступ к вычисленным статам — через trait:

```rust
// src/expr/

pub trait StatProvider {
    fn get_stat(&self, id: StatId) -> f32;
}

pub struct EvalCtx<'a> {
    pub stats: &'a dyn StatProvider,
    // Blueprint-контекст (spawn) — заполняется нулями если не нужен
    pub index: usize,
    pub count: usize,
    pub caster_pos: Vec2,
    pub source_pos: Vec2,
    pub source_dir: Vec2,
    pub target_pos: Vec2,
    pub target_dir: Vec2,
    pub caster_entity: Option<Entity>,
    pub source_entity: Option<Entity>,
    pub target_entity: Option<Entity>,
}

impl EvalCtx<'_> {
    /// Контекст для stat-формул: blueprint-поля заполнены нулями.
    pub fn stat_only(stats: &dyn StatProvider) -> EvalCtx {
        EvalCtx {
            stats,
            index: 0, count: 0,
            caster_pos: Vec2::ZERO, source_pos: Vec2::ZERO, source_dir: Vec2::ZERO,
            target_pos: Vec2::ZERO, target_dir: Vec2::ZERO,
            caster_entity: None, source_entity: None, target_entity: None,
        }
    }
}
```

Eval — один метод, один match:

```rust
impl ScalarExpr {
    pub fn eval(&self, ctx: &EvalCtx) -> f32 {
        match self {
            Self::Literal(v) => *v,
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            Self::Sub(a, b) => a.eval(ctx) - b.eval(ctx),
            Self::Mul(a, b) => a.eval(ctx) * b.eval(ctx),
            Self::Div(a, b) => { let d = b.eval(ctx); if d.abs() < f32::EPSILON { 0.0 } else { a.eval(ctx) / d } },
            Self::Neg(a) => -a.eval(ctx),
            Self::Min(a, b) => a.eval(ctx).min(b.eval(ctx)),
            Self::Max(a, b) => a.eval(ctx).max(b.eval(ctx)),
            Self::Clamp { value, min, max } => value.eval(ctx).clamp(min.eval(ctx), max.eval(ctx)),
            Self::Stat(id) => ctx.stats.get_stat(*id),
            Self::Index => ctx.index as f32,
            Self::Count => ctx.count as f32,
            Self::Recalc(e) => e.eval(ctx),
            Self::Length(v) => v.eval(ctx).length(),
            Self::Distance(a, b) => a.eval(ctx).distance(b.eval(ctx)),
            Self::Dot(a, b) => a.eval(ctx).dot(b.eval(ctx)),
            Self::X(v) => v.eval(ctx).x,
            Self::Y(v) => v.eval(ctx).y,
            Self::Angle(v) => { let v = v.eval(ctx); v.y.atan2(v.x) },
        }
    }
}
```

Аналогичный `eval` для `VecExpr` (возвращает `Vec2`) и `EntityExpr` (возвращает `Option<Entity>`).

Каждый модуль конструирует `EvalCtx` из своих типов:

```rust
// src/stats/ — impl trait
impl StatProvider for ComputedStats {
    fn get_stat(&self, id: StatId) -> f32 { self.get(id) }
}

// Использование в stat evaluator:
let ctx = EvalCtx::stat_only(&computed_stats);
let value = formula.eval(&ctx);

// src/blueprints/ — полный контекст из SpawnSource
let ctx = EvalCtx {
    stats: computed_stats,
    index: source.index,
    count: source.count,
    caster_pos: source.caster.position.unwrap_or(Vec2::ZERO),
    source_pos: source.source.position.unwrap_or(Vec2::ZERO),
    source_dir: source.source.direction.unwrap_or(Vec2::ZERO),
    target_pos: source.target.position.unwrap_or(Vec2::ZERO),
    target_dir: source.target.direction.unwrap_or(Vec2::ZERO),
    caster_entity: source.caster.entity,
    source_entity: source.source.entity,
    target_entity: source.target.entity,
};
let value = expr.eval(&ctx);
```

### 2.3. Парсер и ограничение контекста

Единый Pratt-парсер обрабатывает общий синтаксис: числа, операторы, скобки. Доменные атомы (`stat(...)`, `index`, `caster_pos` и т.д.) делегируются через trait:

```rust
// src/expr/

pub trait AtomParser {
    fn try_parse_scalar(&mut self, name: &str, lexer: &mut Lexer) -> Result<Option<ScalarExprRaw>, ParseError>;
    fn try_parse_vec(&mut self, name: &str, lexer: &mut Lexer) -> Result<Option<VecExprRaw>, ParseError>;
}
```

Две реализации:

- **`StatAtomParser`** (в `src/stats/`) — распознаёт только `stat(name)`. Отвергает `index`, `count`, `caster_pos` и прочие blueprint-атомы с ошибкой.
- **`BlueprintAtomParser`** (в `src/blueprints/`) — распознаёт все атомы.

Ошибки ловятся на этапе парсинга, до runtime:

```ron
// config.stats.ron — ошибка парсинга
(name: "max_life", eval: Formula("index + stat(max_life_flat)"))
// → ParseError: unknown atom 'index' in stat formula context
```

### 2.4. Resolve: Raw → Resolved

`resolve()` конвертирует `ScalarExprRaw` → `ScalarExpr` (`String` → `StatId`). Чтобы `expr/` не зависел от `StatRegistry` (живёт в `stats/`), resolve принимает замыкание:

```rust
// src/expr/
impl ScalarExprRaw {
    pub fn resolve(self, lookup: &impl Fn(&str) -> StatId) -> ScalarExpr {
        match self {
            Self::Literal(v) => ScalarExpr::Literal(v),
            Self::Stat(name) => ScalarExpr::Stat(lookup(&name)),
            Self::Add(a, b) => ScalarExpr::Add(
                Box::new(a.resolve(lookup)),
                Box::new(b.resolve(lookup)),
            ),
            Self::Index => ScalarExpr::Index,
            // ... аналогично для всех вариантов
        }
    }
}
```

Вызов из `stats/` и `blueprints/`:

```rust
let resolved = raw_expr.resolve(&|name| stat_registry.get_id(name));
```

Аналогичный `resolve` для `VecExprRaw` → `VecExpr`.

### 2.5. Formula вместо Standard/Custom/calculators

Было — три вида агрегации + отдельная секция `calculators`:
```ron
(name: "max_life", aggregation: Standard(base: "max_life_base", increased: "max_life_increased", more: Some("max_life_more"))),
(name: "crit_chance", aggregation: Custom),
calculators: [(stat: "crit_chance", formula: Clamp(...), depends_on: [...])],
```

Стало — `Formula("...")` заменяет и `Standard`, и `Custom`. Секция `calculators` удалена.
Формулы проходят тот же парсер и resolve, поэтому внутри `Formula` можно использовать `calc()` шаблоны:
```ron
(name: "max_life", eval: Formula("calc(flat_increased_more, 0, max_life_flat, max_life_increased, max_life_more)")),
(name: "max_mana", eval: Formula("calc(flat_increased, 0, max_mana_flat, max_mana_increased)")),
(name: "movement_speed", eval: Formula("calc(flat_increased, 0, movement_speed_flat, movement_speed_increased)")),
(name: "crit_chance", eval: Formula("clamp(stat(crit_chance_flat) * (1 + stat(crit_chance_increased)), 0, 1)")),
```

`StatEvalKindRaw`:
- `Sum` — `modifiers.sum(stat_id)`
- `Product` — `modifiers.product(stat_id)`
- `Formula(String)` — парсится через `StatAtomParser`, резолвится в `ScalarExpr`, вычисляется через `expr.eval(&EvalCtx::stat_only(computed))`

```rust
// RON — десериализация (Raw)
pub enum StatEvalKindRaw {
    Sum,
    Product,
    Formula(String),
}

// После парсинга и резолва
pub enum StatEvalKind {
    Sum,
    Product,
    Formula(ScalarExpr),    // единый тип
}

// В StatEvaluator:
pub fn evaluate(&self, stat: StatId, modifiers: &Modifiers, computed: &ComputedStats) -> f32 {
    match &entry.kind {
        StatEvalKind::Sum => modifiers.sum(stat),
        StatEvalKind::Product => modifiers.product(stat),
        StatEvalKind::Formula(expr) => expr.eval(&EvalCtx::stat_only(computed)),
    }
}
```

Выражение ничего не знает о модификаторах. `depends_on` для Formula извлекается автоматически из дерева (все `Stat(id)` ноды).

### 2.6. Шаблоны calc()

Шаблоны определяются в `assets/stats/calcs.ron` рядом с `config.stats.ron`. Шаблоны используют арифметику и `stat(...)`.

```ron
// assets/stats/calcs.ron
[
    // Общие шаблоны
    (name: "flat_increased_more", params: ["base", "flat", "increased", "more"],
     formula: "(base + stat(flat)) * (1 + stat(increased)) * stat(more)"),

    (name: "flat_increased", params: ["base", "flat", "increased"],
     formula: "(base + stat(flat)) * (1 + stat(increased))"),

    (name: "increased_more", params: ["base", "increased", "more"],
     formula: "base * (1 + stat(increased)) * stat(more)"),

    (name: "flat_more", params: ["base", "flat", "more"],
     formula: "(base + stat(flat)) * stat(more)"),

    // Конкретные шаблоны — вызывают общие
    (name: "physical_damage", params: ["base"],
     formula: "calc(flat_increased_more, base, physical_damage_flat, physical_damage_increased, physical_damage_more)"),

    (name: "projectile_speed", params: ["base"],
     formula: "calc(flat_increased, base, projectile_speed_flat, projectile_speed_increased)"),

    (name: "area_of_effect", params: ["base"],
     formula: "calc(flat_increased, base, area_of_effect_flat, area_of_effect_increased)"),

    (name: "duration", params: ["base"],
     formula: "calc(flat_increased, base, duration_flat, duration_increased)"),
]
```

Использование в абилках:
```ron
DamagePayload((amount: "calc(physical_damage, 15)"))
Speed((value: "calc(projectile_speed, 600)"))
Size((value: "calc(area_of_effect, 160)"))
Lifetime((remaining: "calc(duration, 4)"))
```

Шаблоны рекурсивные: `physical_damage` вызывает `flat_increased_more` внутри. Лимит глубины — 16.

### 2.7. Как работает подстановка

Подстановка — текстовая: `CalcRegistry::expand()` работает со строками **до** парсинга. Парсер никогда не видит `calc()` — они полностью раскрыты.

При вызове `calc(physical_damage, 15)`:
1. Находим шаблон `physical_damage` с `params: ["base"]`
2. Подставляем `base = 15`
3. Формула шаблона: `calc(flat_increased_more, base, physical_damage_flat, ...)` — `base` заменяется на `15`
4. Рекурсивно раскрываем `flat_increased_more` с аргументами `15, physical_damage_flat, ...`
5. Результат: `(15 + stat(physical_damage_flat)) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)`

Два вида подстановки:
- **Выражение-аргумент** (числа, формулы): `base = 15` → подставляется как текст
- **Имя стата**: `flat = physical_damage_flat` → подставляется как имя в `stat(...)`

Подстановка по word boundaries: параметр заменяется только если символы до и после не являются `[a-zA-Z0-9_]`. Это исключает ложные замены внутри слов (например, параметр `flat` не затронет `flatline`).

### 2.8. Агрегация модификаторов — в эвалюаторе, не в выражениях

Старый подход: `ModifierSum(id)` и `ModifierProduct(id)` были вариантами `Expression`.

Новый подход: `StatEvalKind` напрямую определяет как вычислять стат. `ModifierSum`/`ModifierProduct` убраны из выражений (§2.5).

### 2.9. Суффикс `_flat`

Все статы с `_base` переименовываются в `_flat`. Затрагивает ~30 RON-файлов и 3 места в Rust-коде.

Механическая задача, ортогональна системе выражений. Делать отдельным таск-агентом, чтобы не засорять основной контекст 30+ файлами.

---

## 3. Модульная структура

```
assets/
  stats/calcs.ron              — шаблоны calc() (текстовые шаблоны)
  stats/config.stats.ron       — stat_ids (eval: Sum, Product, Formula), display

src/expr/                      — Автономный модуль (ни от кого не зависит)
  mod.rs                       — ScalarExpr, VecExpr, EntityExpr
                                 ExprFamily, Raw, Resolved
                                 StatId
                                 StatProvider trait
                                 EvalCtx struct + stat_only()
                                 resolve(), uses_stats(), uses_recalc(), collect_stat_deps()
  parser.rs                    — Pratt parser + AtomParser trait
  calc.rs                      — CalcTemplate, CalcRegistry (Bevy Resource)
                                 Текстовая подстановка: expand()

src/stats/                     — Зависит от expr
  stat_registry.rs             — StatRegistry (name → StatId mapping)
                                 impl StatProvider for ComputedStats
                                 StatAtomParser
  evaluator.rs                 — StatEvaluator, StatEvalKind { Sum, Product, Formula(ScalarExpr) }

src/blueprints/                — Зависит от expr, stats
  expr.rs                      — BlueprintAtomParser
                                 EvalCtx конструирование из SpawnSource
```

Зависимости:
```
src/expr/       → ничего
src/stats/      → expr
src/blueprints/ → expr, stats
```

---

## 4. Жизненный цикл выражения

### 4.1. Blueprint выражение

```
RON строка "calc(physical_damage, 15)"
    ↓ AssetLoader: Serde десериализация — строка сохраняется as-is
ScalarExprRaw (нераспарсенная строка)
    ↓ Finalization: CalcRegistry::expand() — текстовая подстановка
"(15 + stat(physical_damage_flat)) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)"
    ↓ parse(BlueprintAtomParser) — Pratt parser, все атомы разрешены
ScalarExpr<Raw>: Mul(Add(Literal(15), Stat("physical_damage_flat")), ...)
    ↓ resolve(|name| stat_registry.get_id(name)) — String → StatId
ScalarExpr: Mul(Add(Literal(15), Stat(#7)), Mul(Add(Literal(1), Stat(#8)), Stat(#9)))
    ↓ eval(&EvalCtx { stats, index, count, caster_pos, ... })
f32 = 42.0
```

### 4.2. Stat формула

```
RON строка "clamp(stat(crit_chance_flat) * (1 + stat(crit_chance_increased)), 0, 1)"
    ↓ Finalization: CalcRegistry::expand() (если есть calc() вызовы)
    ↓ parse(StatAtomParser) — только stat(), index/count/caster_pos → ParseError
ScalarExpr<Raw>: Clamp { value: Mul(Stat("crit_chance_flat"), ...), min: 0.0, max: 1.0 }
    ↓ resolve(|name| stat_registry.get_id(name))
ScalarExpr: Clamp { value: Mul(Stat(#3), Add(Literal(1), Stat(#4))), min: 0.0, max: 1.0 }
    ↓ eval(&EvalCtx::stat_only(computed))
f32 = 0.35
```

Тот же тип `ScalarExpr`, та же функция `eval`. Разница — в `AtomParser` при парсинге и в конструкторе `EvalCtx` при вызове.

Raw-типы выражений хранят неразобранные строки. Expand, парсинг и resolve происходят в finalization-системе, где доступны `Res<CalcRegistry>` и `Res<StatRegistry>`. Это стандартный паттерн проекта: AssetLoader только десериализует, resolve — позже.

После expand() шаблонов в строке не остаётся — `calc()` полностью раскрыты. Парсер видит чистое выражение из арифметики и `stat()`. Runtime работает с плоским деревом без обращений к CalcRegistry.

---

## 5. Валидация

### 5.1. Тест `validate_expressions` (`cargo test`)

Собирает **все** ошибки и выводит разом. Проверяет:

**calcs.ron:**
- Все шаблоны парсятся
- `calc()` внутри шаблонов ссылаются на существующие шаблоны
- Количество аргументов совпадает с `params`
- Нет циклических ссылок
- `stat(name)` ссылаются на существующие статы

**config.stats.ron:**
- Все `Formula("...")` парсятся через `StatAtomParser`
- `stat(name)` внутри формул ссылаются на существующие статы
- `calc()` внутри формул ссылаются на существующие шаблоны с правильной арностью
- Blueprint-атомы (`index`, `count`, `caster_pos` и др.) отвергнуты парсером

**Блупринты (ability/mob RON):**
- Все строковые выражения парсятся через `BlueprintAtomParser`
- `stat(name)` ссылаются на существующие статы
- `calc()` ссылаются на существующие шаблоны с правильной арностью

Расширяет существующие тесты в `validation_tests.rs` и `blueprints/tests.rs`.

### 5.2. Headless (`cargo run --features headless -- --timeout 10`)

Полный пайплайн: парсинг → resolve → runtime. Ловит ошибки которые тест не покрывает (например, деление на ноль в runtime, неправильные значения статов).

---

## 6. Миграция

Обратная совместимость не требуется. Всё старое удаляется.

### 6.1. `AggregationType` → `StatEvalKind`

| Старое | Новое | Действие |
|--------|-------|----------|
| `Sum` | `StatEvalKind::Sum` | Остаётся. `modifiers.sum(stat_id)` |
| `Product` | `StatEvalKind::Product` | Остаётся. `modifiers.product(stat_id)` |
| `Standard { base, increased, more }` | `StatEvalKind::Formula(...)` | Удаляется. Заменяется на `Formula("calc(flat_increased_more, ...)")` или аналог |
| `Custom` | `StatEvalKind::Formula(...)` | Удаляется. Формула из `calculators` переносится inline |

Enum `AggregationType` удаляется целиком, заменяется на `StatEvalKindRaw` (§2.5).

### 6.2. Секция `calculators` в `config.stats.ron`

Удаляется целиком. Формулы переезжают inline в `eval: Formula("...")` соответствующего стата.

### 6.3. `stats::Expression`

Enum `stats::Expression<S>` и файл `src/stats/expression.rs` удаляются. Замена — единый `ScalarExpr` из `src/expr/`.

### 6.4. `blueprints::ScalarExpr` → `expr::ScalarExpr`

Enum `blueprints::ScalarExpr<F>` переезжает в `src/expr/`. Добавляется `Clamp` (был только в stats). Остальные варианты остаются.

### 6.5. Промежуточные статы

Статы `*_base`/`*_increased`/`*_more` (после переименования — `*_flat`/`*_increased`/`*_more`) **остаются**. Они по-прежнему `eval: Sum` или `eval: Product` и собирают модификаторы. `Formula`-статы ссылаются на них через `stat(...)`.

### 6.6. `config.stats.ron` — формат до/после

**До:**
```ron
(
    stat_ids: [
        (name: "max_life_base", aggregation: Sum),
        (name: "max_life_increased", aggregation: Sum),
        (name: "max_life_more", aggregation: Product),
        (name: "max_life", aggregation: Standard(
            base: "max_life_base", increased: "max_life_increased", more: Some("max_life_more"),
        )),
        (name: "crit_chance", aggregation: Custom),
    ],
    calculators: [
        (stat: "crit_chance", formula: Clamp(value: Mul(...), min: 0.0, max: 1.0), depends_on: [...]),
    ],
)
```

**После:**
```ron
(
    stat_ids: [
        (name: "max_life_flat", eval: Sum),
        (name: "max_life_increased", eval: Sum),
        (name: "max_life_more", eval: Product),
        (name: "max_life", eval: Formula("calc(flat_increased_more, 0, max_life_flat, max_life_increased, max_life_more)")),
        (name: "crit_chance", eval: Formula("clamp(stat(crit_chance_flat) * (1 + stat(crit_chance_increased)), 0, 1)")),
    ],
)
```

Нет `calculators`, нет `aggregation`, нет `depends_on`. Поле `eval` вместо `aggregation`.
