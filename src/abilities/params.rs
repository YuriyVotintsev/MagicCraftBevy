use std::collections::HashMap;

use crate::abilities::{ParamValueRaw, NodeKind};
use crate::stats::StatRegistry;

#[derive(Debug, Clone)]
pub enum NodeParams {
    Trigger(crate::building_blocks::triggers::NodeParams),
    Action(crate::building_blocks::actions::NodeParams),
}

impl NodeParams {
    pub fn parse(
        kind: NodeKind,
        name: &str,
        raw: &HashMap<String, ParamValueRaw>,
        stat_registry: &StatRegistry,
    ) -> Self {
        match kind {
            NodeKind::Trigger => Self::Trigger(
                crate::building_blocks::triggers::NodeParams::parse(name, raw, stat_registry)
            ),
            NodeKind::Action => Self::Action(
                crate::building_blocks::actions::NodeParams::parse(name, raw, stat_registry)
            ),
        }
    }

    pub fn expect_trigger(&self) -> &crate::building_blocks::triggers::NodeParams {
        match self {
            Self::Trigger(p) => p,
            _ => panic!("Expected Trigger params"),
        }
    }

    pub fn expect_action(&self) -> &crate::building_blocks::actions::NodeParams {
        match self {
            Self::Action(p) => p,
            _ => panic!("Expected Action params"),
        }
    }
}
