# Stat Scaling Plan

## Damage Model

Hero `physical_damage_base` = 0. Each ability has its own base damage.
Abilities use raw stat components, not computed `physical_damage`.

Formula: `(ability_base + stat(physical_damage_base)) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)`

For DoT/rapid-fire with reduced effectiveness on flat added damage:
`(ability_base + stat(physical_damage_base) * eff) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)`

Computed `physical_damage` removed (no UI for stats).

## Projectile Speed Model

No `projectile_speed_base` / `projectile_speed` computed stat. Only `projectile_speed_increased` (Sum).
Each ability has its own base speed. Hero/class scale it via `projectile_speed_increased`.
Ability speed formula: `base_speed * (1.0 + stat(projectile_speed_increased))`.

## Existing Stats Usage

| Stat | Fireball | Caustic Arrow | Flamethrower | Galv. Hammer | Meteor | Orb. Orbs | Dash | Shield |
|------|----------|---------------|--------------|--------------|--------|-----------|------|--------|
| `physical_damage_base` | damage | damage (×0.15) | damage (×0.03) | damage | damage | damage (×0.5) | — | — |
| `physical_damage_increased` | damage | damage | damage | damage | damage | damage | — | — |
| `physical_damage_more` | damage | damage | damage | damage | damage | damage | — | — |
| `projectile_speed_increased` | speed | speed | speed | speed | — | angular_speed | — | — |
| `movement_speed` | — | — | — | — | — | — | dash speed ×4 | — |
| `projectile_count` | count | count | — | count | count | count (+2) | — | — |

## New Stats Usage

| Stat | Fireball | Caustic Arrow | Flamethrower | Meteor | Orb. Orbs | Shield |
|------|----------|---------------|--------------|--------|-----------|--------|
| `area_of_effect_increased` | burst area | cloud area | — | impact area | radius, size | size, destroy radius |
| `duration_increased` | — | cloud duration | — | — | — | duration |

## Full Stat List

| Stat | Formula | Used In |
|------|---------|---------|
| `max_life` | `base * (1+inc) * more` | Health |
| `max_mana` | `base * (1+inc)` | Mana |
| `physical_damage_base` | `Sum` | Ability damage (flat component) |
| `physical_damage_increased` | `Sum` | Ability damage (% multiplier) |
| `physical_damage_more` | `Product` | Ability damage (× multiplier) |
| `movement_speed` | `base * (1+inc)` | Movement, Dash |
| `projectile_speed_increased` | `Sum` | 5 abilities: FB, CA, FT, GH, OO |
| `projectile_count` | `Sum` | FB, CA, GH, MT count; OO count (+2) |
| `crit_chance` | `clamp(base * (1+inc), 0, 1)` | Damage system |
| `crit_multiplier` | `Sum` | Damage system |
| **`area_of_effect_increased`** | **`Sum`** | **FB burst, CA cloud, MT impact, OO radius+size, Shield** |
| **`duration_increased`** | **`Sum`** | **CA cloud, Shield** |

## Stat Config Changes (config.stats.ron)

| Action | Stat |
|--------|------|
| Remove | `strength_base`, `strength_increased`, `strength_more`, `strength` |
| Remove | `max_life_per_strength` |
| Remove | calculator for `max_life` (rewrite without strength) |
| Remove | `projectile_speed_base` |
| Remove | `projectile_speed` (computed) |
| Remove | calculator for `projectile_speed` |
| Remove | `physical_damage` (computed) |
| Remove | `movement_speed_more`, rewrite `movement_speed` calculator without `more` |
| Remove | `crit_multiplier_base`, `crit_multiplier_increased`, calculator for `crit_multiplier` |
| Keep | `crit_multiplier` (Sum, standalone — was computed, now plain) |
| Keep | `physical_damage_base` (Sum), `physical_damage_increased` (Sum), `physical_damage_more` (Product) |
| Keep | `projectile_speed_increased` (Sum) |
| Add | `area_of_effect_increased` (Sum) |
| Add | `duration_increased` (Sum) |

## Hero/Class Changes

| File | Stat | Before | After |
|------|------|--------|-------|
| `base.hero.ron` | `strength_base` | `5.0` | remove |
| `base.hero.ron` | `max_life_per_strength` | `1.5` | remove |
| `base.hero.ron` | `physical_damage_base` | `10.0` | remove |
| `base.hero.ron` | `projectile_speed_base` | `800.0` | remove |
| `mage.class.ron` | `projectile_speed_base` | `100.0` | `projectile_speed_increased: 0.15` |
| `warrior.class.ron` | `strength_base` | `5.0` | replace with `max_life_base` or similar |
| `warrior.class.ron` | `max_life_per_strength` | `0.5` | remove |

## Ability Changes

Damage shorthand: `DMG(base)` = `(base + stat(physical_damage_base)) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)`
Damage shorthand: `DMG(base, eff)` = `(base + stat(physical_damage_base) * eff) * (1 + stat(physical_damage_increased)) * stat(physical_damage_more)`

| Ability | Parameter | Before | After |
|---------|-----------|--------|-------|
| Fireball | count | (none) | `stat(projectile_count)` |
| Fireball | speed | `stat(projectile_speed) + 100.0` | `800.0 * (1.0 + stat(projectile_speed_increased))` |
| Fireball | damage | `stat(physical_damage)` | DMG(15) |
| Fireball | burst damage | `stat(physical_damage) * 0.5` | DMG(7.5, 0.5) |
| Fireball | burst OnArea.size | `160.0` | `160.0 * (1.0 + stat(area_of_effect_increased))` |
| Caustic Arrow | count | (none) | `stat(projectile_count)` |
| Caustic Arrow | speed | `600.0` | `600.0 * (1.0 + stat(projectile_speed_increased))` |
| Caustic Arrow | DoT damage | `5.0` | DMG(2, 0.15) |
| Caustic Arrow | cloud Lifetime | `4.0` | `4.0 * (1.0 + stat(duration_increased))` |
| Caustic Arrow | OnArea.size | `160.0` | `160.0 * (1.0 + stat(area_of_effect_increased))` |
| Flamethrower | speed | `600.0` | `600.0 * (1.0 + stat(projectile_speed_increased))` |
| Flamethrower | damage | `1.0` | DMG(0.5, 0.03) |
| Galv. Hammer | count | (none) | `stat(projectile_count)` |
| Galv. Hammer | speed | `500.0` | `500.0 * (1.0 + stat(projectile_speed_increased))` |
| Galv. Hammer | damage | `stat(physical_damage)` | DMG(15) |
| Meteor | count | (none) | `stat(projectile_count)` |
| Meteor | damage | `stat(physical_damage)` | DMG(50) |
| Meteor | OnArea.size | `160.0` | `160.0 * (1.0 + stat(area_of_effect_increased))` |
| Orb. Orbs | radius | `80.0` | `80.0 * (1.0 + stat(area_of_effect_increased))` |
| Orb. Orbs | size | `40.0` | `40.0 * (1.0 + stat(area_of_effect_increased))` |
| Orb. Orbs | angular_speed | `stat(movement_speed) / 100.0` | `3.5 * (1.0 + stat(projectile_speed_increased))` |
| Orb. Orbs | damage | `stat(physical_damage)` | DMG(8, 0.5) |
| Orb. Orbs | count | `3` | `stat(projectile_count) + 2` |
| Dash | speed | `1500.0` | `stat(movement_speed) * 4.0` |
| Shield | size | `400.0` | `400.0 * (1.0 + stat(area_of_effect_increased))` |
| Shield | destroy radius | `200.0` | `200.0 * (1.0 + stat(area_of_effect_increased))` |
| Shield | duration | `0.5` | `0.5 * (1.0 + stat(duration_increased))` |

## Artifact Changes

| Artifact | Before | After |
|----------|--------|-------|
| Speed Boots | `movement_speed_base: 30` | `movement_speed_increased: 0.10` |
| Shadow Cloak | `movement_speed_base: 20` | `movement_speed_increased: 0.08` |
| Steel Gauntlets | `strength_base: 3, physical_damage_base: 5` | `physical_damage_base: 8` |
| Golden Crown | `strength_base: 5, max_life_base: 20, physical_damage_base: 5` | `max_life_base: 30, physical_damage_base: 5` |
| Emerald Amulet | `max_life_base: 15, strength_base: 3` | `max_life_base: 20` |

## Affix Changes (remove strength)

| Pool | Affix | Action |
|------|-------|--------|
| active | `flat_strength` (+5/10/18/25 strength_base) | replace with `flat_max_life` or remove |
| defensive | `flat_strength` (+3/7/12/18 strength_base) | replace with `flat_max_life` or remove |
| defensive | `increased_strength` (+4/8/14% strength_increased) | remove |
| passive | `flat_strength` (+4/8/14/20 strength_base) | replace with `flat_max_life` or remove |
