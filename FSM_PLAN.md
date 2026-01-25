# FSM System for Mobs

## Итоговый формат RON

```ron
(
    name: "slime",
    abilities: ["slime_attack"],
    visual: (
        shape: Circle,          // Circle, Rectangle, Triangle
        size: 30.0,
        color: (0.2, 0.8, 0.2), // RGB 0.0-1.0, tuple в RON
    ),
    initial_state: "chase",
    states: {
        "chase": (
            behaviour: [MoveTowardPlayer],
            transitions: [WhenNear([("attack", 50.0)])],
        ),
        "attack": (
            behaviour: [UseAbilities(["slash"]), PlaySound("attack")],
            transitions: [AfterTime("chase", 1.0)],
        ),
    }
)
```

- `behaviour` — компоненты, которые добавляются при входе в состояние и убираются при выходе
- `transitions` — компоненты-чекеры, которые проверяют условия перехода
- Каждый тип (MoveTowardPlayer, WhenNear, etc.) — отдельный struct с своей системой

---

## Архитектура

### Структура файлов

```
src/fsm/
├── mod.rs           # FsmPlugin, реэкспорты
├── types.rs         # MobDef, StateDef, StateId
├── components.rs    # CurrentState, MobType
├── events.rs        # StateTransition event
├── registry.rs      # MobRegistry (загруженные определения)
├── loader.rs        # load_mobs() из assets/mobs/*.ron
├── systems.rs       # fsm_transition_system
├── behaviour/       # Типы поведений
│   ├── mod.rs
│   ├── move_toward_player.rs
│   └── use_abilities.rs
└── transitions/     # Типы переходов
    ├── mod.rs
    ├── when_near.rs
    └── after_time.rs

assets/mobs/
├── slime.ron
└── goblin.ron
```

---

## Ядро системы

### 1. Типы данных (types.rs)

```rust
#[derive(Deserialize)]
pub struct MobDef {
    pub name: String,
    pub abilities: Vec<String>,
    pub visual: VisualDef,
    pub initial_state: String,
    pub states: HashMap<String, StateDef>,
}

#[derive(Deserialize)]
pub struct VisualDef {
    pub shape: Shape,
    pub size: f32,
    pub color: [f32; 3],
}

#[derive(Deserialize)]
pub enum Shape {
    Circle,
    Rectangle,
    Triangle,
}

#[derive(Deserialize)]
pub struct StateDef {
    pub behaviour: Vec<Box<dyn Reflect>>,    // Десериализуется через TypeRegistry
    pub transitions: Vec<Box<dyn Reflect>>,
}
```

### 2. Компоненты (components.rs)

```rust
#[derive(Component)]
pub struct CurrentState(pub String);

#[derive(Component)]
pub struct MobType(pub String);  // Ссылка на MobDef в MobRegistry
```

### 3. События (events.rs)

```rust
#[derive(Event)]
pub struct StateTransition {
    pub entity: Entity,
    pub to: String,
}
```

### 4. Registry (registry.rs)

```rust
#[derive(Resource)]
pub struct MobRegistry {
    pub mobs: HashMap<String, MobDef>,
}
```

### 5. Системы (systems.rs)

**fsm_transition_system:**
1. Читает события StateTransition
2. Для каждого перехода:
   - Получает текущий StateDef из MobRegistry
   - Убирает компоненты behaviour и transitions текущего состояния
   - Меняет CurrentState
   - Добавляет компоненты behaviour и transitions нового состояния

---

## Behaviour и Transitions

### Паттерн добавления нового типа

**Behaviour (например MoveTowardPlayer):**

```rust
// behaviour/move_toward_player.rs
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MoveTowardPlayer;

pub fn move_toward_player_system(
    time: Res<Time>,
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut Transform, &ComputedStats), With<MoveTowardPlayer>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let speed_id = stat_registry.get("movement_speed");

    for (mut transform, stats) in &mut query {
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or(100.0);

        let direction = (player_pos - transform.translation).truncate();
        if direction.length_squared() > 1.0 {
            let normalized = direction.normalize();
            transform.translation.x += normalized.x * speed * time.delta_secs();
            transform.translation.y += normalized.y * speed * time.delta_secs();
        }
    }
}
```

**Transition (например WhenNear):**

```rust
// transitions/when_near.rs
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WhenNear(pub Vec<(String, f32)>);  // [(target_state, distance)]

pub fn when_near_system(
    query: Query<(Entity, &WhenNear, &Transform)>,
    player: Query<&Transform, With<Player>>,
    mut events: EventWriter<StateTransition>,
) {
    let player_pos = player.single().translation;
    for (entity, when_near, transform) in &query {
        let dist = transform.translation.distance(player_pos);
        for (state, threshold) in &when_near.0 {
            if dist < *threshold {
                events.send(StateTransition { entity, to: state.clone() });
                break;
            }
        }
    }
}
```

**Transition (например AfterTime):**

```rust
// transitions/after_time.rs
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AfterTime {
    pub target: String,
    pub duration: f32,
    pub elapsed: f32,  // Таймер внутри компонента
}

impl AfterTime {
    pub fn new(target: String, duration: f32) -> Self {
        Self { target, duration, elapsed: 0.0 }
    }
}

pub fn after_time_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut AfterTime)>,
    mut events: EventWriter<StateTransition>,
) {
    for (entity, mut after_time) in &mut query {
        after_time.elapsed += time.delta_secs();
        if after_time.elapsed >= after_time.duration {
            events.send(StateTransition { entity, to: after_time.target.clone() });
        }
    }
}
```

Таймер живёт внутри компонента AfterTime. При добавлении компонента elapsed = 0, при смене состояния компонент удаляется.

### Регистрация типов

```rust
// В FsmPlugin
app.register_type::<MoveTowardPlayer>();
app.register_type::<UseAbilities>();
app.register_type::<WhenNear>();
app.register_type::<AfterTime>();

app.add_systems(Update, (
    move_toward_player_system,
    use_abilities_system,
    when_near_system,
    after_time_system,
    fsm_transition_system,  // После всех transition систем
));
```

---

## Загрузка из RON (loader.rs)

```rust
pub fn load_mobs(type_registry: &TypeRegistry) -> MobRegistry {
    let mut mobs = HashMap::new();

    for entry in fs::read_dir("assets/mobs").unwrap() {
        let path = entry.unwrap().path();
        let content = fs::read_to_string(&path).unwrap();

        // Десериализация с TypeRegistry для Reflect типов
        let mob_def: MobDef = ron::Options::default()
            .with_type_registry(type_registry)
            .from_str(&content)
            .unwrap();

        mobs.insert(mob_def.name.clone(), mob_def);
    }

    MobRegistry { mobs }
}
```

---

## Интеграция с AbilitySystem

**UseAbilities behaviour:**

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct UseAbilities(pub Vec<String>);

pub fn use_abilities_system(
    query: Query<(Entity, &UseAbilities)>,
    ability_registry: Res<AbilityRegistry>,
    mut ability_input: Query<&mut AbilityInput>,
) {
    for (entity, use_abilities) in &query {
        for ability_name in &use_abilities.0 {
            // Активировать способность через существующую систему
            if let Ok(mut input) = ability_input.get_mut(entity) {
                input.want_to_cast = Some(ability_name.clone());
            }
        }
    }
}
```

---

## Спавн моба

```rust
pub fn spawn_mob(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    mob_registry: &MobRegistry,
    stat_registry: &StatRegistry,
    mob_name: &str,
    position: Vec3,
) -> Entity {
    let mob_def = mob_registry.mobs.get(mob_name).unwrap();

    // Visual
    let mesh = match mob_def.visual.shape {
        Shape::Circle => meshes.add(Circle::new(mob_def.visual.size)),
        Shape::Rectangle => meshes.add(Rectangle::new(mob_def.visual.size, mob_def.visual.size)),
        Shape::Triangle => meshes.add(RegularPolygon::new(mob_def.visual.size, 3)),
    };
    let color = Color::srgb(
        mob_def.visual.color[0],
        mob_def.visual.color[1],
        mob_def.visual.color[2],
    );

    // Stats — базовые модификаторы моба
    let mut modifiers = Modifiers::new();
    let mut dirty = DirtyStats::default();

    // Пометить все статы грязными для первичного расчёта
    for i in 0..stat_registry.len() {
        dirty.mark(StatId(i as u32));
    }

    // Установить базовые статы моба (можно вынести в mob_def.stats)
    if let Some(id) = stat_registry.get("movement_speed_base") {
        modifiers.add(id, 100.0, None);  // Базовая скорость моба
    }
    if let Some(id) = stat_registry.get("max_life_base") {
        modifiers.add(id, 50.0, None);   // Базовое здоровье моба
    }

    let entity = commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(position),
        MobType(mob_name.to_string()),
        CurrentState(mob_def.initial_state.clone()),
        modifiers,
        ComputedStats::new(stat_registry.len()),
        dirty,
        // Abilities добавляются отдельно через AbilityRegistry
    )).id();

    // Добавить компоненты начального состояния
    add_state_components(commands, entity, mob_def, &mob_def.initial_state);

    entity
}
```

Система `recalculate_stats` (из существующего StatsPlugin) автоматически пересчитает статы
для всех entities с `DirtyStats` в `PreUpdate`, включая мобов.

---

## Порядок реализации

1. **Ядро:** types.rs, components.rs, events.rs, registry.rs
2. **Системы:** systems.rs (fsm_transition_system)
3. **Loader:** loader.rs
4. **Первые типы:** MoveTowardPlayer, WhenNear, AfterTime
5. **Интеграция:** UseAbilities с AbilityRegistry
6. **Plugin:** FsmPlugin, регистрация всего
7. **Тест:** slime.ron, спавн и проверка работы

---

## Верификация

1. Создать `assets/mobs/slime.ron` с тестовым мобом
2. Спавнить slime в ArenaPlugin
3. Проверить:
   - Моб в состоянии "chase" движется к игроку
   - При приближении переходит в "attack"
   - После атаки возвращается в "chase"
4. Логирование переходов для отладки
