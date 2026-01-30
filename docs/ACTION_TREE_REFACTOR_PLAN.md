# План: Объединение Activators и Effects в Action Tree

## Цель
Устранить дублирование между activators и effects, создав единую систему Action Tree с древовидной структурой узлов.

## Текущие проблемы

| Дублирование | Файлы |
|--------------|-------|
| ActivatorRegistry ≈ EffectRegistry | registry.rs (90% копипаста) |
| ActivatorDef ≈ EffectDef | activator_def.rs, effect_def.rs |
| register_activator! ≈ register_effect! | macros.rs |
| resolve_activator_def ≈ resolve_effect_def | loading/systems.rs |

**Проблема on_hit:** Это фактически вложенный триггер внутри params, а не отдельная сущность.

## Целевой формат .ability.ron

```ron
(
    id: "fireball",
    action: ("on_input", [
        ("spawn_projectile", { "speed": Float(800.0) }, [
            ("on_hit", [
                ("damage", { "amount": Stat("physical_damage") }),
            ]),
        ]),
    ]),
)
```

Варианты синтаксиса через `#[serde(untagged)]`:
- `("type", { params }, [ children ])` — Full
- `("type", { params })` — WithParams
- `("type", [ children ])` — WithChildren
- `"type"` — Simple

## Архитектура

### ActionDefRaw (для десериализации)
```rust
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ActionDefRaw {
    Full(String, HashMap<String, ParamValueRaw>, Vec<ActionDefRaw>),
    WithParams(String, HashMap<String, ParamValueRaw>),
    WithChildren(String, Vec<ActionDefRaw>),
    Simple(String),
}
```

### ActionDef (типизированный)
```rust
pub struct ActionDef {
    pub action_type: ActionTypeId,
    pub params: HashMap<ParamId, ParamValue>,
    pub children: Vec<ActionDef>,
}
```

### ActionHandler trait
```rust
pub trait ActionHandler: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    /// Для триггеров: добавляет компоненты на entity
    fn setup(&self, commands: &mut Commands, entity: Entity,
             def: &ActionDef, registry: &ActionRegistry) {}

    /// Для исполнителей: выполняет действие
    fn execute(&self, def: &ActionDef, ctx: &ActionContext,
               commands: &mut Commands, registry: &ActionRegistry) {}

    fn register_systems(&self, _app: &mut App) {}
}
```

### ActionRegistry (один вместо двух)
```rust
pub struct ActionRegistry {
    name_to_id: HashMap<String, ActionTypeId>,
    handlers: Vec<Box<dyn ActionHandler>>,
    param_name_to_id: HashMap<String, ParamId>,
    // ...
}
```

## Файлы для изменения

### Новые файлы
| Файл | Описание |
|------|----------|
| `src/abilities/action_def.rs` | ActionDef, ActionDefRaw |
| `src/abilities/action_registry.rs` | ActionRegistry, ActionHandler |
| `src/abilities/actions/mod.rs` | collect_actions! |
| `src/abilities/actions/on_hit.rs` | OnHitHandler (новый триггер) |

### Миграция существующих
| Файл | Изменения |
|------|-----------|
| `src/abilities/activators/*.rs` | impl ActionHandler, setup() |
| `src/abilities/effects/*.rs` | impl ActionHandler, execute() |
| `src/abilities/macros.rs` | Добавить register_action!, collect_actions! |
| `src/abilities/ability_def.rs` | action: ActionDef вместо activator + effects |
| `src/loading/systems.rs` | resolve_action_def() |
| `assets/abilities/*.ron` | Новый формат |

### Удалить после миграции
- `src/abilities/activator_def.rs`
- `src/abilities/activators/` (переместить в actions/)
- `src/abilities/effects/` (переместить в actions/)
- Старый код из registry.rs

## Порядок выполнения

### Этап 1: Новые типы (не ломает существующий код)
1. Создать `action_def.rs` с ActionDefRaw, ActionDef
2. Создать `action_registry.rs` с ActionRegistry, ActionHandler
3. Добавить макросы register_action!, collect_actions!

### Этап 2: Миграция handlers
4. Создать `src/abilities/actions/` директорию
5. Мигрировать activators → actions (impl ActionHandler с setup())
6. Мигрировать effects → actions (impl ActionHandler с execute())
7. Создать on_hit.rs как явный триггер

### Этап 3: Интеграция
8. Обновить ability_def.rs (action вместо activator + effects)
9. Обновить loading/systems.rs (resolve_action_def)
10. Обновить AbilityPlugin

### Этап 4: Миграция данных
11. Конвертировать .ability.ron файлы в новый формат

### Этап 5: Очистка
12. Удалить старые файлы и код

## Ключевые изменения в spawn_projectile

```rust
fn execute(&self, def: &ActionDef, ctx: &ActionContext, ...) {
    // Найти on_hit children и извлечь их children как эффекты
    let on_hit_actions: Vec<ActionDef> = def.children
        .iter()
        .filter(|c| registry.get_name(c.action_type) == Some("on_hit"))
        .flat_map(|on_hit| on_hit.children.clone())
        .collect();

    commands.spawn((
        Projectile { on_hit_actions, context: ctx.clone() },
        // ...
    ));
}
```

## Примеры миграции .ron файлов

### fireball.ability.ron
```ron
// БЫЛО:
(
    id: "fireball",
    activator: (activator_type: "on_input", params: {}),
    effects: [(
        effect_type: "spawn_projectile",
        params: { "on_hit": EffectList([(effect_type: "damage", ...)]) },
    )],
)

// СТАНЕТ:
(
    id: "fireball",
    action: ("on_input", [
        ("spawn_projectile", [
            ("on_hit", [
                ("damage", { "amount": Stat("physical_damage") }),
            ]),
        ]),
    ]),
)
```

### meteor.ability.ron
```ron
// СТАНЕТ:
(
    id: "meteor",
    action: ("interval", { "interval": Float(3.0) }, [
        ("spawn_meteor", { "search_radius": Float(700.0) }, [
            ("on_hit", [
                ("damage", { "amount": Stat("physical_damage") }),
            ]),
        ]),
    ]),
)
```

## Верификация

```bash
cargo build
cargo run --features headless -- --timeout 30
```

Проверить:
- fireball (on_input → spawn_projectile → on_hit → damage)
- flamethrower (while_held → spawn_projectile → on_hit → damage)
- meteor (interval → spawn_meteor → on_hit → damage)
- dash (on_input → dash)
- shield (on_input → shield)
- orbiting_orbs (every_frame → spawn_orbiting → on_hit → damage)
