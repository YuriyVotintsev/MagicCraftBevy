# Полный код ревью проекта MagicCraftBevy

## Общая оценка: 6/10

Проект имеет хорошую базовую структуру с правильным разделением на модули (abilities, stats, fsm), но содержит значительное количество дублирования, неконсистентностей и незавершённых решений.

---

## 1. ДУБЛИРОВАНИЕ КОДА

### 1.1 Expression vs StatExpression ✅ ПОДТВЕРЖДЕНО

**Файлы:** `src/expression/mod.rs` и `src/abilities/expression.rs`

**Решение:** Оставить `Expression` из `src/expression/mod.rs`, удалить `StatExpression` из `abilities/expression.rs`. Добавить недостающий `PercentOf` в `Expression`, заменить все использования `StatExpression` в abilities.

**Проблема:** Два практически идентичных enum для вычисления значений на основе статов:

```rust
// expression/mod.rs
pub enum Expression {
    Constant(f32),
    Stat(StatId),
    ModifierSum(StatId),
    ModifierProduct(StatId),
    Add, Sub, Mul, Div, Min, Max, Clamp
}

// abilities/expression.rs
pub enum StatExpression {
    Constant(f32),
    Stat(StatId),
    Add, Sub, Mul, Div, Min, Max, Clamp,
    PercentOf { stat, percent }
}
```

**Аргументы:**
- Нарушает DRY (Don't Repeat Yourself)
- При изменении логики вычислений нужно менять в двух местах
- Разные названия для одного концепта запутывают
- `Expression` используется в stats/calculators, `StatExpression` в abilities - но делают одно и то же

**Рекомендация:** Объединить в один тип, добавив все варианты (ModifierSum, ModifierProduct, PercentOf).

---

### 1.2 Bullet vs Projectile системы ✅ ПОДТВЕРЖДЕНО

**Файлы:** `src/bullet.rs` и `src/abilities/projectile_systems.rs`

**Решение:** Удалить `src/bullet.rs` полностью - это остатки старой системы. Использовать только Projectile из abilities.

**Проблема:** Две параллельные системы для снарядов с практически идентичным кодом:

| bullet.rs | projectile_systems.rs |
|-----------|----------------------|
| `Bullet` компонент | `Projectile` компонент |
| `Velocity(Vec2)` | `ProjectileVelocity(Vec2)` |
| `move_bullets()` | `move_projectiles()` |
| `bullet_enemy_collision()` | `projectile_collision()` |

**Аргументы:**
- Одинаковая логика: движение по velocity, проверка границ арены, коллизия с мобами
- `bullet.rs` даже не используется - игрок стреляет через ability system (Projectile)
- Функция `spawn_bullet` нигде не вызывается
- Дублирование констант: `BULLET_SIZE = 15.0` и `PROJECTILE_SIZE = 15.0`

**Рекомендация:** Удалить `bullet.rs` полностью, использовать только Projectile из abilities.

---

### 1.3 Дублирование констант размеров ✅ ПОДТВЕРЖДЕНО

**Файлы:** `bullet.rs:6`, `projectile_systems.rs:10`, `arena.rs:14`

**Решение:** Добавить поле коллайдера в MobDef (настройка в .ron файлах) + компонент `Collider` в рантайме. При спавне читаем из MobDef, создаём компонент. Коллизии проверяют через компонент.

**Проблема:**
```rust
// bullet.rs
const MOB_SIZE: f32 = 30.0;
const BULLET_SIZE: f32 = 15.0;

// projectile_systems.rs
const PROJECTILE_SIZE: f32 = 15.0;
const ENEMY_SIZE: f32 = 30.0;

// arena.rs
const SLIME_SIZE: f32 = 30.0;
```

**Аргументы:**
- Три разных названия для одного значения (MOB_SIZE, ENEMY_SIZE, SLIME_SIZE)
- При изменении размера моба нужно менять в трёх местах
- Размер снаряда дублируется
- Размер моба должен быть в MobDef, а не константой

**Рекомендация:** Хранить размеры в компонентах сущностей (добавить `Collider { radius: f32 }`) или в определениях (MobDef.visual.size уже есть, но не используется для коллизий).

---

## 2. МЁРТВЫЙ КОД

### 2.1 StringRegistry не используется ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/abilities/string_registry.rs`

**Решение:** Удалить `string_registry.rs`. Реестры достаточно разные (разные дополнительные данные), унификация не даёт выгоды, для проекта такого размера это over-engineering.

**Проблема:** Generic `StringRegistry<T>` реализует паттерн name↔id маппинга, но все реестры делают это вручную:

- `ActivatorRegistry` - свои `name_to_id`, `id_to_name`
- `EffectRegistry` - свои `name_to_id`, `id_to_name`
- `AbilityRegistry` - свои `name_to_id`, `next_id`
- `StatRegistry` - свои `name_to_id`, `stats`

**Аргументы:**
- 53 строки неиспользуемого кода
- Дублирование логики в каждом реестре
- StringRegistry хорошо написан, но забыт

**Рекомендация:** Либо рефакторить все реестры использовать `StringRegistry<T>`, либо удалить файл.

---

### 2.2 UseAbilities компонент без системы ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/fsm/behaviour/use_abilities.rs`

**Решение:** Реализовать `use_abilities_system`, которая будет обрабатывать компонент UseAbilities и заставлять мобов использовать способности.

**Проблема:** Компонент создаётся при FSM переходах:
```rust
// systems.rs:93-94
BehaviourDef::UseAbilities(abilities) => {
    entity_commands.insert(UseAbilities::new(abilities.clone()));
}
```

Но нет ни одной системы, которая бы его обрабатывала. В `FsmPlugin` зарегистрированы:
- `move_toward_player_system`
- `when_near_system`
- `after_time_system`
- `fsm_transition_system`

Нет `use_abilities_system`.

**Аргументы:**
- Мобы с UseAbilities поведением не используют способности
- Компонент добавляется и удаляется без эффекта
- Введение в заблуждение - выглядит как работающая функциональность

**Рекомендация:** Реализовать систему или удалить BehaviourDef::UseAbilities.

---

### 2.3 Неиспользуемая переменная wanted_ability ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/abilities/dispatcher.rs:24`

**Решение:** Использовать `wanted_ability` для оптимизации - не итерировать все способности, а сразу получать нужную по ID:
```rust
if let Some(wanted_id) = input.want_to_cast {
    if let Some(ability_instance) = abilities.get_mut(wanted_id) {
        // проверяем только одну способность
    }
}
```

**Проблема:**
```rust
let wanted_ability = input.want_to_cast;  // присваивается

for ability_instance in abilities.list.iter_mut() {
    // wanted_ability нигде не используется
    // проверяются ВСЕ способности, независимо от желаемой
}
```

**Аргументы:**
- Компилятор должен давать warning (возможно подавлен)
- Неэффективно - проверяются все способности вместо одной
- Работает только потому что OnInputActivator сам проверяет `want_to_cast == ability_id`

---

## 3. АРХИТЕКТУРНЫЕ ПРОБЛЕМЫ

### 3.1 Неконсистентность data-driven подхода ✅ ПОДТВЕРЖДЕНО

**Проблема:** Мобы загружаются из .ron файлов, но способности и игрок захардкожены.

**Решение:** Незавершённая работа - доделать загрузку способностей из RON файлов:
1. Создать `assets/abilities/fireball.ron`
2. Написать `load_abilities()` по аналогии с `load_mobs()`
3. Удалить `create_fireball_ability()` из player.rs

`AbilityDefRaw` с serde уже существует, нужен только loader.

**Мобы (правильно):**
```
assets/mobs/slime.ron → MobDef → spawn_mob()
```

**Способности (неправильно):**
```rust
// player.rs:84-131
fn create_fireball_ability(...) {
    // 50 строк хардкода
    let fireball_id = ability_registry.allocate_id("fireball");
    let activator_type = activator_registry.get_id("on_input").unwrap();
    // ...
}
```

**Аргументы:**
- Для изменения способности нужна перекомпиляция
- Невозможен hot-reload способностей
- Дизайнер/моддер не может добавлять способности без программиста
- Уже есть `AbilityDefRaw` с serde - всё готово для загрузки из файлов

---

### 3.2 Хардкод базовых статов мобов ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/fsm/spawn.rs:41-46`

**Решение:** Добавить `base_stats: HashMap<String, f32>` в MobDef, загружать из .ron файлов, применять при спавне.

```rust
if let Some(id) = stat_registry.get("movement_speed_base") {
    modifiers.add(id, 100.0, None);  // магическое число
}
if let Some(id) = stat_registry.get("max_life_base") {
    modifiers.add(id, 50.0, None);   // магическое число
}
```

**Аргументы:**
- Все мобы имеют одинаковые статы (100 скорости, 50 здоровья)
- MobDef уже содержит abilities, visual, states - но не base_stats
- Невозможно создать быстрого или медленного моба без изменения кода

**Рекомендация:** Добавить в MobDef:
```rust
#[serde(default)]
pub base_stats: HashMap<String, f32>,
```

---

### 3.3 Жёсткая связь FSM с Player ✅ ПОДТВЕРЖДЕНО

**Файлы:** `src/fsm/transitions/when_near.rs`, `src/fsm/behaviour/move_toward_player.rs`

**Решение:** Выделить отдельный модуль для конкретных реализаций, FSM оставить чистым ядром:

```
src/
  fsm/                      # ядро - состояния, переходы, события
    components.rs
    events.rs
    systems.rs              # только fsm_transition_system
    registry.rs
    types.rs

  mob_ai/                   # конкретные реализации (новый модуль)
    behaviours/
      move_toward_player.rs
      use_abilities.rs
    transitions/
      when_near_player.rs
      after_time.rs
```

FSM ядро не знает про Player и конкретику. Модуль mob_ai зависит от всего что нужно, может содержать как гибкие, так и хардкоженные поведения (для боссов и т.п.).

**Проблема:**
```rust
pub fn when_near_system(
    query: Query<(Entity, &WhenNear, &Transform)>,
    player: Query<&Transform, With<Player>>,  // жёсткая зависимость
    ...
)
```

**Аргументы:**
- FSM модуль зависит от Player компонента из другого модуля
- Нарушает принцип инверсии зависимостей
- Невозможно использовать FSM для мобов, преследующих другие цели

---

### 3.4 Отсутствие системы здоровья/урона ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/abilities/effects/damage.rs`

**Решение:** Реализовать систему здоровья:
1. `Health { current: f32 }` - отдельный компонент (состояние)
2. `max_life` - остаётся статом в ComputedStats (характеристика)
3. При спавне: `Health { current: stats.get(max_life_id) }`
4. `damage_system` - query по Health, применяет урон напрямую
5. `death_system` - проверяет `health.current <= 0`

Разделение: статы = "что я могу", Health = "что со мной сейчас".

**Проблема:**
```rust
impl EffectExecutor for DamageEffect {
    fn execute(&self, ...) {
        let amount = ...;
        info!("Damage effect: {} damage", amount);  // только лог!
    }
}
```

А в `projectile_collision`:
```rust
if distance < threshold {
    commands.entity(mob_entity).despawn();  // мгновенная смерть
}
```

**Аргументы:**
- DamageEffect бесполезен - урон никуда не применяется
- Мобы умирают от любого попадания независимо от урона
- Невозможны: броня, сопротивления, иммунитет, регенерация
- Нет визуального фидбека о количестве урона

---

### 3.5 Слишком много зависимостей в spawn_player ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/player.rs:27-82`

**Решение:** Создать конфиг игрока `assets/player.ron`:

```rust
pub struct PlayerDef {
    pub base_stats: HashMap<String, f32>,
    pub abilities: Vec<String>,
    pub visual: VisualDef,      // цвет, размер
    pub collider: ColliderDef,  // радиус коллизии
}
```

Пример `assets/player.ron`:
```ron
(
    base_stats: {
        "strength_base": 10.0,
        "max_life_base": 100.0,
        "movement_speed_base": 400.0,
        "physical_damage_base": 10.0,
        "projectile_speed_base": 800.0,
        "crit_chance_base": 0.05,
        "crit_multiplier_base": 1.5,
    },
    abilities: ["fireball"],
    visual: ( size: 100.0, color: [0.2, 0.6, 1.0] ),
    collider: ( radius: 50.0 ),
)
```

После реализации 3.1 (загрузка способностей) и этого конфига, spawn_player упростится до `commands` + `stat_registry` + `player_def`.

**Проблема:**
```rust
fn spawn_player(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    calculators: Res<StatCalculators>,
    mut ability_registry: ResMut<AbilityRegistry>,
    activator_registry: Res<ActivatorRegistry>,
    mut effect_registry: ResMut<EffectRegistry>,
) {
    // 55 строк инициализации
}
```

**Аргументы:**
- 6 ресурсов-зависимостей (calculators даже не используется)
- Нарушает Single Responsibility - спавн + создание способностей + настройка статов
- Хардкод статов и визуала в коде

---

## 4. ПОТЕНЦИАЛЬНЫЕ БАГИ

### 4.1 Сравнение float с нулём через == ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/expression/mod.rs:96`

**Решение:** Заменить `divisor == 0.0` на `divisor.abs() < f32::EPSILON`.

```rust
if divisor == 0.0 {  // неправильно
```

В `abilities/expression.rs:34` сделано правильно:
```rust
if divisor.abs() < f32::EPSILON {  // правильно
```

**Аргументы:**
- Из-за погрешности float, `0.1 + 0.2 - 0.3` может не равняться `0.0`
- Может привести к делению на очень маленькое число → огромный результат

---

### 4.2 Рекурсивная топологическая сортировка ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/stats/calculators.rs:70-88`

**Решение:** Переписать на итеративный алгоритм (Kahn's algorithm) с `Result`. Главное - при обнаружении цикла возвращать понятную ошибку с цепочкой зависимостей, а не просто panic.

```rust
fn topological_sort(&self) -> Result<Vec<StatId>, CycleError> {
    // Kahn's algorithm
    // При ошибке: Err(CycleError { cycle: ["stat_a", "stat_b", "stat_a"] })
}
```

**Проблема:**
```rust
fn visit(&self, stat: StatId, visited: &mut HashMap<StatId, bool>, result: &mut Vec<StatId>) {
    // рекурсивный вызов
    self.visit(dep, visited, result);
}
```

**Аргументы:**
- panic при циклах не информативен - не показывает цепочку зависимостей
- Сложно дебажить какие статы образуют цикл

---

### 4.3 Несогласованность типов в ActivatorState ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/abilities/activator_def.rs`

**Решение:** Использовать `ParamId` вместо `String` в ActivatorState:
```rust
pub struct ActivatorState {
    pub params: HashMap<ParamId, f32>,
}
```

**Проблема:**
```rust
pub struct ActivatorDef {
    pub params: HashMap<ParamId, ParamValue>,  // ParamId
}

pub struct ActivatorState {
    pub params: HashMap<String, f32>,  // String!
}
```

**Аргументы:**
- Опечатка в строке приведёт к runtime багу (получим 0.0 без ошибки)
- ParamId был создан именно чтобы избежать строк

---

## 5. ПРОИЗВОДИТЕЛЬНОСТЬ

### 5.1 Клонирование stats каждый кадр ⏳ ПРОВЕРИТЬ ПОСЛЕ 2.3

**Файл:** `src/abilities/context.rs:39`

**Статус:** После реализации пункта 2.3 (оптимизация dispatcher) убедиться, что снапшот создаётся только при реальном касте, а не для каждой способности каждый кадр.

```rust
stats_snapshot: Arc::new(stats.clone()),
```

Вызывается в `ability_dispatcher` для каждой сущности каждый кадр.

**Аргументы:**
- ComputedStats содержит `Vec<f32>` - клонирование аллоцирует память
- Arc::new тоже аллокация
- При 100 сущностях = 100 клонов + 100 Arc каждый кадр

**Изначальная задумка:** снапшот копируется только при касте заклинания и используется для всей цепочки событий этого инстанса каста.

---

### 5.2 Линейный поиск способностей ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/abilities/components.rs:35-41`

**Решение:** Заменить `Vec<AbilityInstance>` на `HashMap<AbilityId, AbilityInstance>` для O(1) доступа.

```rust
pub fn get(&self, ability_id: AbilityId) -> Option<&AbilityInstance> {
    self.list.iter().find(|a| a.def_id == ability_id)  // O(n)
}
```

**Аргументы:**
- При 10 способностях = 10 сравнений на каждый вызов
- Вызывается в циклах

---

## 6. СТИЛЬ КОДА

### 6.1 println! вместо bevy_log ✅ ПОДТВЕРЖДЕНО

**Файлы:** `src/fsm/loader.rs:29`, `src/fsm/spawn.rs:64`, `src/fsm/systems.rs:42-45`

**Решение:** Заменить `println!`/`eprintln!` на `info!`/`warn!`/`error!` из bevy::prelude.

```rust
println!("Loaded mob: {}", mob_def.name);
eprintln!("Failed to parse mob file {:?}: {}", file_path, e);
```

---

### 6.2 expect/panic вместо Result ✅ ПОДТВЕРЖДЕНО

**Файл:** `src/stats/loader.rs`

**Решение:** Panic допустим для обязательных файлов, но улучшить сообщение об ошибке:
```rust
.unwrap_or_else(|e| {
    panic!(
        "\n\n=== FATAL ERROR ===\n\
         Failed to load required file: {}\n\
         Error: {}\n\
         Make sure the game is installed correctly.\n\n",
        stat_ids_path, e
    )
})
```

**Текущий код:**
```rust
.expect(&format!("Failed to read stat_ids file: {}", stat_ids_path))
```

**Аргументы:**
- Пользователь видит panic backtrace вместо понятной ошибки

---

## ПРИОРИТЕТЫ ИСПРАВЛЕНИЯ

**Высокий приоритет (влияет на архитектуру):**
1. Удалить `bullet.rs` - дублирование
2. Data-driven способности - консистентность
3. Система здоровья/урона - базовый геймплей

**Средний приоритет (качество кода):**
4. Объединить Expression/StatExpression
5. Базовые статы в MobDef
6. Реализовать UseAbilities или удалить

**Низкий приоритет (улучшения):**
7. StringRegistry - использовать или удалить
8. FSM независимость от Player
9. Асинхронная загрузка ресурсов
10. Производительность (Arc, поиск)
