use gunship::math::{PI, TAU};

/// Eases out with elastic effect.
///
/// Start: 0, end: 1, Input range: [0, 1]
pub fn ease_out_elastic(t: f32) -> f32 {
    // TODO: Use epsilon for these tests.
    // TODO: Are these tests needed? Or can we support value for `t` outside the [0, 1] range?
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }

    const P: f32 = 0.3;
    const S: f32 = P / TAU * 1.57079633;

	f32::powf(2.0, -10.0 * t) * ((t - S) * TAU / P).sin() + 1.0
}

/// Eases out, overshooting slightly before returning.
///
/// Start: 0, end: 1, Input range: [0, 1]
pub fn ease_out_back(t: f32) -> f32 {
    let f = 1.0 - t;
    return 1.0 - (f * f * f - f * f32::sin(f * PI));
}
