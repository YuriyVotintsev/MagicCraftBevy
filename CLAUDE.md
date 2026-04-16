# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Magic Craft is a Bevy 0.18 roguelite/incremental arena shooter. Between combat waves the player buys runes in a hex-grid shop; runes grant stat modifiers. Player has a single life pool per run; death returns to main menu.

**Tech Stack:** Bevy 0.18, Avian3D 0.5 (3D physics, top-down gameplay), Ron 0.12, Serde, `bevy_tweening`, `delaunator`, `rand`.

## Build Commands

```bash
cargo run --features dev                                             # Normal dev run
HEADLESS=1 SKIP_MENU=1 cargo run --features dev -- --timeout 10      # Headless smoke test, 10s
```

**Testing:**
- `--timeout N` is required when `HEADLESS=1`; 10s is enough for smoke validation.
- `HEADLESS=1` disables the winit plugin and runs a fixed-rate schedule loop.
- `SKIP_MENU=1` jumps straight from `Loading` to `Playing`, bypassing the main menu.
- Both env vars require `--features dev`.

## Architecture

### Plugins (registered in `main.rs`)
Each subsystem is a self-contained `Plugin`:

- `LoadingPlugin` — registers RON asset loaders and gates `GameState::Loading`.
- `ArenaPlugin` — camera, floor, walls, arena sizing.
- `StatsPlugin` — `Stat` enum, `Modifiers`, `ComputedStats`, `DirtyStats`, `StatCalculators`.
- `ActorsPlugin` — composed of `ComponentsPlugin` (combat / physics / player / visual), `MobsPlugin`, `PlayerPlugin`.
- `RunPlugin` — run lifecycle (`RunState`), money, coin pickups, player death handler.
- `RunePlugin` — shop offer, rune grid, joker slots, shop generation.
- `HealthMaterialPlugin` / `HitFlashPlugin` — custom material for HP indication on mob sprites + hit-flash tween.
- `TweeningPlugin` (from `bevy_tweening`).
- `TransitionPlugin` — iris scene transitions (custom shader).
- `UiPlugin` — HUD, main menu, shop view, game-over, pause menu, dev-menu, loading screen.
- `WavePlugin` — wave state/spawn/summoning circles.
- `ParticlesPlugin` — event-driven particle system.
- `CompositeScalePlugin` — multiplicative visual scale from independent "layers".

### State Machine

```rust
GameState   { Loading, MainMenu, Playing, GameOver }
WavePhase   { Combat, Shop }         // SubState of GameState::Playing
CombatPhase { Running, Paused, DevMenu }  // SubState of WavePhase::Combat
```

Systems are gated with `run_if(in_state(...))`. `DespawnOnExit(WavePhase::Combat)` tears down combat-scoped entities.

### System Ordering (`src/schedule.rs`)

```rust
GameSet {
    Input, MobAI, Spawning,
    AbilityActivation, AbilityExecution, AbilityLifecycle,
    Damage, DamageApply,
    WaveManagement, Cleanup,
}
ShopSet { Input, Process, Display }   // only during WavePhase::Shop
PostGameSet                           // PostUpdate, only during CombatPhase::Running
```

- Prefer `in_set(GameSet::X)` to `before()` / `after()`. Add a new set rather than sprinkling explicit ordering.
- The combat chain runs only in `CombatPhase::Running`; `ShopSet` runs only in `WavePhase::Shop`.
- Two explicit `ApplyDeferred` barriers live between `Spawning` → `AbilityActivation` and `AbilityExecution` → `AbilityLifecycle` (see `main.rs`).

### Data-Driven Assets (`assets/`)

RON files loaded at startup (see `LoadingPlugin`):

- `balance.ron` → `GameBalance` (wave pacing, arena size, run economy, rune tier weights).
- `mobs.ron` → `MobsBalance` (per-mob stat blocks for ghost/tower/slime_small/spinner/jumper).
- `runes.ron` → `RuneCosts` (price per `RuneKind`).
- `palette.ron` → `palette::*` lookup (RGB + flash colors keyed by string).
- `particles/*.particle.ron` → `ParticleConfigRaw`, supports inheritance via `parent`.

### Stats

- `Stat` is an enum of logical stats (`MaxLife`, `PhysicalDamage`, `MovementSpeed`, ...) in `src/stats/registry.rs`. Meta (`Stat::iter`, `Stat::COUNT`, `stat.name()`) is derived via `strum`.
- `ModifierKind { Flat, Increased, More }` — every modifier addresses one bucket of one stat.
- `Stat::formula()` returns `Formula::FlatIncMore` for nearly every stat; `Formula::Custom(fn)` is reserved for odd cases (currently `CritChance` clamps to `[0, 1]`).
- `ComputedStats` stores per-stat bucket triples (`[[f32; 3]; Stat::COUNT]`) plus cached final values. Two read APIs:
  - `final_of(stat)` — cached `apply(stat, 0.0)`, for self-contained stats like `MaxLife`, `MovementSpeed`.
  - `apply(stat, base)` — applies the stat's formula to a caller-provided base, for per-ability stats like `PhysicalDamage`, `ProjectileSpeed`.
- `Modifiers` holds `(Stat, ModifierKind, f32)` tuples. `sum()` aggregates `Flat`/`Increased` buckets; `product()` aggregates `More` starting at `1.0`.
- `DirtyStats` tracks which stats need recomputation; `mark_dirty_on_modifier_change` runs in `PreUpdate`.
- `StatCalculators::build()` topo-sorts on `stat.deps()` at plugin init (currently no stat has deps, but the infrastructure guards correctness if Custom formulas start reading other stats). `invalidate()` propagates dirty through `reverse_deps`.

### Combat Flow

1. `OnCollisionDamage` / `MeleeAttacker` / projectile systems emit `PendingDamage` messages.
2. `apply_pending_damage` (in `GameSet::DamageApply`) reads `PendingDamage`, rolls crit from source's `ComputedStats`, subtracts from `Health`, inserts `HitFlash`.
3. `death_system` watches `Health.current <= 0`, emits `DeathEvent`, despawns non-`SkipCleanup` entities.

### Coordinate System

Gameplay is 2D but physics runs in 3D with Y-up ground plane. Use helpers in `src/coord.rs`:

```rust
coord::ground_pos(v: Vec2) -> Vec3  // (x, 0, -y)
coord::ground_vel(v: Vec2) -> Vec3
coord::to_2d(v: Vec3)     -> Vec2
```

Length unit: 100 pixels = 1 meter (`PhysicsPlugins::default().with_length_unit(100.0)`).

### Physics & Collision Layers

`GameLayer { Default, Player, Enemy, PlayerProjectile, EnemyProjectile, Wall }`. `Collider { shape, sensor }` component auto-derives Avian layers from `Faction` (see `actors/components/physics/collider.rs`). Faction filtering prevents friendly fire; projectiles are sensors.

## Directory Layout

```
src/
├── actors/
│   ├── components/
│   │   ├── combat/     # Health, damage, death, projectiles, melee, targeting
│   │   ├── physics/    # Collider, DynamicBody, StaticBody, Size
│   │   ├── player/     # KeyboardMovement, PlayerInput, ability cooldowns
│   │   └── visual/     # Sprite, Shadow, Bobbing/JumpWalk animations, particles hooks
│   ├── mobs/           # ghost, tower, slime, spinner, jumper + spawn.rs dispatch
│   └── player.rs       # player spawn + fireball firing
├── arena/              # camera, floor, walls, window sizing
├── loading/            # generic RonAssetLoader + loading state machine
├── rune/               # Rune, RuneGrid, JokerSlots, ShopOffer, hex math, shop gen
├── run/                # RunState, PlayerMoney, coin pickups, player death
├── stats/              # Stat, Modifiers, ComputedStats, DirtyStats, calculators, display
├── ui/                 # main menu, HUD, shop view, pause, dev menu, game over, loading
├── wave/               # WavePhase, CombatPhase, enemy spawning, summoning circles
├── balance.rs          # GameBalance asset (wave / arena / run / runes)
├── composite_scale.rs  # Multiplicative scale from named layers
├── coord.rs            # 2D ↔ 3D ground-plane helpers
├── faction.rs          # Faction { Player, Enemy }
├── game_state.rs       # GameState enum
├── health_material.rs  # Custom WGSL health-bar material
├── hit_flash.rs        # Hit-flash tween component
├── main.rs             # App setup, plugin registration, schedule config
├── palette.rs          # Palette lookup from palette.ron
├── particles.rs        # Event-driven particle system (single file)
├── schedule.rs         # GameSet / ShopSet / PostGameSet
└── transition.rs       # Iris scene transition
```

## Particle System

Event-driven. Configs live in `assets/particles/*.particle.ron`; RON supports `parent` inheritance.

**API:**
```rust
let emitter = start_particles(&mut commands, "enemy_death", position);
stop_particles(&mut commands, emitter);   // only for continuous emitters
```

**Config fields:** `count` (burst size), `spawn_rate` (particles/sec, 0 = one-shot burst), `speed`, `vertical_speed`, `lifetime`, `start_size`, `end_size`, `elevation`, `color` (palette key), `shape` (`Point` or `Circle(radius)`).

**Hook components:** `OnDeathParticles { config }`, `OnCollisionParticles { config }` — attach to an entity and the effect fires on death/collision. `ParticleEmitter::shape_override` lets a system resize the spawn shape at runtime (e.g. summoning-circle growth).

## Common Development Tasks

**Add a new mob:**
1. Create `src/actors/mobs/newmob.rs` with a `NewMobStats` (serde `Deserialize`) struct, a spawn fn `spawn_newmob(commands, pos, stats, calculators) -> Entity`, behavior systems, and `register_systems(app)`.
2. Add a `MobKind::NewMob` variant and update `id()`, `from_id()`, `size()`.
3. Add the `new_mob` field to `MobsBalance` in `spawn.rs`; extend `spawn_mob` match.
4. Add the corresponding block to `assets/mobs.ron`.
5. Register in `MobsPlugin::build` via `newmob::register_systems(app)`.

*(This duplication is known debt — consider a registry refactor before adding a 6th mob.)*

**Add a new stat:**
1. Add a variant to `Stat` (`src/stats/registry.rs`). `strum` handles `iter`/`COUNT`/`name`.
2. If the stat needs a non-standard formula, add an arm to `Stat::formula()` returning `Formula::Custom(your_fn)`; otherwise it defaults to `FlatIncMore`.
3. If a Custom formula reads other computed stats, add them to `Stat::deps()` so topo-sort orders recalculation correctly.
4. Consume it via `computed.final_of(Stat::NewStat)` (self-contained) or `computed.apply(Stat::NewStat, base)` (per-ability base).

**Add a new rune:**
1. Add a variant to `RuneKind` and extend `RuneKind::ALL` + `RuneKind::def()` in `src/rune/content.rs`.
2. Add its price in `assets/runes.ron`.
3. Shop roll / grid placement picks it up automatically via `RuneKind::ALL` + tier weights.

**Add a reusable actor component:**
Write a plain `#[derive(Component)]` struct in the appropriate `actors/components/**` subfolder. If it needs init or update systems, add a `register_systems(app)` function (`Added<T>` queries are the common init pattern) and wire it into the relevant components plugin (`CombatPlugin`, `VisualPlugin`, `PhysicsPlugin`, `PlayerComponentsPlugin`).

## File Operations

**IMPORTANT:** Do not use `cat`, `sed`, `awk`, `echo >`, or heredocs for file editing. Use dedicated tools:
- Read files: Read tool
- Edit files: Edit tool
- Write files: Write tool
- Bash: only for real terminal operations (git, cargo, etc.)

