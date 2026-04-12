use bevy::prelude::*;
use serde::Deserialize;

use crate::actors::abilities::{AbilitiesBalance, AbilityKind};
use crate::actors::effects::OnDeathParticles;
use crate::actors::mob_abilities::MobAbilities;
use crate::actors::components::ability::find_nearest_enemy::FindNearestEnemy;
use crate::actors::components::common::bobbing_animation::BobbingAnimation;
use crate::actors::components::common::collider::{Collider, Shape as ColliderShape};
use crate::actors::components::common::dynamic_body::DynamicBody;
use crate::actors::components::common::health::Health;
use crate::actors::components::common::jump_walk_animation::JumpWalkAnimation;
use crate::actors::components::common::shadow::Shadow;
use crate::actors::components::common::shoot_squish::ShootSquish;
use crate::actors::components::common::size::Size;
use crate::actors::components::common::spinner_visual::SpinnerVisual;
use crate::actors::components::common::sprite::{Sprite, SpriteColor, SpriteShape};
use crate::actors::components::common::static_body::StaticBody;
use crate::actors::components::common::tower_visual::TowerVisual;
use crate::actors::components::mob::ghost_transparency::GhostTransparency;
use crate::actors::components::mob::jumper_ai::JumperAi;
use crate::actors::components::mob::lunge_movement::LungeMovement;
use crate::actors::components::mob::move_toward::MoveToward;
use crate::actors::components::mob::spinner_ai::SpinnerAi;
use crate::actors::TargetInfo;
use crate::actors::SpawnSource;
use crate::faction::Faction;
use crate::palette;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, StatCalculators, StatId, StatRegistry};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MobKind {
    Ghost,
    Tower,
    SlimeSmall,
    Spinner,
    Jumper,
}

impl MobKind {
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "ghost" => Some(MobKind::Ghost),
            "tower" => Some(MobKind::Tower),
            "slime_small" => Some(MobKind::SlimeSmall),
            "spinner" => Some(MobKind::Spinner),
            "jumper" => Some(MobKind::Jumper),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            MobKind::Ghost => "ghost",
            MobKind::Tower => "tower",
            MobKind::SlimeSmall => "slime_small",
            MobKind::Spinner => "spinner",
            MobKind::Jumper => "jumper",
        }
    }

    pub fn size(&self, mobs: &MobsBalance) -> f32 {
        match self {
            MobKind::Ghost => mobs.ghost.size,
            MobKind::Tower => mobs.tower.size,
            MobKind::SlimeSmall => mobs.slime_small.size,
            MobKind::Spinner => mobs.spinner.size,
            MobKind::Jumper => mobs.jumper.size,
        }
    }
}

#[derive(Resource, Clone, Deserialize, Debug)]
pub struct MobsBalance {
    pub ghost: GhostStats,
    pub tower: TowerStats,
    pub slime_small: SlimeSmallStats,
    pub jumper: JumperStats,
    pub spinner: SpinnerStats,
}

#[derive(Clone, Deserialize, Debug)]
pub struct GhostStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub mass: f32,
    pub melee_range: f32,
    pub melee_cooldown: f32,
    pub visible_distance: f32,
    pub invisible_distance: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TowerStats {
    pub hp: f32,
    pub damage: f32,
    pub size: f32,
    pub shot_cooldown: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SlimeSmallStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub mass: f32,
    pub melee_range: f32,
    pub melee_cooldown: f32,
    pub lunge_duration: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct JumperStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub mass: f32,
    pub idle_duration: f32,
    pub jump_duration: f32,
    pub land_duration: f32,
    pub jump_distance: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SpinnerStats {
    pub hp: f32,
    pub damage: f32,
    pub size: f32,
    pub mass: f32,
    pub spike_length: f32,
    pub idle_duration: f32,
    pub windup_duration: f32,
    pub charge_duration: f32,
    pub cooldown_duration: f32,
    pub charge_speed: f32,
}

fn enemy_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy").unwrap_or((1.0, 1.0, 1.0));
    let flash = palette::flash_lookup("enemy");
    SpriteColor { r, g, b, a: 1.0, flash }
}

fn compute_stats(
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    base_stats: &[(&str, f32)],
    extra_modifiers: &[(StatId, f32)],
) -> (Modifiers, DirtyStats, ComputedStats) {
    let mut modifiers = Modifiers::new();
    for (name, value) in base_stats {
        if let Some(id) = stat_registry.get(name) {
            modifiers.add(id, *value);
        }
    }
    for &(id, value) in extra_modifiers {
        modifiers.add(id, value);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::new(stat_registry.len());
    dirty.mark_all((0..stat_registry.len() as u32).map(StatId));
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    (modifiers, dirty, computed)
}

fn make_source(entity: Entity, pos: Vec2) -> SpawnSource {
    SpawnSource {
        caster: TargetInfo::from_entity_and_position(entity, pos),
        caster_faction: Faction::Enemy,
        source: TargetInfo::EMPTY,
        target: TargetInfo::EMPTY,
        index: 0,
        count: 1,
    }
}

fn current_max_life(computed: &ComputedStats, registry: &StatRegistry) -> f32 {
    registry.get("max_life").map(|id| computed.get(id)).unwrap_or(1.0)
}

pub fn spawn_mob(
    commands: &mut Commands,
    kind: MobKind,
    pos: Vec2,
    mobs: &MobsBalance,
    _abilities: &AbilitiesBalance,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    extra_modifiers: &[(StatId, f32)],
) -> Entity {
    match kind {
        MobKind::Ghost => spawn_ghost(commands, pos, &mobs.ghost, stat_registry, calculators, extra_modifiers),
        MobKind::Tower => spawn_tower(commands, pos, &mobs.tower, stat_registry, calculators, extra_modifiers),
        MobKind::SlimeSmall => spawn_slime_small(commands, pos, &mobs.slime_small, stat_registry, calculators, extra_modifiers),
        MobKind::Spinner => spawn_spinner(commands, pos, &mobs.spinner, stat_registry, calculators, extra_modifiers),
        MobKind::Jumper => spawn_jumper(commands, pos, &mobs.jumper, stat_registry, calculators, extra_modifiers),
    }
}

fn spawn_slime_small(
    commands: &mut Commands,
    pos: Vec2,
    s: &SlimeSmallStats,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    extra_modifiers: &[(StatId, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        stat_registry, calculators,
        &[("movement_speed_flat", s.speed), ("max_life_flat", s.hp), ("physical_damage_flat", s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed, stat_registry);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: s.mass },
        Health { current: hp },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        LungeMovement { speed: None, duration: Some(s.lunge_duration), pause_duration: 0.4, distance: None },
        MobAbilities::new(vec![AbilityKind::MeleeAttack], s.melee_cooldown, Some(s.melee_range)),
    )).id();

    commands.entity(id).insert((
        make_source(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn((
            Sprite {
                color: enemy_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.7, bounce_duration: 0.5, max_tilt: 12.0, land_squish: 0.3, land_duration: 0.4 },
        ));
    });

    id
}

fn spawn_ghost(
    commands: &mut Commands,
    pos: Vec2,
    s: &GhostStats,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    extra_modifiers: &[(StatId, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        stat_registry, calculators,
        &[("movement_speed_flat", s.speed), ("max_life_flat", s.hp), ("physical_damage_flat", s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed, stat_registry);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: 1.0 },
        Health { current: hp },
        GhostTransparency { visible_distance: s.visible_distance, invisible_distance: s.invisible_distance },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        MoveToward {},
        MobAbilities::new(vec![AbilityKind::MeleeAttack], s.melee_cooldown, Some(s.melee_range)),
    )).id();

    commands.entity(id).insert((
        make_source(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn((
            Sprite {
                color: enemy_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
            },
            BobbingAnimation { amplitude: 0.2, speed: 2.0, base_elevation: 0.5 },
        ));
    });

    id
}

fn spawn_tower(
    commands: &mut Commands,
    pos: Vec2,
    s: &TowerStats,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    extra_modifiers: &[(StatId, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        stat_registry, calculators,
        &[("max_life_flat", s.hp), ("physical_damage_flat", s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed, stat_registry);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        StaticBody,
        Health { current: hp },
        TowerVisual {},
        ShootSquish { amplitude: 0.3, duration: 0.25 },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        MobAbilities::new(vec![AbilityKind::TowerShot], s.shot_cooldown, None),
    )).id();

    commands.entity(id).insert((
        make_source(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death_large" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn(Sprite {
            color: enemy_sprite_color(), shape: SpriteShape::Circle,
            position: Vec2::ZERO, scale: 1.0, image: None, elevation: 1.2, half_length: 0.5,
        });
    });

    id
}

fn spawn_jumper(
    commands: &mut Commands,
    pos: Vec2,
    s: &JumperStats,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    extra_modifiers: &[(StatId, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        stat_registry, calculators,
        &[("movement_speed_flat", s.speed), ("max_life_flat", s.hp), ("physical_damage_flat", s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed, stat_registry);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: s.mass },
        Health { current: hp },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        JumperAi {
            ability: "jumper_shot".to_string(),
            idle_duration: s.idle_duration,
            jump_duration: s.jump_duration,
            land_duration: s.land_duration,
            jump_distance: s.jump_distance,
        },
    )).id();

    commands.entity(id).insert((
        make_source(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death_large" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn((
            Sprite {
                color: enemy_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.7, bounce_duration: 0.5, max_tilt: 12.0, land_squish: 0.7, land_duration: 0.4 },
        ));
    });

    id
}

fn spawn_spinner(
    commands: &mut Commands,
    pos: Vec2,
    s: &SpinnerStats,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    extra_modifiers: &[(StatId, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        stat_registry, calculators,
        &[("max_life_flat", s.hp), ("physical_damage_flat", s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed, stat_registry);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: s.mass },
        Health { current: hp },
        SpinnerVisual { spike_length: s.spike_length },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        SpinnerAi {
            idle_duration: s.idle_duration,
            windup_duration: s.windup_duration,
            charge_duration: s.charge_duration,
            cooldown_duration: s.cooldown_duration,
            charge_speed: s.charge_speed,
        },
    )).id();

    commands.entity(id).insert((
        make_source(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death_large" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn(Sprite {
            color: enemy_sprite_color(), shape: SpriteShape::Circle,
            position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
        });
    });
    id
}
