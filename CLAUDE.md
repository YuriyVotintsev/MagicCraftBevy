# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Magic Craft is a Bevy 0.18 roguelite arena shooter with a Path-of-Exile-inspired ability modification system. Players face waves of enemies and customize abilities through a deep spell-crafting system.

**Tech Stack:** Bevy 0.18, Avian2D 0.5 (physics), Ron (config format), Serde

## Build Commands

```bash
cargo run                                      # Normal mode
cargo run --features headless -- --timeout 60  # Headless mode (CI/testing)
```

## Architecture

### Plugin-Based Structure
Each major system is a self-contained Bevy Plugin registered in `main.rs`:
- `AbilityPlugin` - Core ability system (most complex)
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
- **AbilityContext**: Data passed to effects/triggers during execution
- **PendingDamage**: Queued damage applied in GameSet::Damage
- **DirtyStats**: Tracks which stats need recalculation (optimization)
- **Raw→Processed types**: `*Raw` types use strings for Ron deserialization, converted to typed IDs at load time

## Key Directories

```
src/
├── abilities/       # Core ability system: dispatcher, registries, effects, triggers
├── stats/           # Stat calculation: modifiers, expressions, health, damage
├── fsm/             # Mob FSM core: states, transitions, events
├── mob_ai/          # Concrete behaviors (move_toward_player, when_near, etc.)
├── player/          # Player spawning and input
├── ui/              # All UI screens (menu, HUD, shop, game over)
├── schedule.rs      # SystemSet definitions
└── game_state.rs    # GameState enum
```

## Common Development Tasks

**Add new ability type:**
1. Implement `Trigger` or `EffectExecutor` trait in `abilities/triggers/` or `abilities/effects/`
2. Register in `abilities/mod.rs` plugin setup

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
