use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use super::{ghost, jumper, slime, spinner, tower};

#[derive(Asset, Resource, TypePath, Clone, Deserialize, Debug)]
pub struct MobsBalance {
    pub ghost: ghost::GhostStats,
    pub tower: tower::TowerStats,
    pub slime_small: slime::SlimeSmallStats,
    pub jumper: jumper::JumperStats,
    pub spinner: spinner::SpinnerStats,
}
