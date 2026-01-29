#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AbilityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TriggerTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EffectTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ParamId(pub u32);


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

impl From<u32> for EffectTypeId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for ParamId {
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

impl From<EffectTypeId> for u32 {
    fn from(id: EffectTypeId) -> Self {
        id.0
    }
}

impl From<ParamId> for u32 {
    fn from(id: ParamId) -> Self {
        id.0
    }
}

