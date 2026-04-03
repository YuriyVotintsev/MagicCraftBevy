use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScenarioDef {
    #[serde(default = "default_time_scale")]
    pub time_scale: f32,
    #[serde(default = "default_timeout")]
    pub timeout: f32,
    pub steps: Vec<Step>,
}

fn default_time_scale() -> f32 {
    1.0
}
fn default_timeout() -> f32 {
    60.0
}

#[derive(Deserialize, Clone)]
pub struct Step {
    pub at: f32,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub assert: Vec<Assertion>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Action {
    MoveDir(f32, f32),
    StopMove,
    Press(String),
    Release(String),
    ReleaseAll,
    SetTimeScale(f32),
    DumpState,
    Log(String),
}

#[derive(Deserialize, Clone, Debug)]
pub enum Assertion {
    PlayerAlive,
    PlayerDead,
    PlayerHealth(Cmp, f32),
    PlayerPosX(Cmp, f32),
    PlayerPosY(Cmp, f32),
    MobCount(Cmp, usize),
    DumpState,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Cmp {
    Gt,
    Lt,
    Eq,
    Gte,
    Lte,
}

impl Cmp {
    pub fn check_f32(&self, actual: f32, expected: f32) -> bool {
        match self {
            Cmp::Gt => actual > expected,
            Cmp::Lt => actual < expected,
            Cmp::Eq => (actual - expected).abs() < 0.001,
            Cmp::Gte => actual >= expected,
            Cmp::Lte => actual <= expected,
        }
    }

    pub fn check_usize(&self, actual: usize, expected: usize) -> bool {
        match self {
            Cmp::Gt => actual > expected,
            Cmp::Lt => actual < expected,
            Cmp::Eq => actual == expected,
            Cmp::Gte => actual >= expected,
            Cmp::Lte => actual <= expected,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Cmp::Gt => ">",
            Cmp::Lt => "<",
            Cmp::Eq => "==",
            Cmp::Gte => ">=",
            Cmp::Lte => "<=",
        }
    }
}
