# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Magic Craft is a Bevy 0.18 roguelite arena shooter with a Path-of-Exile-inspired ability modification system. Players face waves of enemies and customize abilities through a deep spell-crafting system.

**Tech Stack:** Bevy 0.18, Avian2D 0.5 (physics), Ron (config format), Serde

## Build Commands

```bash
cargo run                                      # Normal mode
cargo run --features headless -- --timeout 10  # Headless mode (quick test, 10 seconds is enough)
```

**Testing:**
- Use headless mode with `--timeout 10` for quick validation (10 seconds is sufficient)
- Longer tests are unnecessary for basic functionality verification

## Architecture

### Plugin-Based Structure
Each major system is a self-contained Bevy Plugin registered in `main.rs`:
- `BlueprintPlugin` - Data-driven entity definition system (most complex)
- `StatsPlugin` - Stat calculation with modifiers
- `FsmPlugin` - Finite state machine for mob AI
- `MobAiPlugin` - Concrete behavior implementations
- `PlayerPlugin`, `WavePlugin`, `UiPlugin`, etc.

### State Management
```rust
GameState { Loading, MainMenu, Playing, GameOver }
WavePhase { Combat, ShopDelay, Shop }  // SubState of Playing
```
Systems use `run_if(in_state(...))` for state gating.

### System Ordering (schedule.rs)
```rust
GameSet { Input, MobAI, AbilityActivation, AbilityExecution, Damage, WaveManagement }
```
Use `in_set(GameSet::X)` instead of `before()`/`after()`.
Create new set if required to avoid `before()`/`after()`.

### Data-Driven Design
All game content loads from `.ron` files in `assets/`:
- `abilities/*.ability.ron` - Ability definitions
- `mobs/*.mob.ron` - Mob AI states and behaviors
- `player.player.ron` - Player configuration
- `stats/config.stats.ron` - Stat definitions and formulas

Custom asset loaders implement Bevy's `AssetLoader` trait.

### Trait-Based Registries
Abilities use trait registries for extensibility:
- `Trigger` trait → `TriggerRegistry` (when abilities trigger)
- `EffectExecutor` trait → `EffectRegistry` (what abilities do)
- `BehaviourRegistry`, `TransitionRegistry` for FSM

### Key Patterns
- **SpawnSource**: Context data passed during entity spawning
- **PendingDamage**: Queued damage applied in GameSet::Damage
- **DirtyStats**: Tracks which stats need recalculation (optimization)
- **Raw→Processed types**: `*Raw` types use strings for Ron deserialization, converted to typed IDs at load time

### Blueprint Component Design Rules

**1. RON name = ECS component (1:1 mapping)**
```
RON файл              →  ECS Component
──────────────────────────────────────
Speed((value: "400")) →  Speed { value: f32 }
Straight(())          →  Straight { spread: f32, direction: Vec2 }
Collider((shape: Circle)) →  Collider { shape: Shape }
OnCollision((entities: [...])) →  OnCollision { entities: Vec<EntityDef> }
```

**2. Use `#[blueprint_component]` macro**

The proc macro generates `DefRaw`, `Def`, Component struct, and `insert_component()`:
```rust
#[blueprint_component]
pub struct Straight {
    #[raw(default = 0)]
    pub spread: ScalarExpr,           // → f32 in Component
    #[default_expr("target.direction")]
    pub direction: VecExpr,           // → Vec2 in Component
}
```

Field type mapping:
| Def type | Component type | Notes |
|----------|---------------|-------|
| `ScalarExpr` | `f32` | Evaluated at spawn |
| `VecExpr` | `Vec2` | Evaluated at spawn |
| `EntityExpr` | `Entity` | Evaluated at spawn |
| `Option<ScalarExpr>` | `Option<f32>` | Optional expression |
| `Vec<EntityDef>` | `Vec<EntityDef>` | Cloned |
| Other types | Same type | Cloned with `#[serde(default)]` if has `#[raw(default = ...)]` |

**3. Added<T> systems handle dynamic initialization**
```rust
fn init_straight(
    mut commands: Commands,
    query: Query<(Entity, &Speed, &Straight), Added<Straight>>,
) {
    for (entity, speed, straight) in &query {
        commands.entity(entity).insert((
            RigidBody::Kinematic,
            LinearVelocity(straight.direction * speed.value),
        ));
    }
}
```

**4. Runtime state in separate components**
```rust
#[blueprint_component]
pub struct Growing {
    pub start_size: ScalarExpr,
    pub end_size: ScalarExpr,
}

#[derive(Component, Default)]
pub struct GrowingProgress {  // Separate runtime state
    pub elapsed: f32,
    pub duration: f32,
}
```

**5. Naming conventions**
- No `Trigger` suffix: `OnCollision` not `OnCollisionTrigger`
- No `Request` suffix: `Dash` not `DashRequest`
- No `Projectile` suffix: `Falling` not `FallingProjectile`
- Use `as` for conflicts: `use bevy::prelude::{Sprite as BevySprite, *}`

## Key Directories

```
src/
├── blueprints/      # Data-driven entity definitions: components, expressions, spawning
├── stats/           # Stat calculation: modifiers, expressions, health, damage
├── fsm/             # Mob FSM core: states, transitions, events
├── mob_ai/          # Concrete behaviors (move_toward_player, when_near, etc.)
├── player/          # Player spawning and input
├── ui/              # All UI screens (menu, HUD, shop, game over)
├── schedule.rs      # SystemSet definitions
└── game_state.rs    # GameState enum
```

## File Operations

**IMPORTANT:** Never use `cat`, `sed`, `awk`, `echo >`, or heredocs for file editing. Always use dedicated tools:
- **Read files:** Use Read tool (not cat/head/tail)
- **Edit files:** Use Edit tool (not sed/awk)
- **Write files:** Use Write tool (not echo/cat with heredoc)
- **Bash tool:** Only for actual terminal operations (git, cargo, npm, etc.)

## Common Development Tasks

**Add new blueprint component:**
1. Create `component_name.rs` in `blueprints/components/`
2. Define struct with `#[blueprint_component]` macro:
   ```rust
   #[blueprint_component]
   pub struct MyComponent {
       pub damage: ScalarExpr,              // Required field
       #[raw(default = false)]
       pub enabled: bool,                   // Optional with default
       #[default_expr("target.direction")]
       pub direction: VecExpr,              // Optional with expression default
   }
   ```
3. If needs Transform/RigidBody/etc — add `init_component` system with `Added<Component>` query
4. Register in `blueprints/components/mod.rs` via `collect_components!` macro

**Add new mob:**
Create `.mob.ron` in `assets/mobs/` with visual, collider, base_stats, states

**Add new stat:**
Edit `assets/stats/config.stats.ron` to add stat_id and optional calculator formula

**Add FSM behavior:**
1. Create behavior system in `mob_ai/behaviours/`
2. Register in `MobAiPlugin` (mob_ai/mod.rs)

## Physics Configuration

- Length unit: 100 pixels = 1 meter
- Faction-based collision filtering: Player vs Enemy (prevents friendly fire)
- Projectiles use sensor colliders for hit detection
