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

### 2.1. Макро-система послойных выражений

Выражения строятся послойно через накопительные декларативные макросы. Каждый уровень определяет два макроса — для вариантов enum и для match-армов eval. Макрос добавляет свои варианты к накопленным и делегирует предыдущему уровню через callback-паттерн:

```rust
macro_rules! with_level_n {
    ($cb:ident ! { $($accumulated:tt)* }) => {
        with_level_n_minus_1! { $cb ! {
            NewVariant,
            $($accumulated)*
        }}
    };
}
```

**Генераторы** — полностью generic, ничего не знают о содержимом:

```rust
// src/expr/macros.rs

macro_rules! make_scalar_enum {
    { $name:ident; $($body:tt)* } => {
        #[derive(Debug, Clone)]
        pub enum $name { $($body)* }
    };
}

macro_rules! make_scalar_eval {
    { $name:ty, $ctx:ty; $($pat:pat => $expr:expr),* $(,)? } => {
        impl $name {
            pub fn eval(&self, ctx: &$ctx) -> f32 {
                match self { $($pat => $expr),* }
            }
        }
    };
}
```

Аналогичные `make_vec_enum!` и `make_vec_eval!` для `VecExpr` (возвращает `Vec2`).

#### Уровень 1 — Базовая математика (`src/expr/`)

```rust
macro_rules! with_math {
    ($cb:ident ! { $name:ident; $($rest:tt)* }) => {
        $cb! { $name;
            Literal(f32),
            Add(Box<Self>, Box<Self>),
            Sub(Box<Self>, Box<Self>),
            Mul(Box<Self>, Box<Self>),
            Div(Box<Self>, Box<Self>),
            Neg(Box<Self>),
            Min(Box<Self>, Box<Self>),
            Max(Box<Self>, Box<Self>),
            Clamp { value: Box<Self>, min: f32, max: f32 },
            $($rest)*
        }
    };
}

macro_rules! with_math_eval {
    ($cb:ident ! { $($rest:tt)* }) => {
        $cb! {
            Self::Literal(v) => *v,
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            Self::Sub(a, b) => a.eval(ctx) - b.eval(ctx),
            Self::Mul(a, b) => a.eval(ctx) * b.eval(ctx),
            Self::Div(a, b) => safe_div(a.eval(ctx), b.eval(ctx)),
            Self::Neg(a) => -a.eval(ctx),
            Self::Min(a, b) => a.eval(ctx).min(b.eval(ctx)),
            Self::Max(a, b) => a.eval(ctx).max(b.eval(ctx)),
            Self::Clamp { value, min, max } => value.eval(ctx).clamp(*min, *max),
            $($rest)*
        }
    };
}
```

#### Уровень 2 — Статы (`src/stats/`)

Добавляет `Stat(S)`. Параметр `$S` — тип ссылки на стат (`String` для Raw, `StatId` для Resolved):

```rust
macro_rules! with_stat {
    ($cb:ident ! { $name:ident; $S:ty; $($rest:tt)* }) => {
        with_math! { $cb ! { $name; Stat($S), $($rest)* } }
    };
}

macro_rules! with_stat_eval {
    ($cb:ident ! { $($rest:tt)* }) => {
        with_math_eval! { $cb ! {
            Self::Stat(id) => ctx.stats.get(*id),
            $($rest)*
        }}
    };
}
```

#### Уровень 3 — Блупринты (`src/blueprints/`)

Добавляет `Index`, `Count`, `Recalc` и vec→scalar операции к скалярам. Параметр `$V` — тип VecExpr:

```rust
macro_rules! with_blueprint_scalar {
    ($cb:ident ! { $name:ident; $S:ty; $V:ty; $($rest:tt)* }) => {
        with_stat! { $cb ! { $name; $S;
            Index, Count, Recalc(Box<Self>),
            Length(Box<$V>), Distance(Box<$V>, Box<$V>), Dot(Box<$V>, Box<$V>),
            X(Box<$V>), Y(Box<$V>), Angle(Box<$V>),
            $($rest)*
        }}
    };
}

macro_rules! with_blueprint_scalar_eval {
    ($cb:ident ! { $($rest:tt)* }) => {
        with_stat_eval! { $cb ! {
            Self::Index => ctx.source.index as f32,
            Self::Count => ctx.source.count as f32,
            Self::Recalc(e) => e.eval(ctx),
            Self::Length(v) => v.eval(ctx).length(),
            Self::Distance(a, b) => a.eval(ctx).distance(b.eval(ctx)),
            Self::Dot(a, b) => a.eval(ctx).dot(b.eval(ctx)),
            Self::X(v) => v.eval(ctx).x,
            Self::Y(v) => v.eval(ctx).y,
            Self::Angle(v) => { let v = v.eval(ctx); v.y.atan2(v.x) },
            $($rest)*
        }}
    };
}
```

VecExpr — два уровня (статы не используют векторы):

```rust
// src/expr/ — базовая векторная математика
macro_rules! with_vec_math {
    ($cb:ident ! { $name:ident; $Scalar:ty; $($rest:tt)* }) => {
        $cb! { $name;
            Add(Box<Self>, Box<Self>),
            Sub(Box<Self>, Box<Self>),
            Scale(Box<Self>, Box<$Scalar>),
            Normalize(Box<Self>),
            Rotate(Box<Self>, Box<$Scalar>),
            Lerp(Box<Self>, Box<Self>, Box<$Scalar>),
            Vec2Expr(Box<$Scalar>, Box<$Scalar>),
            FromAngle(Box<$Scalar>),
            $($rest)*
        }
    };
}

// src/blueprints/ — + spawn context
macro_rules! with_vec_blueprint {
    ($cb:ident ! { $name:ident; $Scalar:ty; $($rest:tt)* }) => {
        with_vec_math! { $cb ! { $name; $Scalar;
            CasterPos, SourcePos, SourceDir, TargetPos, TargetDir,
            Recalc(Box<Self>),
            $($rest)*
        }}
    };
}
```

Аналогичные `with_vec_math_eval!` и `with_vec_blueprint_eval!` для eval-армов VecExpr.

EntityRef — не параметризован, обычный enum:

```rust
pub enum EntityRef {
    Caster, Source, Target, Recalc(Box<Self>),
}
```

**Генерация типов:**

```rust
// src/stats/
with_stat! { make_scalar_enum ! { StatExprRaw; String; } }
with_stat! { make_scalar_enum ! { StatExpr;    StatId; } }

// src/blueprints/
with_blueprint_scalar! { make_scalar_enum ! { BlueprintExprRaw; String; BlueprintVecExprRaw; } }
with_blueprint_scalar! { make_scalar_enum ! { BlueprintExpr;    StatId; BlueprintVecExpr;    } }

with_vec_blueprint! { make_vec_enum ! { BlueprintVecExprRaw; BlueprintExprRaw; } }
with_vec_blueprint! { make_vec_enum ! { BlueprintVecExpr;    BlueprintExpr;    } }
```

**Результат после раскрытия — плоские enum'ы без обёрток:**

```rust
// StatExpr — 10 вариантов
pub enum StatExpr {
    Literal(f32), Add(..), Sub(..), Mul(..), Div(..), Neg(..), Min(..), Max(..), Clamp{..},
    Stat(StatId),
}

// BlueprintExpr — 22 варианта
pub enum BlueprintExpr {
    Literal(f32), Add(..), Sub(..), Mul(..), Div(..), Neg(..), Min(..), Max(..), Clamp{..},
    Stat(StatId), Index, Count, Recalc(..),
    Length(..), Distance(..), Dot(..), X(..), Y(..), Angle(..),
}
```

### 2.2. Контексты eval

Каждый уровень определяет struct-контекст. Контексты — суперсеты: каждый следующий содержит все поля предыдущих.

```rust
// src/stats/
pub struct StatCtx<'a> {
    pub stats: &'a ComputedStats,
}

// src/blueprints/
pub struct BlueprintCtx<'a> {
    pub stats: &'a ComputedStats,     // суперсет StatCtx
    pub source: &'a SpawnSource,
}
```

Конвенция: `ctx.stats` доступно на всех уровнях ≥ 2. Макрос `with_stat_eval!` генерирует `Self::Stat(id) => ctx.stats.get(*id)` — работает с любым контекстом, содержащим поле `stats`.

**Генерация eval:**

```rust
// src/stats/
with_stat_eval!             { make_scalar_eval ! { StatExpr,      StatCtx;      } }

// src/blueprints/
with_blueprint_scalar_eval! { make_scalar_eval ! { BlueprintExpr, BlueprintCtx; } }
with_vec_blueprint_eval!    { make_vec_eval !    { BlueprintVecExpr, BlueprintCtx; } }
```

Eval после раскрытия — один плоский match, без делегирования:

```rust
impl BlueprintExpr {
    pub fn eval(&self, ctx: &BlueprintCtx) -> f32 {
        match self {
            Self::Literal(v) => *v,
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            // ... арифметика ...
            Self::Stat(id) => ctx.stats.get(*id),       // из with_stat_eval!
            Self::Index => ctx.source.index as f32,      // из with_blueprint_scalar_eval!
            Self::Count => ctx.source.count as f32,
            Self::Recalc(e) => e.eval(ctx),
            Self::Length(v) => v.eval(ctx).length(),
            // ...
        }
    }
}
```

**Resolve** Raw → Resolved — отдельная функция per level, конвертирует `String` → `StatId` через `StatRegistry`:

```rust
impl StatExprRaw {
    pub fn resolve(&self, reg: &StatRegistry) -> StatExpr { ... }
}
impl BlueprintExprRaw {
    pub fn resolve(&self, reg: &StatRegistry) -> BlueprintExpr { ... }
}
```

Resolve — рекурсивный обход дерева. Арифметические варианты копируют структуру, `Stat(name)` → `Stat(reg.get(name))`. Можно генерировать макросом по аналогии с eval.

### 2.3. Formula вместо Standard/Custom/calculators

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
- `Formula(String)` — парсится, резолвится в `StatExpr`, вычисляется через `expr.eval(&StatCtx { stats: computed })`

`depends_on` извлекается автоматически из дерева формулы (все `Stat(id)` ноды).

### 2.4. Шаблоны calc()

Шаблоны определяются в `assets/stats/calcs.ron` рядом с `config.stats.ron`. Эти шаблоны используют максимум уровень 2 (StatExpr) — арифметику и `stat(...)`.

Если в будущем блупринтам понадобятся шаблоны с `index`, `count`, `caster` и др. — они будут в отдельном файле (например `assets/blueprints/calcs.ron`). Пока не требуется.

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

### 2.5. Как работает подстановка

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

### 2.6. Агрегация модификаторов — в эвалюаторе, не в выражениях

Старый подход: `ModifierSum(id)` и `ModifierProduct(id)` были вариантами `Expression`.

Новый подход: `StatEvalKind` напрямую определяет как вычислять стат. `ModifierSum`/`ModifierProduct` убраны из выражений:

```rust
// RON — десериализация (Raw)
pub enum StatEvalKindRaw {
    Sum,                // modifiers.sum(stat_id)
    Product,            // modifiers.product(stat_id)
    Formula(String),    // парсится → StatExpr → eval(&StatCtx { stats: computed })
}

// После резолва
pub enum StatEvalKind {
    Sum,
    Product,
    Formula(StatExpr),  // уже распарсена и зарезолвлена
}

// В StatEvaluator:
pub fn evaluate(&self, stat: StatId, modifiers: &Modifiers, computed: &ComputedStats) -> f32 {
    match &entry.kind {
        StatEvalKind::Sum => modifiers.sum(stat),
        StatEvalKind::Product => modifiers.product(stat),
        StatEvalKind::Formula(expr) => expr.eval(&StatCtx { stats: computed }),
    }
}
```

Выражение ничего не знает о модификаторах. `depends_on` для Formula извлекается автоматически из дерева (все `Stat` ноды).

### 2.7. Суффикс `_flat`

Все статы с `_base` переименовываются в `_flat`. Затрагивает ~30 RON-файлов и 3 места в Rust-коде.

Механическая задача, ортогональна системе выражений. Делать отдельным таск-агентом, чтобы не засорять основной контекст 30+ файлами.

---

## 3. Модульная структура

```
assets/
  stats/calcs.ron              — шаблоны calc() (текстовые шаблоны)
  stats/config.stats.ron       — stat_ids (eval: Sum, Product, Formula), display

src/expr/                      — Инфраструктура (без зависимостей от stats/blueprints)
  macros.rs                    — Генераторы: make_scalar_enum!, make_scalar_eval!,
                                   make_vec_enum!, make_vec_eval!
                                 Уровень 1: with_math!, with_math_eval!,
                                   with_vec_math!, with_vec_math_eval!
  parser.rs                    — Pratt parser + AtomParser trait
                                 stat(), clamp(), min(), max() — общий для всех уровней
  calc.rs                      — CalcTemplate, CalcRegistry (Resource)
                                 Текстовая подстановка: expand()

src/stats/                     — Уровень 2: + stat(...)
  expr.rs                      — with_stat!, with_stat_eval!
                                 StatExpr, StatExprRaw, resolve()
                                 StatCtx, StatAtomParser
  evaluator.rs                 — StatEvaluator, StatEvalKind { Sum, Product, Formula(StatExpr) }

src/blueprints/                — Уровень 3: + spawn context
  expr.rs                      — with_blueprint_scalar!, with_blueprint_scalar_eval!
                                 with_vec_blueprint!, with_vec_blueprint_eval!
                                 BlueprintExpr, BlueprintVecExpr, resolve()
                                 BlueprintCtx, BlueprintAtomParser
                                 EntityRef { Caster, Source, Target, Recalc }
```

Зависимости (каждый уровень видит только предыдущие):
```
src/expr/       → ничего
src/stats/      → expr
src/blueprints/ → expr, stats
```

---

## 4. Жизненный цикл выражения

```
RON строка "calc(physical_damage, 15)"
    ↓ AssetLoader: Serde десериализация — строка сохраняется as-is
ScalarExprRaw("calc(physical_damage, 15)")
    ↓ Finalization system: CalcRegistry::expand() — текстовая подстановка
"(15 + stat(physical_damage_flat)) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)"
    ↓ parse() — Pratt parser
BlueprintExprRaw: Mul(Add(Literal(15), Stat("physical_damage_flat")), ...)
    ↓ resolve(&stat_registry) — String → StatId
BlueprintExpr: Mul(Add(Literal(15), Stat(#7)), Mul(Add(Literal(1), Stat(#8)), Stat(#9)))
    ↓ eval(&BlueprintCtx { source, stats })
f32 = 42.0
```

Raw-типы выражений (`ScalarExprRaw`, `StatEvalKindRaw::Formula`) хранят неразобранные строки. Expand, парсинг и resolve происходят в finalization-системе, где доступны `Res<CalcRegistry>` и `Res<StatRegistry>`. Это стандартный паттерн проекта: AssetLoader только десериализует, resolve — позже.

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
- Нет blueprint-уровневых узлов (`index`, `count`, `caster_pos`, `target_dir` и др.)

**config.stats.ron:**
- Все `Formula("...")` парсятся
- `stat(name)` внутри формул ссылаются на существующие статы
- `calc()` внутри формул ссылаются на существующие шаблоны с правильной арностью
- Нет blueprint-уровневых узлов (`index`, `count`, `caster_pos`, `target_dir` и др.)

**Блупринты (ability/mob RON):**
- Все строковые выражения в компонентах парсятся
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

Enum `AggregationType` удаляется целиком, заменяется на `StatEvalKindRaw` (§2.6).

### 6.2. Секция `calculators` в `config.stats.ron`

Удаляется целиком. Формулы переезжают inline в `eval: Formula("...")` соответствующего стата.

### 6.3. `stats::Expression`

Enum `stats::Expression<S>` и файл `src/stats/expression.rs` удаляются. Замена — макро-генерируемый `StatExpr` (§2.1, уровень 2).

### 6.4. Промежуточные статы

Статы `*_base`/`*_increased`/`*_more` (после переименования — `*_flat`/`*_increased`/`*_more`) **остаются**. Они по-прежнему `eval: Sum` или `eval: Product` и собирают модификаторы. `Formula`-статы ссылаются на них через `stat(...)`.

### 6.5. `config.stats.ron` — формат до/после

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
