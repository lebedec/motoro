use crate::math::Vec4;

pub trait Colors {
    fn to_vec4(&self) -> Vec4;
}

impl Colors for Vec4 {
    #[inline(always)]
    fn to_vec4(&self) -> Vec4 {
        *self
    }
}

impl Colors for [u8; 4] {
    #[inline(always)]
    fn to_vec4(&self) -> Vec4 {
        let [r, g, b, a] = *self;
        [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ]
    }
}

impl Colors for (f32, f32, f32, f32) {
    #[inline(always)]
    fn to_vec4(&self) -> Vec4 {
        let (r, g, b, a) = *self;
        [r, g, b, a]
    }
}

impl Colors for &str {
    fn to_vec4(&self) -> Vec4 {
        match *self {
            "white" => [1.0, 1.0, 1.0, 1.0],
            "red" => [1.0, 0.0, 0.0, 1.0],
            "transparent" => [0.0, 0.0, 0.0, 0.0],
            "none" => [0.0, 0.0, 0.0, 0.0],
            value if value.starts_with("#") && value.len() == 7 => {
                let r = u8::from_str_radix(&value[1..3], 16).unwrap_or(0);
                let g = u8::from_str_radix(&value[3..5], 16).unwrap_or(0);
                let b = u8::from_str_radix(&value[5..7], 16).unwrap_or(0);
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
            }
            value if value.starts_with("#") && value.len() == 9 => {
                let r = u8::from_str_radix(&value[1..3], 16).unwrap_or(0);
                let g = u8::from_str_radix(&value[3..5], 16).unwrap_or(0);
                let b = u8::from_str_radix(&value[5..7], 16).unwrap_or(0);
                let a = u8::from_str_radix(&value[7..9], 16).unwrap_or(0);
                [
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ]
            }
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }
}
