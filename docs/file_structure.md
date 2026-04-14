# Bevy Project Organization Convention

## Core Principle

The project follows a **plugin-per-feature** architecture. Each feature is a self-contained module that owns its components, systems, resources, and events. The only shared code lives in `common/`.

## Directory Structure

```
src/
  main.rs              # Only App::new(), add_plugins(), and run()
  lib.rs               # Re-exports all plugins via GamePlugin
  common/
    mod.rs             # CommonPlugin (registers shared events, resources)
    components.rs      # Components used by 2+ features (Health, Speed, Team, etc.)
    events.rs          # Events used by 2+ features
    resources.rs       # Resources used by 2+ features
  <feature>/
    mod.rs             # FeaturePlugin — registers all systems from submodules
    components.rs      # Components owned exclusively by this feature
    <submodule>.rs     # Systems grouped by responsibility
```

## Rules

### 1. main.rs

- Contains only `App::new()`, plugin registration, and `.run()`.
- No systems, no components, no logic.

```rust
use bevy::prelude::*;
use crate::lib::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}
```

### 2. lib.rs

- Declares all top-level modules (`mod common;`, `mod player;`, `mod combat;`, etc.).
- Exports a single `GamePlugin` that registers all feature plugins.

```rust
mod common;
mod player;
mod combat;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            common::CommonPlugin,
            player::PlayerPlugin,
            combat::CombatPlugin,
        ));
    }
}
```

### 3. Feature modules (`<feature>/mod.rs`)

- Each feature folder has a `mod.rs` that declares submodules and exposes a single `pub struct FeaturePlugin`.
- The plugin registers all systems, events, and resources belonging to this feature.
- Systems are private functions inside submodules. Only the plugin is `pub`.
- Re-export components with `pub use components::*;` so other modules can use them.

```rust
mod components;
mod movement;
mod animation;

use bevy::prelude::*;
pub use components::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, movement::spawn_player)
           .add_systems(Update, (
               movement::move_player,
               animation::animate_player,
           ));
    }
}
```

### 4. Feature components (`<feature>/components.rs`)

- Contains only `#[derive(Component)]` structs and enums **owned by this feature**.
- A component is "owned" by a feature if that feature is responsible for creating and primarily managing it.
- All components are `pub`.

### 5. Feature submodules (`<feature>/<submodule>.rs`)

- Contain system functions grouped by responsibility (e.g., `movement.rs`, `damage.rs`, `spawning.rs`, `ai.rs`).
- System functions are `pub(super)` — visible to `mod.rs` for plugin registration, but not to the rest of the crate.
- Import shared components: `use crate::common::*;`
- Import own feature components: `use super::components::*;`

### 6. common/ module

- A component goes into `common/components.rs` **only if** it is used by 2 or more features and no single feature owns it.
- Same rule for events and resources.
- `CommonPlugin` registers shared events and initializes shared resources. It does NOT register systems — systems always belong to a feature.

### 7. When to split

- A feature file is under ~300 lines → keep everything in one file (components + systems + plugin), no subfolder needed.
- A feature file grows beyond ~300 lines → extract into a folder with `mod.rs`, `components.rs`, and system submodules.
- A submodule grows beyond ~400 lines → split into smaller submodules by responsibility.

## When Refactoring

1. Identify all features (player, enemy, combat, ui, camera, audio, etc.).
2. For each component, decide which feature **owns** it. If no single feature owns it → `common/`.
3. For each system, decide which feature it belongs to based on its primary purpose.
4. Move components and systems into their feature folders.
5. Create a plugin in each feature's `mod.rs` that registers its systems.
6. Wire all plugins through `GamePlugin` in `lib.rs`.
7. Ensure `main.rs` contains only app setup and plugin registration.
8. Run `cargo check` after each feature migration to catch broken imports early.

## Naming Conventions

- Plugins: `PlayerPlugin`, `CombatPlugin`, `EnemyPlugin`
- System functions: `snake_case` verbs — `spawn_player`, `apply_damage`, `move_projectiles`
- Components: `PascalCase` nouns — `Player`, `Health`, `Velocity`
- Events: `PascalCase` past tense or noun — `DamageDealt`, `EnemySpawned`
- Resources: `PascalCase` nouns — `GameSettings`, `WaveTimer`
- Feature folders: `snake_case` — `player/`, `combat/`, `enemy_ai/`

## Import Style

```rust
// In system submodules:
use bevy::prelude::*;
use crate::common::*;       // shared components, events, resources
use super::components::*;   // own feature components
```

## Anti-Patterns to Avoid

- **Global `components/` and `systems/` folders** — do not separate components from systems globally. They belong together inside their feature.
- **Systems in `mod.rs`** — `mod.rs` only declares submodules, re-exports, and the plugin. Systems live in submodules.
- **Cross-feature component imports** — if feature A frequently imports components from feature B, consider moving those components to `common/`.
- **Giant `common/`** — if `common/components.rs` has more components than any single feature, something is wrong. Re-evaluate ownership.
- **Logic in `main.rs`** — main.rs is only for app bootstrap.
