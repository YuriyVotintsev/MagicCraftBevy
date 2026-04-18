use bevy::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
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

    pub fn from_pixel(v: Vec2, size: f32) -> Self {
        let sqrt3 = 3.0f32.sqrt();
        let q_f = (sqrt3 / 3.0 * v.x - v.y / 3.0) / size;
        let r_f = (v.y * 2.0 / 3.0) / size;
        let s_f = -q_f - r_f;
        let mut q = q_f.round() as i32;
        let mut r = r_f.round() as i32;
        let s = s_f.round() as i32;
        let q_diff = (q as f32 - q_f).abs();
        let r_diff = (r as f32 - r_f).abs();
        let s_diff = (s as f32 - s_f).abs();
        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        }
        HexCoord::new(q, r)
    }

    pub fn ring_radius(self) -> i32 {
        let s = -self.q - self.r;
        self.q.abs().max(self.r.abs()).max(s.abs())
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
