use bevy::prelude::*;
use magic_craft_macros::ability_component;


#[ability_component]
pub struct Size {
    pub value: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, sync_size_to_scale);
}

fn sync_size_to_scale(mut query: Query<(&Size, &mut Transform), Added<Size>>) {
    for (size, mut transform) in &mut query {
        transform.scale = Vec3::splat(size.value / 2.0);
    }
}
