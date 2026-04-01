use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use rand::Rng;

use crate::blueprints::SpawnSource;
use crate::particles::{HitParticleConfig, Particle};
use crate::GameState;

#[blueprint_component]
pub struct HitParticles;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        spawn_hit_particles.run_if(in_state(GameState::Playing)),
    );
}

fn spawn_hit_particles(
    mut commands: Commands,
    query: Query<(Entity, &SpawnSource), Added<HitParticles>>,
    config: Res<HitParticleConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, source) in &query {
        let pos = source.target.position
            .or(source.source.position)
            .unwrap_or(Vec2::ZERO);

        let color = crate::palette::color(&config.color);
        let material = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        let mesh = meshes.add(Sphere::new(0.5));

        let mut rng = rand::rng();
        for _ in 0..config.count {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = config.speed * rng.random_range(0.5..1.0);
            let dir = Vec2::new(angle.cos(), angle.sin());
            let start_scale = config.start_size / 2.0;
            let end_scale = config.end_size / 2.0;

            let spawn_pos = crate::coord::ground_pos(pos);
            commands.spawn((
                Particle {
                    velocity: crate::coord::ground_vel(dir * speed),
                    remaining: config.lifetime,
                    lifetime: config.lifetime,
                    start_scale,
                    end_scale,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::new(spawn_pos.x, config.elevation, spawn_pos.z))
                    .with_scale(Vec3::splat(start_scale)),
            ));
        }

        commands.entity(entity).despawn();
    }
}
