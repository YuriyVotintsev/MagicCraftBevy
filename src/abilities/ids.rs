#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AbilityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TriggerTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ActionTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ActionDefId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TriggerDefId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeDefId(pub u32);


impl From<u32> for AbilityId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for TriggerTypeId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for ActionTypeId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for NodeTypeId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for ActionDefId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for TriggerDefId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for NodeDefId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}


impl From<AbilityId> for u32 {
    fn from(id: AbilityId) -> Self {
        id.0
    }
}

impl From<TriggerTypeId> for u32 {
    fn from(id: TriggerTypeId) -> Self {
        id.0
    }
}

impl From<ActionTypeId> for u32 {
    fn from(id: ActionTypeId) -> Self {
        id.0
    }
}

impl From<NodeTypeId> for u32 {
    fn from(id: NodeTypeId) -> Self {
        id.0
    }
}

impl From<ActionDefId> for u32 {
    fn from(id: ActionDefId) -> Self {
        id.0
    }
}

impl From<TriggerDefId> for u32 {
    fn from(id: TriggerDefId) -> Self {
        id.0
    }
}

impl From<NodeDefId> for u32 {
    fn from(id: NodeDefId) -> Self {
        id.0
    }
}

