use std::collections::{HashMap, HashSet};

use bevy::asset::{Asset, io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use rand::Rng;
use serde::Deserialize;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleRegistry>()
            .init_resource::<ParticleMaterialCache>()
            .add_systems(
                Update,
                (process_stop_markers, emit_particles, update_particles).chain(),
            );
    }
}

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ParticleConfigRaw {
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default)]
    pub spawn_rate: Option<f32>,
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub lifetime: Option<f32>,
    #[serde(default)]
    pub start_size: Option<f32>,
    #[serde(default)]
    pub end_size: Option<f32>,
    #[serde(default)]
    pub elevation: Option<f32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub vertical_speed: Option<f32>,
    #[serde(default)]
    pub shape: Option<SpawnShape>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum SpawnShape {
    Point,
    Circle(f32),
}

impl Default for SpawnShape {
    fn default() -> Self {
        Self::Point
    }
}

#[derive(Clone, Debug)]
pub struct ParticleConfig {
    pub count: u32,
    pub spawn_rate: f32,
    pub speed: f32,
    pub vertical_speed: f32,
    pub lifetime: f32,
    pub start_size: f32,
    pub end_size: f32,
    pub elevation: f32,
    pub color: String,
    pub shape: SpawnShape,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            count: 1,
            spawn_rate: 0.0,
            speed: 100.0,
            vertical_speed: 0.0,
            lifetime: 0.3,
            start_size: 20.0,
            end_size: 0.0,
            elevation: 0.0,
            color: "enemy".into(),
            shape: SpawnShape::Point,
        }
    }
}

#[derive(Resource, Default)]
pub struct ParticleRegistry {
    configs: HashMap<String, ParticleConfig>,
}

impl ParticleRegistry {
    pub fn get(&self, name: &str) -> Option<&ParticleConfig> {
        self.configs.get(name)
    }

    pub fn resolve_all(raws: &HashMap<String, ParticleConfigRaw>) -> Self {
        let mut registry = Self::default();
        for name in raws.keys() {
            let resolved = Self::resolve_one(name, raws, &mut HashSet::new());
            registry.configs.insert(name.clone(), resolved);
        }
        registry
    }

    fn resolve_one(
        name: &str,
        raws: &HashMap<String, ParticleConfigRaw>,
        visited: &mut HashSet<String>,
    ) -> ParticleConfig {
        if visited.contains(name) {
            warn!("Circular particle config inheritance: {}", name);
            return ParticleConfig::default();
        }
        visited.insert(name.to_string());

        let Some(raw) = raws.get(name) else {
            warn!("Particle config not found: {}", name);
            return ParticleConfig::default();
        };

        let mut base = if let Some(parent_name) = &raw.parent {
            Self::resolve_one(parent_name, raws, visited)
        } else {
            ParticleConfig::default()
        };

        if let Some(v) = raw.count { base.count = v; }
        if let Some(v) = raw.spawn_rate { base.spawn_rate = v; }
        if let Some(v) = raw.speed { base.speed = v; }
        if let Some(v) = raw.vertical_speed { base.vertical_speed = v; }
        if let Some(v) = raw.lifetime { base.lifetime = v; }
        if let Some(v) = raw.start_size { base.start_size = v; }
        if let Some(v) = raw.end_size { base.end_size = v; }
        if let Some(v) = raw.elevation { base.elevation = v; }
        if let Some(ref v) = raw.color { base.color = v.clone(); }
        if let Some(ref v) = raw.shape { base.shape = v.clone(); }

        base
    }
}

#[derive(Resource, Default)]
pub struct ParticleMaterialCache {
    mesh: Option<Handle<Mesh>>,
    materials: HashMap<String, Handle<StandardMaterial>>,
}

impl ParticleMaterialCache {
    fn get_mesh(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.mesh
            .get_or_insert_with(|| meshes.add(Sphere::new(0.5)))
            .clone()
    }

    fn get_material(
        &mut self,
        color_name: &str,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.materials
            .entry(color_name.to_string())
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: crate::palette::color(color_name),
                    unlit: true,
                    ..default()
                })
            })
            .clone()
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

#[derive(Component)]
pub struct ParticleEmitter {
    pub config_name: String,
    pub accumulator: f32,
    pub fired: bool,
    pub stopped: bool,
    pub drain_timer: f32,
    pub shape_override: Option<SpawnShape>,
    pub material_override: Option<Handle<StandardMaterial>>,
}

#[derive(Component)]
pub struct StopEmitter;

pub fn start_particles(commands: &mut Commands, config: &str, position: Vec2) -> Entity {
    let spawn_pos = crate::coord::ground_pos(position);
    commands
        .spawn((
            ParticleEmitter {
                config_name: config.to_string(),
                accumulator: 0.0,
                fired: false,
                stopped: false,
                drain_timer: 0.0,
                shape_override: None,
                material_override: None,
            },
            Transform::from_translation(spawn_pos),
            Visibility::Hidden,
        ))
        .id()
}

pub fn stop_particles(commands: &mut Commands, entity: Entity) {
    if let Ok(mut ec) = commands.get_entity(entity) {
        ec.insert(StopEmitter);
    }
}

fn process_stop_markers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ParticleEmitter), With<StopEmitter>>,
    registry: Res<ParticleRegistry>,
) {
    for (entity, mut emitter) in &mut query {
        emitter.stopped = true;
        let lifetime = registry
            .get(&emitter.config_name)
            .map(|c| c.lifetime)
            .unwrap_or(1.0);
        emitter.drain_timer = lifetime;
        commands.entity(entity).remove::<StopEmitter>();
    }
}

fn emit_particles(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut ParticleEmitter, &Transform)>,
    registry: Res<ParticleRegistry>,
    mut cache: ResMut<ParticleMaterialCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut emitter, transform) in &mut query {
        let Some(config) = registry.get(&emitter.config_name) else {
            warn!("Unknown particle config: {}", emitter.config_name);
            commands.entity(entity).despawn();
            continue;
        };

        if emitter.fired || emitter.stopped {
            emitter.drain_timer -= dt;
            if emitter.drain_timer <= 0.0 {
                commands.entity(entity).despawn();
            }
            continue;
        }

        let is_burst = config.spawn_rate == 0.0;

        if is_burst {
            spawn_burst(
                &mut commands,
                config,
                &emitter,
                transform,
                &mut cache,
                &mut meshes,
                &mut materials,
                config.count,
            );
            emitter.fired = true;
            emitter.drain_timer = config.lifetime;
        } else {
            emitter.accumulator += dt;
            let interval = 1.0 / config.spawn_rate;
            while emitter.accumulator >= interval {
                emitter.accumulator -= interval;
                spawn_burst(
                    &mut commands,
                    config,
                    &emitter,
                    transform,
                    &mut cache,
                    &mut meshes,
                    &mut materials,
                    config.count,
                );
            }
        }
    }
}

fn spawn_burst(
    commands: &mut Commands,
    config: &ParticleConfig,
    emitter: &ParticleEmitter,
    transform: &Transform,
    cache: &mut ParticleMaterialCache,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    count: u32,
) {
    let mesh = cache.get_mesh(meshes);
    let material = emitter.material_override.clone()
        .unwrap_or_else(|| cache.get_material(&config.color, materials));
    let pos = transform.translation;

    let shape = emitter.shape_override.as_ref().unwrap_or(&config.shape);

    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = config.speed * rng.random_range(0.5..1.0);
        let dir = Vec2::new(angle.cos(), angle.sin());

        let start_scale = config.start_size;
        let end_scale = config.end_size;

        let shape_offset = match shape {
            SpawnShape::Point => Vec3::ZERO,
            SpawnShape::Circle(radius) => {
                let a = rng.random_range(0.0..std::f32::consts::TAU);
                Vec3::new(a.cos() * radius, 0.0, a.sin() * radius)
            }
        };

        let velocity = crate::coord::ground_vel(dir * speed)
            + Vec3::new(0.0, config.vertical_speed, 0.0);

        let spawn_pos = pos + shape_offset;

        commands.spawn((
            Particle {
                velocity,
                remaining: config.lifetime,
                lifetime: config.lifetime,
                start_scale,
                end_scale,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(spawn_pos.x, config.elevation + spawn_pos.y, spawn_pos.z))
                .with_scale(Vec3::splat(start_scale)),
        ));
    }
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

#[derive(Default, TypePath)]
pub struct ParticleConfigLoader;

impl AssetLoader for ParticleConfigLoader {
    type Asset = ParticleConfigRaw;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let config: ParticleConfigRaw = ron::from_str(content)?;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["particle.ron"]
    }
}
