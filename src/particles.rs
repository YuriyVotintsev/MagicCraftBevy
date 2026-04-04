use bevy::prelude::*;
use serde::Deserialize;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_config)
            .add_systems(Startup, load_player_death_config)
            .add_systems(Update, update_particles);
    }
}

#[derive(Resource, Deserialize, Clone)]
pub struct HitParticleConfig {
    pub count: u32,
    pub speed: f32,
    pub lifetime: f32,
    pub start_size: f32,
    pub end_size: f32,
    pub elevation: f32,
    pub color: String,
}

impl Default for HitParticleConfig {
    fn default() -> Self {
        Self {
            count: 8,
            speed: 300.0,
            lifetime: 0.3,
            start_size: 20.0,
            end_size: 0.0,
            elevation: 50.0,
            color: "player_ability".into(),
        }
    }
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec3,
    pub remaining: f32,
    pub lifetime: f32,
    pub start_scale: f32,
    pub end_scale: f32,
}

fn load_config(mut commands: Commands) {
    let config = std::fs::read_to_string("assets/particles/hit_burst.ron")
        .ok()
        .and_then(|s| ron::from_str::<HitParticleConfig>(&s).ok())
        .unwrap_or_default();
    commands.insert_resource(config);
}

#[derive(Resource, Deserialize, Clone)]
pub struct PlayerDeathParticleConfig {
    pub count: u32,
    pub speed: f32,
    pub lifetime: f32,
    pub start_size: f32,
    pub end_size: f32,
    pub elevation: f32,
    pub color: String,
}

impl Default for PlayerDeathParticleConfig {
    fn default() -> Self {
        Self {
            count: 10,
            speed: 250.0,
            lifetime: 0.5,
            start_size: 80.0,
            end_size: 0.0,
            elevation: 0.5,
            color: "player".into(),
        }
    }
}

fn load_player_death_config(mut commands: Commands) {
    let config = std::fs::read_to_string("assets/particles/player_death_burst.ron")
        .ok()
        .and_then(|s| ron::from_str::<PlayerDeathParticleConfig>(&s).ok())
        .unwrap_or_default();
    commands.insert_resource(config);
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform) in &mut query {
        particle.remaining -= dt;
        if particle.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += particle.velocity * dt;
        let t = 1.0 - (particle.remaining / particle.lifetime).clamp(0.0, 1.0);
        let scale = particle.start_scale + (particle.end_scale - particle.start_scale) * t;
        transform.scale = Vec3::splat(scale.max(0.01));
    }
}
