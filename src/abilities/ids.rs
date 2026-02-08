#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AbilityId(pub u32);

impl From<u32> for AbilityId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<AbilityId> for u32 {
    fn from(id: AbilityId) -> Self {
        id.0
    }
}
