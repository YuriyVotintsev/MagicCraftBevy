use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AbilityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ActivatorTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EffectTypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ParamId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TagId(pub u32);

impl From<u32> for AbilityId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u32> for ActivatorTypeId {
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

impl From<u32> for TagId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<AbilityId> for u32 {
    fn from(id: AbilityId) -> Self {
        id.0
    }
}

impl From<ActivatorTypeId> for u32 {
    fn from(id: ActivatorTypeId) -> Self {
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

impl From<TagId> for u32 {
    fn from(id: TagId) -> Self {
        id.0
    }
}
