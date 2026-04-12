use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::abilities::AbilityKind;
use crate::actors::components::common::collider::{Collider, Shape as ColliderShape};
use crate::actors::components::common::dynamic_body::DynamicBody;
use crate::actors::components::common::health::Health;
use crate::actors::components::common::jump_walk_animation::JumpWalkAnimation;
use crate::actors::components::common::shadow::Shadow;
use crate::actors::components::common::size::Size;
use crate::actors::components::common::sprite::{Sprite, SpriteColor, SpriteShape};
use crate::actors::components::player::keyboard_movement::KeyboardMovement;
use crate::actors::components::player::player_input::{
    InputBinding, InputTrigger, KeyKind, MouseButtonKind, PlayerAbilityCooldowns, PlayerInput, TargetingMode,
};
use crate::actors::TargetInfo;
use crate::actors::SpawnSource;
use crate::palette;
use crate::player::selected_spells::SpellSlot;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, StatCalculators, StatId, StatRegistry};
use crate::wave::WavePhase;
use crate::Faction;

use super::SelectedSpells;

#[derive(Component)]
pub struct Player;

pub fn reset_player_velocity(mut query: Query<&mut LinearVelocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 = Vec3::ZERO;
    }
}

fn player_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("player").unwrap_or((0.5, 0.8, 1.0));
    let flash = palette::flash_lookup("player");
    SpriteColor { r, g, b, a: 1.0, flash }
}

pub fn spawn_player(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    calculators: Res<StatCalculators>,
    mut selected_spells: ResMut<SelectedSpells>,
) {
    let base_stats: &[(&str, f32)] = &[
        ("max_life_flat", 20.0),
        ("movement_speed_flat", 550.0),
        ("crit_chance_flat", 0.05),
        ("crit_multiplier", 1.5),
        ("pickup_radius_flat", 200.0),
    ];

    let mut modifiers = Modifiers::new();
    for (name, value) in base_stats {
        if let Some(id) = stat_registry.get(name) {
            modifiers.add(id, *value);
        }
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::new(stat_registry.len());
    dirty.mark_all((0..stat_registry.len() as u32).map(StatId));
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    let hp = stat_registry.get("max_life").map(|id| computed.get(id)).unwrap_or(20.0);

    let entity = commands.spawn((
        Name::new("Player"),
        Player,
        Transform::from_translation(Vec3::ZERO),
        Visibility::default(),
        Faction::Player,
        modifiers, dirty, computed,
        Size { value: 120.0 },
        Collider { shape: ColliderShape::Rectangle, sensor: false },
        DynamicBody { mass: 3.0 },
        Health { current: hp },
        KeyboardMovement {},
        PlayerAbilityCooldowns::default(),
        DespawnOnExit(WavePhase::Combat),
    )).id();
    commands.entity(entity).insert((
        SpawnSource {
            caster: TargetInfo::from_entity_and_position(entity, Vec2::ZERO),
            caster_faction: Faction::Player,
            source: TargetInfo::EMPTY,
            target: TargetInfo::EMPTY,
            index: 0,
            count: 1,
        },
        PlayerInput {
            bindings: vec![
                InputBinding { slot: SpellSlot::Active, trigger: InputTrigger::MouseHold(MouseButtonKind::Left), targeting: TargetingMode::Cursor },
                InputBinding { slot: SpellSlot::Defensive, trigger: InputTrigger::KeyJustPressed(KeyKind::Space), targeting: TargetingMode::MovementDirection },
                InputBinding { slot: SpellSlot::Passive, trigger: InputTrigger::Auto, targeting: TargetingMode::Untargeted },
            ],
        },
    ));

    commands.entity(entity).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn((
            Sprite {
                color: player_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.6, bounce_duration: 0.45, max_tilt: 12.0, land_squish: 0.3, land_duration: 0.125 },
        ));
    });

    selected_spells.set(SpellSlot::Active, AbilityKind::Fireball);
}
