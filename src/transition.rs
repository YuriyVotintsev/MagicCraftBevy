use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::palette;
use crate::GameState;

pub const CLOSE_DURATION: f32 = 0.35;
pub const OPEN_DURATION: f32 = 0.35;
const MAX_RADIUS: f32 = 1.2;

#[derive(ShaderType, Clone)]
pub struct IrisData {
    pub color: LinearRgba,
    pub radius: f32,
    pub softness: f32,
    pub _pad0: f32,
    pub _pad1: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct IrisMaterial {
    #[uniform(0)]
    pub data: IrisData,
}

impl UiMaterial for IrisMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/transition_iris.wgsl".into()
    }
}

#[derive(Clone, Debug)]
pub enum TransitionAction {
    Game(GameState),
}

#[derive(Resource, Default, Debug)]
pub enum Transition {
    #[default]
    Idle,
    Closing {
        action: TransitionAction,
        elapsed: f32,
    },
    Opening {
        elapsed: f32,
    },
}

impl Transition {
    pub fn request(&mut self, action: TransitionAction) {
        if !matches!(self, Transition::Idle) {
            return;
        }
        *self = Transition::Closing {
            action,
            elapsed: 0.0,
        };
    }
}

#[derive(Component)]
struct TransitionOverlay {
    material: Handle<IrisMaterial>,
}

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<IrisMaterial>::default())
            .init_resource::<Transition>()
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, update_transition);
    }
}

fn spawn_overlay(mut commands: Commands, mut materials: ResMut<Assets<IrisMaterial>>) {
    let handle = materials.add(IrisMaterial {
        data: IrisData {
            color: palette::color("ui_overlay_bg").to_linear(),
            radius: MAX_RADIUS,
            softness: 0.01,
            _pad0: 0.0,
            _pad1: 0.0,
        },
    });
    commands.spawn((
        Name::new("TransitionOverlay"),
        TransitionOverlay {
            material: handle.clone(),
        },
        MaterialNode(handle),
        GlobalZIndex(10000),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
}

fn update_transition(
    time: Res<Time<Real>>,
    mut transition: ResMut<Transition>,
    mut game_state: ResMut<NextState<GameState>>,
    overlay: Query<&TransitionOverlay>,
    mut materials: ResMut<Assets<IrisMaterial>>,
) {
    let dt = time.delta_secs();
    let radius = match *transition {
        Transition::Idle => MAX_RADIUS,
        Transition::Closing {
            ref action,
            ref mut elapsed,
        } => {
            *elapsed += dt;
            let t = (*elapsed / CLOSE_DURATION).clamp(0.0, 1.0);
            let r = (1.0 - t) * MAX_RADIUS;
            if t >= 1.0 {
                match action.clone() {
                    TransitionAction::Game(s) => game_state.set(s),
                }
                *transition = Transition::Opening { elapsed: 0.0 };
            }
            r
        }
        Transition::Opening { ref mut elapsed } => {
            *elapsed += dt;
            let t = (*elapsed / OPEN_DURATION).clamp(0.0, 1.0);
            let r = t * MAX_RADIUS;
            if t >= 1.0 {
                *transition = Transition::Idle;
            }
            r
        }
    };
    for overlay in &overlay {
        if let Some(m) = materials.get_mut(&overlay.material) {
            m.data.radius = radius;
        }
    }
}
