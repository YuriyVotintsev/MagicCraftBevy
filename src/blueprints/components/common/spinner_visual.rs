use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use crate::palette;
use crate::particles;

const SPIKE_COUNT: usize = 6;
const SPIKE_OFFSET: f32 = 0.55;
const BODY_ELEVATION: f32 = 0.5;
const DECAY_RATE: f32 = 3.0;

#[blueprint_component]
pub struct SpinnerVisual {
    #[raw(default = 0.4)]
    pub spike_half_length: ScalarExpr,
}

#[derive(Component)]
pub struct SpinnerVisualState {
    pub body_entity: Entity,
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
        let body_color = palette::color("enemy");
        let body_material = materials.add(StandardMaterial {
            base_color: body_color,
            unlit: true,
            ..default()
        });
        let body_mesh = meshes.add(Sphere::new(0.5));
        let body = commands
            .spawn((
                Mesh3d(body_mesh),
                MeshMaterial3d(body_material),
                Transform::from_translation(Vec3::new(0.0, BODY_ELEVATION, 0.0)),
                ScaleModifiers::default(),
            ))
            .id();
        commands.entity(entity).add_child(body);

        let spike_color = palette::color("enemy_ability");
        let spike_material = materials.add(StandardMaterial {
            base_color: spike_color,
            unlit: true,
            ..default()
        });

        let mut spike_entities = [Entity::PLACEHOLDER; SPIKE_COUNT];
        for i in 0..SPIKE_COUNT {
            let angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let spike_mesh = meshes.add(Cone::new(0.15, visual.spike_half_length));
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

        let shadow_color = palette::color_alpha("shadow", 0.45);
        let shadow_material = materials.add(StandardMaterial {
            base_color: shadow_color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let shadow_mesh = meshes.add(Circle::new(0.5));
        let shadow = commands
            .spawn((
                Mesh3d(shadow_mesh),
                MeshMaterial3d(shadow_material),
                Transform::from_translation(Vec3::new(0.0, 0.01, 0.0))
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            ))
            .id();
        commands.entity(entity).add_child(shadow);

        commands.entity(entity).insert((
            SpinnerVisualState {
                body_entity: body,
                spike_entities,
                spin_angle: 0.0,
                spin_speed: 0.0,
                spike_growth: 1.0,
                squish: 1.0,
                trail_emitters: [None; SPIKE_COUNT],
            },
            Visibility::default(),
        ));
    }
}

fn animate_spinner(
    time: Res<Time>,
    squish_layer: Res<SpinnerSquishScaleLayer>,
    mut state_query: Query<(Entity, &mut SpinnerVisualState)>,
    mut transform_query: Query<&mut Transform>,
    mut scale_query: Query<&mut ScaleModifiers>,
    has_windup: Query<(), With<super::super::mob::spinner_windup::SpinnerWindup>>,
    has_charge: Query<(), With<super::super::mob::spinner_charge::SpinnerCharge>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, mut state) in &mut state_query {
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

            if let Ok(mut spike_transform) = transform_query.get_mut(spike_entity) {
                spike_transform.translation = Vec3::new(
                    angle.cos() * SPIKE_OFFSET,
                    BODY_ELEVATION,
                    -(angle.sin() * SPIKE_OFFSET),
                );
                spike_transform.rotation =
                    Quat::from_rotation_y(angle) * Quat::from_rotation_z(FRAC_PI_2);

                spike_transform.scale = Vec3::new(1.0, state.spike_growth, 1.0);
            }
        }

        if let Ok(mut body_modifiers) = scale_query.get_mut(state.body_entity) {
            let sq = state.squish;
            let expand = 1.0 / sq.sqrt();
            body_modifiers.set(squish_layer.0, Vec3::new(expand, sq, expand));
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
