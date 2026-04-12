use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;

use crate::actors::components::common::sprite::CircleSprite;
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use crate::palette;
use crate::particles;

const SPIKE_COUNT: usize = 6;
const SPIKE_OFFSET: f32 = 0.55;
const BODY_ELEVATION: f32 = 0.5;
const DECAY_RATE: f32 = 3.0;

#[derive(Component)]
pub struct SpinnerVisual {
    pub spike_length: f32,
}

#[derive(Component)]
pub struct SpinnerVisualState {
    pub spike_entities: [Entity; SPIKE_COUNT],
    pub spin_angle: f32,
    pub spin_speed: f32,
    pub spike_growth: f32,
    pub squish: f32,
    pub trail_emitters: [Option<Entity>; SPIKE_COUNT],
}

#[derive(Resource)]
pub struct SpinnerSquishScaleLayer(pub ScaleLayerId);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_layer);
    app.add_systems(PostUpdate, (init_spinner_visual, animate_spinner).chain());
    app.add_observer(on_remove_visual_state);
}

fn on_remove_visual_state(
    on: On<Remove, SpinnerVisualState>,
    query: Query<&SpinnerVisualState>,
    mut commands: Commands,
) {
    let entity = on.event_target();
    let Ok(state) = query.get(entity) else { return };
    for emitter in &state.trail_emitters {
        if let Some(e) = *emitter {
            particles::stop_particles(&mut commands, e);
        }
    }
}

fn register_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(SpinnerSquishScaleLayer(registry.register()));
}

fn init_spinner_visual(
    mut commands: Commands,
    query: Query<(Entity, &SpinnerVisual), Added<SpinnerVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, visual) in &query {
        let spike_color = palette::color("enemy_ability");
        let spike_material = materials.add(StandardMaterial {
            base_color: spike_color,
            unlit: true,
            ..default()
        });

        let mut spike_entities = [Entity::PLACEHOLDER; SPIKE_COUNT];
        for i in 0..SPIKE_COUNT {
            let angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let spike_mesh = meshes.add(Cone::new(0.15, visual.spike_length));
            let position = Vec3::new(
                angle.cos() * SPIKE_OFFSET,
                BODY_ELEVATION,
                -(angle.sin() * SPIKE_OFFSET),
            );
            let rotation =
                Quat::from_rotation_y(angle) * Quat::from_rotation_z(-FRAC_PI_2);

            let spike = commands
                .spawn((
                    Mesh3d(spike_mesh),
                    MeshMaterial3d(spike_material.clone()),
                    Transform::from_translation(position).with_rotation(rotation),
                ))
                .id();
            commands.entity(entity).add_child(spike);
            spike_entities[i] = spike;
        }

        commands.entity(entity).insert(
            SpinnerVisualState {
                spike_entities,
                spin_angle: 0.0,
                spin_speed: 0.0,
                spike_growth: 1.0,
                squish: 1.0,
                trail_emitters: [None; SPIKE_COUNT],
            },
        );
    }
}

fn animate_spinner(
    time: Res<Time>,
    squish_layer: Res<SpinnerSquishScaleLayer>,
    mut state_query: Query<(
        Entity,
        &mut SpinnerVisualState,
        &Transform,
        &SpinnerVisual,
        &Children,
        Option<&crate::actors::components::common::size::Size>,
    )>,
    mut transform_query: Query<&mut Transform, Without<SpinnerVisualState>>,
    mut scale_query: Query<&mut ScaleModifiers>,
    circle_query: Query<(), With<CircleSprite>>,
    has_windup: Query<(), With<super::super::mob::spinner_windup::SpinnerWindup>>,
    has_charge: Query<(), With<super::super::mob::spinner_charge::SpinnerCharge>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, mut state, spinner_transform, visual, children, size) in &mut state_query {
        let spinner_pos = crate::coord::to_2d(spinner_transform.translation);
        let entity_scale = size.map_or(1.0, |s| s.value / 2.0);

        let active = has_windup.contains(entity) || has_charge.contains(entity);
        if !active {
            state.spin_speed = lerp_toward(state.spin_speed, 0.0, DECAY_RATE * dt);
            state.spike_growth = lerp_toward(state.spike_growth, 1.0, DECAY_RATE * dt);
            state.squish = lerp_toward(state.squish, 1.0, DECAY_RATE * dt);

            for emitter in &mut state.trail_emitters {
                if let Some(e) = emitter.take() {
                    particles::stop_particles(&mut commands, e);
                }
            }
        }

        state.spin_angle += state.spin_speed * dt;

        for (i, &spike_entity) in state.spike_entities.iter().enumerate() {
            let base_angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let angle = base_angle + state.spin_angle;

            let local_offset = Vec2::new(angle.cos(), angle.sin()) * SPIKE_OFFSET;

            if let Ok(mut spike_transform) = transform_query.get_mut(spike_entity) {
                spike_transform.translation = Vec3::new(
                    local_offset.x,
                    BODY_ELEVATION,
                    -local_offset.y,
                );
                spike_transform.rotation =
                    Quat::from_rotation_y(angle) * Quat::from_rotation_z(-FRAC_PI_2);

                spike_transform.scale = Vec3::new(1.0, state.spike_growth, 1.0);
            }

            if let Some(emitter_entity) = state.trail_emitters[i] {
                let dir = Vec2::new(angle.cos(), angle.sin());
                let tip_local = dir * (SPIKE_OFFSET + visual.spike_length / 2.0 * state.spike_growth);
                let world_pos = spinner_pos + tip_local * entity_scale;
                if let Ok(mut emitter_transform) = transform_query.get_mut(emitter_entity) {
                    emitter_transform.translation = crate::coord::ground_pos(world_pos);
                    emitter_transform.translation.y = BODY_ELEVATION * entity_scale;
                }
            }
        }

        let sq = state.squish;
        let expand = 1.0 / sq.sqrt();
        let squish_scale = Vec3::new(expand, sq, expand);

        for child in children.iter() {
            if circle_query.contains(child) {
                if let Ok(mut body_modifiers) = scale_query.get_mut(child) {
                    body_modifiers.set(squish_layer.0, squish_scale);
                }
            }
        }
    }
}

fn lerp_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    if (current - target).abs() <= max_delta {
        target
    } else if current > target {
        current - max_delta
    } else {
        current + max_delta
    }
}
