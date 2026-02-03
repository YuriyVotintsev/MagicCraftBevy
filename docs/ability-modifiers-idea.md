# Идея: Модификация заклинаний в рантайме

## Проблема

Модификатор типа "Дополнительный взрыв на каждом OnHit" должен:
- Добавлять эффект к любому заклинанию с OnHit
- Сниматься при смене экипировки
- Не ломать базовое заклинание

## Решение: Модификаторы как слой поверх AbilityDef

**Принцип:** AbilityDef остаётся неизменным. Модификаторы применяются в момент спавна.

```rust
#[derive(Component)]
pub struct AbilityModifiers {
    modifiers: Vec<AbilityModifier>,
}

pub struct AbilityModifier {
    pub id: ModifierId,
    pub source: Entity,              // шмотка которая дала модификатор
    pub trigger: ModifierTrigger,    // когда применять
    pub entities: Vec<EntityDef>,    // что добавлять
}

pub enum ModifierTrigger {
    OnSpawn,      // при спавне основной entity
    OnCollision,  // при срабатывании OnCollision
    OnHit,        // при срабатывании OnHit
    OnKill,
}
```

## Flow

**1. Экипировка надета:**
```rust
commands.entity(player).get::<AbilityModifiers>().add(
    AbilityModifier {
        source: item_entity,
        trigger: ModifierTrigger::OnHit,
        entities: vec![
            EntityDef { components: vec![
                ComponentDef::Explosion(50.0),
                ComponentDef::DamagePayload(20.0),
            ]}
        ],
    }
);
```

**2. При срабатывании триггера:**
```rust
fn process_trigger(...) {
    // Спавним базовые entities из триггера
    for entity_def in &trigger.entities {
        registry.spawn_entity(commands, entity_def, &ctx);
    }

    // Проверяем модификаторы на caster'е
    if let Ok(mods) = modifiers_query.get(source.caster) {
        for modifier in mods.matching(ModifierTrigger::OnHit) {
            for entity_def in &modifier.entities {
                registry.spawn_entity(commands, entity_def, &ctx);
            }
        }
    }
}
```

**3. Экипировка снята:**
```rust
commands.entity(player).get::<AbilityModifiers>()
    .remove_by_source(item_entity);
```

## Примеры модификаторов

**"Взрыв на каждом OnHit":**
```rust
AbilityModifier {
    trigger: ModifierTrigger::OnHit,
    entities: vec![EntityDef {
        components: vec![Explosion(50.0), DamagePayload(20.0)]
    }],
}
```

**"Добавить Pierce(3) ко всем снарядам":**
```rust
AbilityModifier {
    trigger: ModifierTrigger::OnSpawn,
    condition: HasComponent::Projectile,
    add_components: vec![ComponentDef::Pierce(3)],
}
```

**"Chain Lightning":**
```rust
AbilityModifier {
    trigger: ModifierTrigger::OnHit,
    condition: AbilityId("lightning"),
    entities: vec![/* template молнии с ChainLevel */],
}
```

## Преимущества

1. **AbilityDef неизменен** - базовое заклинание всегда одинаковое
2. **Легко снять** - удаляем модификатор по source
3. **Стекается** - несколько шмоток добавляют модификаторы
4. **Гибко** - модификатор добавляет любые компоненты/entities
5. **Совместимо с component-based архитектурой**

## Альтернатива: Модификаторы как компоненты

Для уникальных эффектов - модификатор это компонент на caster'е:

```rust
#[derive(Component)]
pub struct ExplosionOnHit {
    pub radius: f32,
    pub damage: f32,
}

fn on_hit_explosion_system(
    hit_events: EventReader<HitEvent>,
    caster_query: Query<&ExplosionOnHit>,
) {
    for event in hit_events.read() {
        if let Ok(modifier) = caster_query.get(event.caster) {
            // spawn explosion
        }
    }
}
```

Менее data-driven, но проще для уникальных эффектов.
