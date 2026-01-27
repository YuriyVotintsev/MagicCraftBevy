use avian2d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    PlayerProjectile,
    EnemyProjectile,
    Wall,
}
