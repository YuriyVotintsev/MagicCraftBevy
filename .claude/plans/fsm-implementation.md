# FSM System for Mobs - Implementation Plan

## Порядок реализации

1. [ ] **Ядро:** types.rs, components.rs, events.rs, registry.rs
2. [ ] **Системы:** systems.rs (fsm_transition_system)
3. [ ] **Loader:** loader.rs
4. [ ] **Первые типы:** MoveTowardPlayer, WhenNear, AfterTime
5. [ ] **Интеграция:** UseAbilities с AbilityRegistry
6. [ ] **Plugin:** FsmPlugin, регистрация всего
7. [ ] **Тест:** slime.ron, спавн и проверка работы

---

## Структура файлов

```
src/fsm/
├── mod.rs           # FsmPlugin, реэкспорты
├── types.rs         # MobDef, StateDef, VisualDef, Shape
├── components.rs    # CurrentState, MobType
├── events.rs        # StateTransition event
├── registry.rs      # MobRegistry
├── loader.rs        # load_mobs()
├── systems.rs       # fsm_transition_system
├── spawn.rs         # spawn_mob()
├── behaviour/
│   ├── mod.rs
│   └── move_toward_player.rs
└── transitions/
    ├── mod.rs
    ├── when_near.rs
    └── after_time.rs

assets/mobs/
└── slime.ron
```

---

## Итоговый формат RON

```ron
(
    name: "slime",
    abilities: ["slime_attack"],
    visual: (
        shape: Circle,
        size: 30.0,
        color: [0.2, 0.8, 0.2],
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

---

## Детали реализации

### types.rs

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
    pub behaviour: Vec<Box<dyn Reflect>>,
    pub transitions: Vec<Box<dyn Reflect>>,
}
```

### components.rs

```rust
#[derive(Component)]
pub struct CurrentState(pub String);

#[derive(Component)]
pub struct MobType(pub String);
```

### events.rs

```rust
#[derive(Event)]
pub struct StateTransition {
    pub entity: Entity,
    pub to: String,
}
```

### MoveTowardPlayer

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MoveTowardPlayer;

pub fn move_toward_player_system(
    time: Res<Time>,
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut Transform, &ComputedStats), With<MoveTowardPlayer>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player.single() else { return };
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

### WhenNear

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WhenNear(pub Vec<(String, f32)>);

pub fn when_near_system(
    query: Query<(Entity, &WhenNear, &Transform)>,
    player: Query<&Transform, With<Player>>,
    mut events: EventWriter<StateTransition>,
) {
    let Ok(player_transform) = player.single() else { return };
    let player_pos = player_transform.translation;
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

### AfterTime

```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AfterTime {
    pub target: String,
    pub duration: f32,
    pub elapsed: f32,
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

---

## Верификация

1. Создать `assets/mobs/slime.ron`
2. Спавнить slime в ArenaPlugin
3. Проверить:
   - Моб в "chase" движется к игроку
   - При приближении переходит в "attack"
   - После атаки возвращается в "chase"
