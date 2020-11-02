use druid::Color;

pub const BACKGROUND: Color = Color::rgb8(0x22, 0x36, 0x43);
pub const TEXT: Color = Color::rgb8(0xfb, 0xf9, 0xf0);
pub const DIM_TEXT: Color = Color::rgb8(0x74, 0x80, 0x85);
pub const SUBTLE_ACCENT: Color = Color::rgb8(0x57, 0x9f, 0xb3);
pub const ACCENT: Color = Color::rgb8(0x1a, 0x22, 0x43);
pub const BOLD_ACCENT: Color = Color::rgb8(0xe8, 0x5f, 0x24);
pub const ERROR: Color = Color::rgb8(0xb0, 0x00, 0x20);

pub fn active(color: Color, text: Color) -> Color {
    mix(text, color, 0.32)
}

pub fn hot(color: Color, text: Color) -> Color {
    mix(text, color, 0.08)
}

pub fn disabled(color: Color) -> Color {
    let (r, g, b, a) = color.as_rgba();
    let grey = (r + b + g) / 3.0;
    Color::rgba(grey, grey, grey, a * 0.38)
}

fn mix(color1: Color, color2: Color, weight: f64) -> Color {
    let color1 = color1.as_rgba();
    let color2 = color2.as_rgba();

    let normalized_weight = weight * 2.0 - 1.0;
    let alpha_distance = color1.3 - color2.3;

    let mut combined_weight =
        (normalized_weight + alpha_distance) / (1.0 + normalized_weight * alpha_distance);
    if !combined_weight.is_finite() {
        combined_weight = normalized_weight;
    }

    let weight1 = (combined_weight + 1.0) / 2.0;
    let weight2 = 1.0 - weight1;

    Color::rgba(
        color1.0 * weight1 + color2.0 * weight2,
        color1.1 * weight1 + color2.1 * weight2,
        color1.2 * weight1 + color2.2 * weight2,
        color1.3 * weight + color2.3 * (1.0 - weight),
    )
}
