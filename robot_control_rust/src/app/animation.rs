//! Animation system for smooth UI transitions.
//!
//! Provides `Interpolate` trait, `Easing` functions (including cubic Bézier),
//! and `AnimationManager` for keyed, type-safe animations.

use egui::{Color32, Pos2, Vec2};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Interpolate trait
// ---------------------------------------------------------------------------

/// Trait for types that support linear interpolation.
pub trait Interpolate: Copy {
    /// Linearly interpolate between `self` and `other` by factor `t` (0.0..1.0).
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for Pos2 {
    fn lerp(self, other: Self, t: f32) -> Self {
        Pos2::new(self.x.lerp(other.x, t), self.y.lerp(other.y, t))
    }
}

impl Interpolate for Vec2 {
    fn lerp(self, other: Self, t: f32) -> Self {
        Vec2::new(self.x.lerp(other.x, t), self.y.lerp(other.y, t))
    }
}

impl Interpolate for Color32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        let [r1, g1, b1, a1] = self.to_array();
        let [r2, g2, b2, a2] = other.to_array();
        Color32::from_rgba_premultiplied(
            (r1 as f32).lerp(r2 as f32, t) as u8,
            (g1 as f32).lerp(g2 as f32, t) as u8,
            (b1 as f32).lerp(b2 as f32, t) as u8,
            (a1 as f32).lerp(a2 as f32, t) as u8,
        )
    }
}

// ---------------------------------------------------------------------------
// Easing
// ---------------------------------------------------------------------------

/// Cubic Bézier helper: compute X coordinate for parameter t.
fn cubic_bezier_x(x1: f32, x2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * x1 + 3.0 * mt * t * t * x2 + t * t * t
}

/// Cubic Bézier helper: compute Y coordinate for parameter t.
fn cubic_bezier_y(y1: f32, y2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * y1 + 3.0 * mt * t * t * y2 + t * t * t
}

/// Cubic Bézier helper: derivative of X with respect to t.
fn cubic_bezier_dx(x1: f32, x2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * x1 + 6.0 * mt * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2)
}

/// Easing functions for animation curves.
///
/// Includes 10 built-in curves plus custom cubic Bézier via `Bezier(x1, y1, x2, y2)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseOutBack,
    EaseOutElastic,
    EaseOutBounce,
    /// Custom cubic Bézier curve with control points (x1, y1, x2, y2).
    /// All values typically in 0..1 range; y values may overshoot for spring effects.
    Bezier(f32, f32, f32, f32),
}

impl Easing {
    /// Evaluate the easing function at parameter `t` (clamped to 0.0..1.0).
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::EaseOutBack => {
                let c1: f32 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Easing::EaseOutElastic => {
                if t <= 0.0 {
                    return 0.0;
                }
                if t >= 1.0 {
                    return 1.0;
                }
                let c4: f32 = (2.0 * std::f32::consts::PI) / 3.0;
                2.0f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
            Easing::EaseOutBounce => {
                let n1: f32 = 7.5625;
                let d1: f32 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t2 = t - 1.5 / d1;
                    n1 * t2 * t2 + 0.75
                } else if t < 2.5 / d1 {
                    let t2 = t - 2.25 / d1;
                    n1 * t2 * t2 + 0.9375
                } else {
                    let t2 = t - 2.625 / d1;
                    n1 * t2 * t2 + 0.984375
                }
            }
            Easing::Bezier(x1, y1, x2, y2) => {
                let t_solved = solve_bezier_t(*x1, *x2, t);
                cubic_bezier_y(*y1, *y2, t_solved)
            }
        }
    }
}

/// Solve for the Bézier parameter t that produces the given x value
/// using Newton-Raphson iteration.
fn solve_bezier_t(x1: f32, x2: f32, target_x: f32) -> f32 {
    let mut t = target_x; // initial guess
    for _ in 0..8 {
        let x_est = cubic_bezier_x(x1, x2, t);
        let dx = cubic_bezier_dx(x1, x2, t);
        if dx.abs() < 1e-10 {
            break;
        }
        t -= (x_est - target_x) / dx;
        t = t.clamp(0.0, 1.0);
    }
    t
}

// ---------------------------------------------------------------------------
// Animation<T>
// ---------------------------------------------------------------------------

/// A single keyed animation tracking a value from `from` to `to`.
struct Animation<T: Interpolate> {
    from: T,
    to: T,
    start_time: f64,
    duration: f32,
    easing: Easing,
}

impl<T: Interpolate> Animation<T> {
    fn value_at(&self, current_time: f64) -> T {
        if current_time.is_nan() {
            return self.to;
        }
        let elapsed = (current_time - self.start_time) as f32;
        if elapsed <= 0.0 {
            return self.from;
        }
        if elapsed >= self.duration {
            return self.to;
        }
        let t = self.easing.evaluate(elapsed / self.duration);
        self.from.lerp(self.to, t)
    }

    fn is_complete(&self, current_time: f64) -> bool {
        if current_time.is_nan() {
            return true;
        }
        (current_time - self.start_time) as f32 >= self.duration
    }
}

impl<T: Interpolate + PartialEq> Animation<T> {
    fn is_same_target(&self, target: T) -> bool {
        self.to == target
    }
}

// ---------------------------------------------------------------------------
// AnimationManager
// ---------------------------------------------------------------------------

/// Manages all active animations keyed by string identifiers.
///
/// Provides type-safe animation for `f32`, `Color32`, and `Pos2` values.
/// Each call to `animate_*` either starts a new animation or updates an
/// existing one if the target has changed.
pub struct AnimationManager {
    floats: HashMap<String, Animation<f32>>,
    colors: HashMap<String, Animation<Color32>>,
    positions: HashMap<String, Animation<Pos2>>,
}

impl AnimationManager {
    /// Create a new, empty animation manager.
    pub fn new() -> Self {
        Self {
            floats: HashMap::new(),
            colors: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    /// Animate a `f32` value toward `target`.
    ///
    /// - On first call for a given key, starts from `current`.
    /// - If the target changes mid-animation, smoothly redirects from the
    ///   current interpolated position.
    /// - If the animation is complete and the target hasn't changed, returns
    ///   the final value without restarting.
    pub fn animate_float(
        &mut self,
        key: String,
        target: f32,
        current: f32,
        duration: f32,
        easing: Easing,
        current_time: f64,
    ) -> f32 {
        if let Some(anim) = self.floats.get(&key) {
            if anim.is_complete(current_time) && anim.is_same_target(target) {
                return target;
            }
        }

        let from = if let Some(anim) = self.floats.get(&key) {
            anim.value_at(current_time)
        } else {
            current
        };

        self.floats.insert(
            key,
            Animation {
                from,
                to: target,
                start_time: current_time,
                duration,
                easing,
            },
        );

        from
    }

    /// Animate a `Color32` value toward `target`.
    pub fn animate_color(
        &mut self,
        key: String,
        target: Color32,
        current: Color32,
        duration: f32,
        easing: Easing,
        current_time: f64,
    ) -> Color32 {
        if let Some(anim) = self.colors.get(&key) {
            if anim.is_complete(current_time) && anim.is_same_target(target) {
                return target;
            }
        }

        let from = if let Some(anim) = self.colors.get(&key) {
            anim.value_at(current_time)
        } else {
            current
        };

        self.colors.insert(
            key,
            Animation {
                from,
                to: target,
                start_time: current_time,
                duration,
                easing,
            },
        );

        from
    }

    /// Animate a `Pos2` value toward `target`.
    pub fn animate_position(
        &mut self,
        key: String,
        target: Pos2,
        current: Pos2,
        duration: f32,
        easing: Easing,
        current_time: f64,
    ) -> Pos2 {
        if let Some(anim) = self.positions.get(&key) {
            if anim.is_complete(current_time) && anim.is_same_target(target) {
                return target;
            }
        }

        let from = if let Some(anim) = self.positions.get(&key) {
            anim.value_at(current_time)
        } else {
            current
        };

        self.positions.insert(
            key,
            Animation {
                from,
                to: target,
                start_time: current_time,
                duration,
                easing,
            },
        );

        from
    }

    /// Remove all completed animations to free memory.
    pub fn cleanup_completed(&mut self, current_time: f64) {
        self.floats.retain(|_, a| !a.is_complete(current_time));
        self.colors.retain(|_, a| !a.is_complete(current_time));
        self.positions.retain(|_, a| !a.is_complete(current_time));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // === Layer 1: Interpolate ===

    #[test]
    fn f32_lerp_at_zero() {
        assert_eq!(0.0f32.lerp(100.0, 0.0), 0.0);
    }

    #[test]
    fn f32_lerp_at_one() {
        assert_eq!(0.0f32.lerp(100.0, 1.0), 100.0);
    }

    #[test]
    fn f32_lerp_at_half() {
        assert!((0.0f32.lerp(100.0, 0.5) - 50.0).abs() < 1e-6);
    }

    #[test]
    fn color32_lerp_red_to_blue() {
        let red = Color32::from_rgb(255, 0, 0);
        let blue = Color32::from_rgb(0, 0, 255);
        let mid = red.lerp(blue, 0.5);
        let [r, g, b, _] = mid.to_array();
        assert!(r > 120 && r < 136, "r={}", r);
        assert_eq!(g, 0);
        assert!(b > 120 && b < 136, "b={}", b);
    }

    // === Layer 1: Easing ===

    #[test]
    fn linear_returns_input() {
        assert!((Easing::Linear.evaluate(0.0)).abs() < 1e-6);
        assert!((Easing::Linear.evaluate(0.5) - 0.5).abs() < 1e-6);
        assert!((Easing::Linear.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_at_boundaries() {
        let e = Easing::EaseIn;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_at_boundaries() {
        let e = Easing::EaseOut;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_out_at_boundaries() {
        let e = Easing::EaseInOut;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_cubic_at_boundaries() {
        let e = Easing::EaseInCubic;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_cubic_at_boundaries() {
        let e = Easing::EaseOutCubic;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_cubic_is_monotonic() {
        let e = Easing::EaseOutCubic;
        let mut prev = 0.0f32;
        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let val = e.evaluate(t);
            assert!(val >= prev, "Not monotonic at t={}: {} < {}", t, val, prev);
            prev = val;
        }
    }

    #[test]
    fn ease_in_out_cubic_at_boundaries() {
        let e = Easing::EaseInOutCubic;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_back_exceeds_one() {
        let e = Easing::EaseOutBack;
        let mut exceeded = false;
        for i in 1..100 {
            let t = i as f32 / 100.0;
            if e.evaluate(t) > 1.0 {
                exceeded = true;
                break;
            }
        }
        assert!(exceeded, "EaseOutBack should overshoot 1.0");
    }

    #[test]
    fn ease_out_elastic_at_boundaries() {
        let e = Easing::EaseOutElastic;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_bounce_at_half() {
        let e = Easing::EaseOutBounce;
        let val = e.evaluate(0.5);
        assert!(val > 0.0 && val < 1.0, "val={}", val);
    }

    #[test]
    fn bezier_standard_curve() {
        let e = Easing::Bezier(0.25, 0.1, 0.25, 1.0);
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
        let mid = e.evaluate(0.5);
        assert!(mid > 0.3 && mid < 0.9, "mid={}", mid);
    }

    #[test]
    fn bezier_roundtrip() {
        for i in 1..10 {
            let t = i as f32 / 10.0;
            let x = cubic_bezier_x(0.42, 0.58, t);
            let solved = solve_bezier_t(0.42, 0.58, x);
            assert!((solved - t).abs() < 1e-4, "t={} solved={}", t, solved);
        }
    }

    // === Layer 2: AnimationManager ===

    #[test]
    fn new_animation_returns_from_value() {
        let mut mgr = AnimationManager::new();
        let val = mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        assert!((val - 0.0).abs() < 1e-6);
    }

    #[test]
    fn animation_completes_at_to_value() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("t".into(), 100.0, 0.0, 0.5, Easing::Linear, 0.0);
        let val = mgr.animate_float("t".into(), 100.0, 0.0, 0.5, Easing::Linear, 1.0);
        assert!((val - 100.0).abs() < 1e-6);
    }

    #[test]
    fn animation_midpoint_interpolates() {
        let mut mgr = AnimationManager::new();
        // First call: starts animation, returns `from`
        mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        // Second call at t=0.5 (halfway through duration=1.0) → ~50.0
        let val = mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.5);
        assert!((val - 50.0).abs() < 1.0, "val={}", val);
    }

    #[test]
    fn two_animations_different_keys() {
        let mut mgr = AnimationManager::new();
        let a = mgr.animate_float("a".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        let b = mgr.animate_float("b".into(), 200.0, 0.0, 1.0, Easing::Linear, 0.0);
        assert!((a - 0.0).abs() < 1e-6);
        assert!((b - 0.0).abs() < 1e-6);
    }

    #[test]
    fn same_key_restarts_on_new_target() {
        let mut mgr = AnimationManager::new();
        let v1 = mgr.animate_float("x".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        assert!((v1 - 0.0).abs() < 1e-6);
        // New target
        let v2 = mgr.animate_float("x".into(), 200.0, 0.0, 1.0, Easing::Linear, 0.0);
        assert!((v2 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn color_animation_interpolates() {
        let mut mgr = AnimationManager::new();
        let red = Color32::from_rgb(255, 0, 0);
        let blue = Color32::from_rgb(0, 0, 255);
        // First call: starts animation, returns `from` (red)
        mgr.animate_color("c".into(), blue, red, 1.0, Easing::Linear, 0.0);
        // Second call at t=0.5 → interpolated between red and blue
        let mid = mgr.animate_color("c".into(), blue, red, 1.0, Easing::Linear, 0.5);
        let [r, _, b, _] = mid.to_array();
        assert!(r > 120 && r < 136, "r={}", r);
        assert!(b > 120 && b < 136, "b={}", b);
    }

    #[test]
    fn no_restart_when_target_unchanged_after_completion() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("s".into(), 50.0, 0.0, 0.5, Easing::Linear, 0.0);
        let val = mgr.animate_float("s".into(), 50.0, 0.0, 0.5, Easing::Linear, 2.0);
        assert!((val - 50.0).abs() < 1e-6);
    }

    #[test]
    fn cleanup_removes_completed() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("d".into(), 100.0, 0.0, 0.1, Easing::Linear, 0.0);
        // Wait for completion
        let val = mgr.animate_float("d".into(), 100.0, 0.0, 0.1, Easing::Linear, 1.0);
        assert!((val - 100.0).abs() < 1e-6);
        mgr.cleanup_completed(1.0);
        // After cleanup, "d" is gone. New call with current=42 starts fresh
        let val = mgr.animate_float("d".into(), 200.0, 42.0, 0.5, Easing::Linear, 1.0);
        assert!((val - 42.0).abs() < 1e-6, "val={}", val);
    }

    // === Layer 5: Frontend-Backend Integration ===

    #[test]
    fn serial_data_triggers_smooth_gauge() {
        let mut mgr = AnimationManager::new();
        let v1 = mgr.animate_float("gauge".into(), 50.0, 0.0, 0.5, Easing::EaseOutCubic, 0.0);
        assert!((v1 - 0.0).abs() < 1e-6);
        // New data mid-animation
        let v2 = mgr.animate_float("gauge".into(), 80.0, 0.0, 0.5, Easing::EaseOutCubic, 0.2);
        assert!((0.0..=80.0).contains(&v2), "v2={}", v2);
    }

    #[test]
    fn rapid_data_updates_coalesce() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("r".into(), 10.0, 0.0, 1.0, Easing::Linear, 0.0);
        mgr.animate_float("r".into(), 20.0, 0.0, 1.0, Easing::Linear, 0.1);
        mgr.animate_float("r".into(), 30.0, 0.0, 1.0, Easing::Linear, 0.2);
        let val = mgr.animate_float("r".into(), 40.0, 0.0, 1.0, Easing::Linear, 0.3);
        assert!((0.0..=40.0).contains(&val), "val={}", val);
    }

    #[test]
    fn disconnect_reconnect_state_sync() {
        let mut mgr = AnimationManager::new();
        let green = Color32::from_rgb(80, 200, 120);
        let red = Color32::from_rgb(220, 80, 80);
        mgr.animate_color("link".into(), green, red, 0.3, Easing::EaseOut, 0.0);
        let c = mgr.animate_color("link".into(), red, green, 0.3, Easing::EaseOut, 1.0);
        assert_eq!(c, green); // just started reconnect anim
        let c = mgr.animate_color("link".into(), red, green, 0.3, Easing::EaseOut, 1.3);
        assert_eq!(c, red);
    }

    #[test]
    fn no_data_loss_during_animation() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("s".into(), 50.0, 0.0, 1.0, Easing::Linear, 0.0);
        let val = mgr.animate_float("s".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.3);
        assert!((0.0..=100.0).contains(&val), "val={}", val);
    }

    #[test]
    fn animation_target_matches_latest_data() {
        let mut mgr = AnimationManager::new();
        for (i, target) in [10.0f32, 20.0, 30.0, 40.0].iter().enumerate() {
            let t = i as f64 * 0.05;
            mgr.animate_float("stream".into(), *target, 0.0, 0.5, Easing::Linear, t);
        }
        let val = mgr.animate_float("stream".into(), 40.0, 0.0, 0.5, Easing::Linear, 2.0);
        assert!((val - 40.0).abs() < 1e-6);
    }

    #[test]
    fn error_state_propagates_with_animation() {
        let mut mgr = AnimationManager::new();
        let normal = Color32::from_rgb(100, 100, 110);
        let error = Color32::from_rgb(255, 80, 80);
        let c = mgr.animate_color("st".into(), error, normal, 0.3, Easing::EaseOut, 0.0);
        assert_eq!(c, normal);
        let c = mgr.animate_color("st".into(), error, normal, 0.3, Easing::EaseOut, 0.3);
        assert_eq!(c, error);
    }

    #[test]
    fn high_frequency_data_graceful() {
        let mut mgr = AnimationManager::new();
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let target = (i as f32).sin() * 50.0 + 50.0;
            let _ = mgr.animate_float("hf".into(), target, 0.0, 0.1, Easing::Linear, t);
        }
        let val = mgr.animate_float("hf".into(), 50.0, 0.0, 0.1, Easing::Linear, 1.0);
        assert!(val.is_finite(), "val={}", val);
    }

    #[test]
    fn button_press_feedback() {
        let mut mgr = AnimationManager::new();
        let v1 = mgr.animate_float("btn".into(), 0.95, 1.0, 0.1, Easing::EaseOutBack, 0.0);
        assert!((v1 - 1.0).abs() < 1e-6);
        let v2 = mgr.animate_float("btn".into(), 1.0, 0.95, 0.2, Easing::EaseOutBack, 0.15);
        assert!((0.95..=1.05).contains(&v2), "v2={}", v2);
    }

    // === Layer 2b: NaN / Infinity / Edge Cases ===

    #[test]
    fn nan_time_returns_to_value() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        let val = mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, f64::NAN);
        assert!(
            (val - 100.0).abs() < 1e-6,
            "NaN should return to value: {}",
            val
        );
    }

    #[test]
    fn infinity_time_returns_to_value() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        let val = mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, f64::INFINITY);
        assert!(
            (val - 100.0).abs() < 1e-6,
            "Inf should return to value: {}",
            val
        );
    }

    #[test]
    fn neg_infinity_time_returns_from_value() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("t".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        // -Inf elapsed = -Inf <= 0.0 -> returns from (0.0)
        let val = mgr.animate_float(
            "t".into(),
            100.0,
            0.0,
            1.0,
            Easing::Linear,
            f64::NEG_INFINITY,
        );
        assert!(
            (val - 0.0).abs() < 1e-6,
            "-Inf elapsed is -Inf <= 0, returns from: {}",
            val
        );
    }

    #[test]
    fn nan_color_returns_to_value() {
        let mut mgr = AnimationManager::new();
        let red = Color32::from_rgb(255, 0, 0);
        let blue = Color32::from_rgb(0, 0, 255);
        mgr.animate_color("c".into(), blue, red, 1.0, Easing::Linear, 0.0);
        let val = mgr.animate_color("c".into(), blue, red, 1.0, Easing::Linear, f64::NAN);
        assert_eq!(val, blue, "NaN color should return to value");
    }

    #[test]
    fn nan_is_complete_returns_true() {
        let anim = Animation {
            from: 0.0f32,
            to: 100.0,
            start_time: 0.0,
            duration: 1.0,
            easing: Easing::Linear,
        };
        assert!(
            anim.is_complete(f64::NAN),
            "NaN time should be considered complete"
        );
        assert!(
            anim.is_complete(f64::INFINITY),
            "Inf time should be considered complete"
        );
    }

    #[test]
    fn negative_duration_is_instant() {
        let mut mgr = AnimationManager::new();
        // First call creates animation, returns from
        let v1 = mgr.animate_float("t".into(), 100.0, 0.0, -1.0, Easing::Linear, 0.0);
        assert!((v1 - 0.0).abs() < 1e-6, "First call returns from: {}", v1);
        // Second call: elapsed=0.5 >= duration(-1.0) -> instant completion -> returns to
        let v2 = mgr.animate_float("t".into(), 100.0, 0.0, -1.0, Easing::Linear, 0.5);
        assert!(
            (v2 - 100.0).abs() < 1e-6,
            "Negative duration should be instant: {}",
            v2
        );
    }

    #[test]
    fn zero_duration_is_instant() {
        let mut mgr = AnimationManager::new();
        // First call creates animation, returns from
        let v1 = mgr.animate_float("t".into(), 100.0, 0.0, 0.0, Easing::Linear, 0.0);
        assert!((v1 - 0.0).abs() < 1e-6, "First call returns from: {}", v1);
        // Second call: elapsed=0.001 >= duration(0.0) -> instant -> returns to
        let v2 = mgr.animate_float("t".into(), 100.0, 0.0, 0.0, Easing::Linear, 0.001);
        assert!(
            (v2 - 100.0).abs() < 1e-6,
            "Zero duration should be instant: {}",
            v2
        );
    }

    #[test]
    fn cleanup_handles_nan_time() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("t".into(), 100.0, 0.0, 0.1, Easing::Linear, 0.0);
        // NaN time should mark as complete so cleanup removes it
        mgr.cleanup_completed(f64::NAN);
        // After cleanup, requesting "t" should start fresh
        let val = mgr.animate_float("t".into(), 200.0, 50.0, 0.5, Easing::Linear, 0.0);
        assert!(
            (val - 50.0).abs() < 1e-6,
            "After NaN cleanup, should start fresh: {}",
            val
        );
    }

    // === Layer 4: Performance ===

    #[test]
    fn thousand_animations_under_5ms() {
        let mut mgr = AnimationManager::new();
        let start = std::time::Instant::now();
        for i in 0..1000 {
            let _ = mgr.animate_float(
                format!("a{}", i),
                i as f32,
                0.0,
                1.0,
                Easing::EaseOutCubic,
                0.5,
            );
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 5,
            "1000 animations took {:?}",
            elapsed
        );
    }

    #[test]
    fn bezier_under_500ns_per_call() {
        let e = Easing::Bezier(0.42, 0.0, 0.58, 1.0);
        let start = std::time::Instant::now();
        for i in 0..10_000 {
            let _ = e.evaluate(i as f32 / 10_000.0);
        }
        let per = start.elapsed() / 10_000;
        assert!(per.as_nanos() < 500, "Per call: {:?}", per);
    }

    #[test]
    fn cleanup_1000_under_1ms() {
        let mut mgr = AnimationManager::new();
        for i in 0..1000 {
            mgr.animate_float(format!("d{}", i), 100.0, 0.0, 0.1, Easing::Linear, 0.0);
            mgr.animate_float(format!("d{}", i), 100.0, 0.0, 0.1, Easing::Linear, 1.0);
        }
        let start = std::time::Instant::now();
        mgr.cleanup_completed(1.0);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 1, "Cleanup took {:?}", elapsed);
    }
}
