use super::balance::MobsBalance;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MobKind {
    Ghost,
    Tower,
    SlimeSmall,
    Spinner,
    Jumper,
}

impl MobKind {
    #[allow(dead_code)]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "ghost" => Some(MobKind::Ghost),
            "tower" => Some(MobKind::Tower),
            "slime_small" => Some(MobKind::SlimeSmall),
            "spinner" => Some(MobKind::Spinner),
            "jumper" => Some(MobKind::Jumper),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            MobKind::Ghost => "ghost",
            MobKind::Tower => "tower",
            MobKind::SlimeSmall => "slime_small",
            MobKind::Spinner => "spinner",
            MobKind::Jumper => "jumper",
        }
    }

    pub fn size(&self, mobs: &MobsBalance) -> f32 {
        match self {
            MobKind::Ghost => mobs.ghost.size,
            MobKind::Tower => mobs.tower.size,
            MobKind::SlimeSmall => mobs.slime_small.size,
            MobKind::Spinner => mobs.spinner.size,
            MobKind::Jumper => mobs.jumper.size,
        }
    }
}
