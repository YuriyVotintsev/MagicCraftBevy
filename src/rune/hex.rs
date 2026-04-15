use bevy::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const CENTER: Self = HexCoord { q: 0, r: 0 };

    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn s(self) -> i32 {
        -self.q - self.r
    }

    pub fn distance(self, other: Self) -> i32 {
        ((self.q - other.q).abs()
            + (self.r - other.r).abs()
            + (self.s() - other.s()).abs())
            / 2
    }

    pub fn ring(self) -> i32 {
        self.distance(Self::CENTER)
    }

    pub fn to_pixel(self, size: f32) -> Vec2 {
        let sqrt3 = 3.0f32.sqrt();
        let x = size * (sqrt3 * self.q as f32 + sqrt3 * 0.5 * self.r as f32);
        let y = size * 1.5 * self.r as f32;
        Vec2::new(x, y)
    }

    pub fn all_within_radius(radius: i32) -> Vec<HexCoord> {
        let mut result = Vec::new();
        for q in -radius..=radius {
            let r_min = (-radius).max(-q - radius);
            let r_max = radius.min(-q + radius);
            for r in r_min..=r_max {
                result.push(HexCoord::new(q, r));
            }
        }
        result
    }

    pub fn neighbors(self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q - 1, self.r + 1),
            HexCoord::new(self.q, self.r + 1),
        ]
    }
}
