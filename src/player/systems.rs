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
use crate::stats::{ComputedStats, DirtyStats, Modifiers, Stat, StatCalculators};
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
    calculators: Res<StatCalculators>,
    mut selected_spells: ResMut<SelectedSpells>,
) {
    let base_stats: &[(Stat, f32)] = &[
        (Stat::MaxLifeFlat, 20.0),
        (Stat::MovementSpeedFlat, 550.0),
        (Stat::CritChanceFlat, 0.05),
        (Stat::CritMultiplier, 1.5),
        (Stat::PickupRadiusFlat, 200.0),
    ];

    let mut modifiers = Modifiers::new();
    for &(stat, value) in base_stats {
        modifiers.add(stat, value);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::ALL.iter().copied());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    let hp = computed.get(Stat::MaxLife);

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
            target: TargetInfo::EMPTY,
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
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn((
            Sprite {
                color: player_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.6, bounce_duration: 0.45, land_squish: 0.3, land_duration: 0.125 },
        ));
    });

    selected_spells.set(SpellSlot::Active, AbilityKind::Fireball);
}
