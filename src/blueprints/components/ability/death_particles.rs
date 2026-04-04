use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use rand::Rng;

use crate::blueprints::SpawnSource;
use crate::particles::Particle;
use crate::GameState;

#[blueprint_component]
pub struct DeathParticles {
    #[raw(default = 8)]
    pub count: ScalarExpr,
    #[raw(default = 200.0)]
    pub speed: ScalarExpr,
    #[raw(default = 0.4)]
    pub lifetime: ScalarExpr,
    #[raw(default = 60.0)]
    pub start_size: ScalarExpr,
    #[raw(default = 0.0)]
    pub end_size: ScalarExpr,
    #[raw(default = 0.5)]
    pub elevation: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        spawn_death_particles.run_if(in_state(GameState::Playing)),
    );
}

fn spawn_death_particles(
    mut commands: Commands,
    query: Query<(Entity, &SpawnSource, &DeathParticles), Added<DeathParticles>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, source, config) in &query {
        let pos = source.target.position
            .or(source.source.position)
            .unwrap_or(Vec2::ZERO);

        let color = crate::palette::color("enemy");
        let material = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        let mesh = meshes.add(Sphere::new(0.5));

        let count = config.count as u32;
        let mut rng = rand::rng();
        for _ in 0..count {
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
