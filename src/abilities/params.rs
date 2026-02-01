use std::collections::HashMap;

use crate::abilities::{ParamValueRaw, NodeKind};
use crate::stats::StatRegistry;

#[derive(Debug, Clone)]
pub enum NodeParams {
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
            NodeKind::Trigger => panic!("Triggers don't have params"),
            NodeKind::Action => Self::Action(
                crate::building_blocks::actions::NodeParams::parse(name, raw, stat_registry)
            ),
        }
    }

    #[inline]
    pub fn unwrap_action(&self) -> &crate::building_blocks::actions::NodeParams {
        let Self::Action(p) = self;
        p
    }
}
