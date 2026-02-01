use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum NodeParamsRaw {
    Action(crate::building_blocks::actions::ActionParamsRaw),
    Trigger(crate::building_blocks::triggers::TriggerParamsRaw),
}

impl NodeParamsRaw {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Action(p) => p.name(),
            Self::Trigger(p) => p.name(),
        }
    }

    pub fn children(&self) -> &[NodeParamsRaw] {
        match self {
            Self::Action(p) => p.children(),
            Self::Trigger(p) => p.children(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NodeParams {
    Action(crate::building_blocks::actions::ActionParams),
    Trigger(crate::building_blocks::triggers::TriggerParams),
}

impl NodeParams {
    pub fn unwrap_action(&self) -> &crate::building_blocks::actions::ActionParams {
        match self {
            Self::Action(p) => p,
            Self::Trigger(_) => panic!("Expected Action params, got Trigger"),
        }
    }

    #[allow(dead_code)]
    pub fn unwrap_trigger(&self) -> &crate::building_blocks::triggers::TriggerParams {
        match self {
            Self::Trigger(p) => p,
            Self::Action(_) => panic!("Expected Trigger params, got Action"),
        }
    }
}
