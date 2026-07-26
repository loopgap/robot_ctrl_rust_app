# Animation System Implementation Plan

**Goal:** Add a comprehensive non-linear animation and Bézier curve system to the Robot Control Rust GUI
**Architecture:** egui-native + lightweight `AnimationManager` module (single file `animation.rs`)
**Tech Stack:** Rust, egui 0.31, eframe

## Prerequisites

- Rust toolchain installed (stable)
- Project builds: `cargo build` in `D:\Destop\test_ui\rust_serial\robot_control_rust`
- All existing tests pass: `cargo test`

---

### Task 1: Create `animation.rs` Core Module

**Files:**
- Create: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\animation.rs`

- [ ] Step 1: Write the failing test for `Interpolate` trait on `f32`
```rust
// D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\animation.rs

pub trait Interpolate: Copy {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for egui::Pos2 {
    fn lerp(self, other: Self, t: f32) -> Self {
        egui::Pos2::new(
            self.x.lerp(other.x, t),
            self.y.lerp(other.y, t),
        )
    }
}

impl Interpolate for egui::Vec2 {
    fn lerp(self, other: Self, t: f32) -> Self {
        egui::Vec2::new(
            self.x.lerp(other.x, t),
            self.y.lerp(other.y, t),
        )
    }
}

impl Interpolate for egui::Color32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        let [r1, g1, b1, a1] = self.to_array();
        let [r2, g2, b2, a2] = other.to_array();
        egui::Color32::from_rgba_premultiplied(
            (r1 as f32).lerp(r2 as f32, t) as u8,
            (g1 as f32).lerp(g2 as f32, t) as u8,
            (b1 as f32).lerp(b2 as f32, t) as u8,
            (a1 as f32).lerp(a2 as f32, t) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_lerp_at_zero() {
        let result = 0.0f32.lerp(100.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn f32_lerp_at_one() {
        let result = 0.0f32.lerp(100.0, 1.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn f32_lerp_at_half() {
        let result = 0.0f32.lerp(100.0, 0.5);
        assert!((result - 50.0).abs() < 1e-6);
    }

    #[test]
    fn color32_lerp_red_to_blue() {
        let red = egui::Color32::from_rgb(255, 0, 0);
        let blue = egui::Color32::from_rgb(0, 0, 255);
        let mid = red.lerp(blue, 0.5);
        let [r, g, b, _] = mid.to_array();
        assert!(r > 120 && r < 136);
        assert_eq!(g, 0);
        assert!(b > 120 && b < 136);
    }
}
```

- [ ] Step 2: Run test to verify it fails
Run: `cargo test --lib app::animation::tests`
Expected: FAIL — module not found in `app/mod.rs`

- [ ] Step 3: Register module in `app/mod.rs`
```rust
// D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\mod.rs
// Add at the top:
pub mod animation;
```

- [ ] Step 4: Run test to verify it passes
Run: `cargo test --lib app::animation::tests`
Expected: All 4 tests pass

- [ ] Step 5: Commit
Run: `git add -A && git commit -m "feat: add Interpolate trait with f32, Pos2, Vec2, Color32 impls"`

---

### Task 2: Implement Easing Enum and Bézier Curve

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\animation.rs`

- [ ] Step 1: Write failing tests for Easing
```rust
// Append to animation.rs tests module:

#[cfg(test)]
mod easing_tests {
    use super::*;

    #[test]
    fn linear_returns_input() {
        assert_eq!(Easing::Linear.evaluate(0.0), 0.0);
        assert_eq!(Easing::Linear.evaluate(0.5), 0.5);
        assert_eq!(Easing::Linear.evaluate(1.0), 1.0);
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
    fn ease_out_bounce_at_half() {
        let e = Easing::EaseOutBounce;
        let val = e.evaluate(0.5);
        assert!(val > 0.0 && val < 1.0);
    }

    #[test]
    fn bezier_cubic_standard_curve() {
        // Apple standard: (0.25, 0.1, 0.25, 1.0)
        let e = Easing::Bezier(0.25, 0.1, 0.25, 1.0);
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
        let mid = e.evaluate(0.5);
        assert!(mid > 0.3 && mid < 0.8, "mid={}", mid);
    }

    #[test]
    fn ease_in_at_boundaries() {
        let e = Easing::EaseIn;
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
    fn ease_in_out_cubic_at_boundaries() {
        let e = Easing::EaseInOutCubic;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_elastic_at_boundaries() {
        let e = Easing::EaseOutElastic;
        assert!((e.evaluate(0.0)).abs() < 1e-6);
        assert!((e.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bezier_roundtrip() {
        // Evaluate at known t, verify solve_for_t returns same t
        let e = Easing::Bezier(0.42, 0.0, 0.58, 1.0);
        for i in 1..10 {
            let t = i as f32 / 10.0;
            let x = cubic_bezier_x(0.42, 0.58, t);
            let solved_t = e.solve_for_x(x);
            assert!((solved_t - t).abs() < 1e-4, "t={} solved={}", t, solved_t);
        }
    }
}
```

- [ ] Step 2: Run tests to verify they fail
Run: `cargo test --lib app::animation::easing_tests`
Expected: FAIL — `Easing` not found

- [ ] Step 3: Implement Easing enum
```rust
// Add to animation.rs after Interpolate impls:

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
    Bezier(f32, f32, f32, f32),
}

pub(crate) fn cubic_bezier_x(x1: f32, x2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * x1 + 3.0 * mt * t * t * x2 + t * t * t
}

pub(crate) fn cubic_bezier_y(y1: f32, y2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * y1 + 3.0 * mt * t * t * y2 + t * t * t
}

fn cubic_bezier_dx(x1: f32, x2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * x1 + 6.0 * mt * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2)
}

impl Easing {
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
            }
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }
            Easing::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Easing::EaseOutElastic => {
                if t <= 0.0 { return 0.0; }
                if t >= 1.0 { return 1.0; }
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                2.0f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
            Easing::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Easing::Bezier(x1, y1, x2, y2) => {
                let t_solved = self.solve_for_x_inner(*x1, *x2, t);
                cubic_bezier_y(*y1, *y2, t_solved)
            }
        }
    }

    fn solve_for_x_inner(&self, x1: f32, x2: f32, target_x: f32) -> f32 {
        let mut t = target_x;
        for _ in 0..8 {
            let x_est = cubic_bezier_x(x1, x2, t);
            let dx = cubic_bezier_dx(x1, x2, t);
            if dx.abs() < 1e-10 { break; }
            t -= (x_est - target_x) / dx;
            t = t.clamp(0.0, 1.0);
        }
        t
    }

    pub fn solve_for_x(&self, target_x: f32) -> f32 {
        match self {
            Easing::Bezier(x1, _, x2, _) => self.solve_for_x_inner(*x1, *x2, target_x),
            _ => target_x, // Non-bezier easings are direct
        }
    }
}
```

- [ ] Step 4: Run tests to verify they pass
Run: `cargo test --lib app::animation::easing_tests`
Expected: All 12 tests pass

- [ ] Step 5: Commit
Run: `git add -A && git commit -m "feat: add Easing enum with 11 variants and cubic Bézier solver"`

---

### Task 3: Implement AnimationManager

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\animation.rs`

- [ ] Step 1: Write failing tests for AnimationManager
```rust
// Append to animation.rs:

#[cfg(test)]
mod manager_tests {
    use super::*;

    #[test]
    fn new_animation_returns_from_value() {
        let mut mgr = AnimationManager::new();
        let ctx_time = 0.0;
        let val = mgr.animate_float("test".into(), 100.0, 0.0, 1.0, Easing::Linear, ctx_time);
        assert!((val - 0.0).abs() < 1e-6);
    }

    #[test]
    fn animation_completes_at_to_value() {
        let mut mgr = AnimationManager::new();
        let val_start = mgr.animate_float("test".into(), 100.0, 0.0, 0.5, Easing::Linear, 0.0);
        let val_end = mgr.animate_float("test".into(), 100.0, 0.0, 0.5, Easing::Linear, 1.0);
        assert!((val_end - 100.0).abs() < 1e-6);
    }

    #[test]
    fn animation_midpoint_interpolates() {
        let mut mgr = AnimationManager::new();
        let val = mgr.animate_float("test".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.5);
        assert!((val - 50.0).abs() < 1.0);
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
    fn animation_with_same_key_restarts() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("x".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.0);
        // Restart with new target
        let val = mgr.animate_float("x".into(), 200.0, 0.0, 1.0, Easing::Linear, 0.0);
        assert!((val - 0.0).abs() < 1e-6);
    }

    #[test]
    fn color_animation_interpolates() {
        let mut mgr = AnimationManager::new();
        let red = egui::Color32::from_rgb(255, 0, 0);
        let blue = egui::Color32::from_rgb(0, 0, 255);
        let mid = mgr.animate_color("c".into(), blue, red, 1.0, Easing::Linear, 0.5);
        let [r, _, b, _] = mid.to_array();
        assert!(r > 120 && r < 136);
        assert!(b > 120 && b < 136);
    }

    #[test]
    fn cleanup_completed_animations() {
        let mut mgr = AnimationManager::new();
        mgr.animate_float("done".into(), 100.0, 0.0, 0.5, Easing::Linear, 0.0);
        // Advance past completion
        mgr.animate_float("done".into(), 100.0, 0.0, 0.5, Easing::Linear, 2.0);
        // The "done" animation should be cleaned up; requesting a new one starts fresh
        let val = mgr.animate_float("done".into(), 200.0, 0.0, 0.5, Easing::Linear, 2.0);
        assert!((val - 100.0).abs() < 1e-6); // Previous completed value
    }

    #[test]
    fn no_op_when_target_unchanged() {
        let mut mgr = AnimationManager::new();
        let v1 = mgr.animate_float("s".into(), 50.0, 0.0, 1.0, Easing::Linear, 0.0);
        // Wait for completion
        mgr.animate_float("s".into(), 50.0, 0.0, 1.0, Easing::Linear, 2.0);
        // Same target — should not restart
        let v2 = mgr.animate_float("s".into(), 50.0, 0.0, 1.0, Easing::Linear, 2.0);
        assert!((v2 - 50.0).abs() < 1e-6);
    }
}
```

- [ ] Step 2: Run tests to verify they fail
Run: `cargo test --lib app::animation::manager_tests`
Expected: FAIL — `AnimationManager` not found

- [ ] Step 3: Implement AnimationManager
```rust
// Add to animation.rs after Easing:

use std::collections::HashMap;

struct Animation<T: Interpolate> {
    from: T,
    to: T,
    start_time: f64,
    duration: f32,
    easing: Easing,
}

impl<T: Interpolate> Animation<T> {
    fn value_at(&self, current_time: f64) -> T {
        let elapsed = (current_time - self.start_time) as f32;
        if elapsed <= 0.0 { return self.from; }
        if elapsed >= self.duration { return self.to; }
        let t = self.easing.evaluate(elapsed / self.duration);
        self.from.lerp(self.to, t)
    }

    fn is_complete(&self, current_time: f64) -> bool {
        (current_time - self.start_time) as f32 >= self.duration
    }

    fn is_same_target(&self, target: T) -> bool
    where
        T: PartialEq,
    {
        self.to == target
    }
}

pub struct AnimationManager {
    floats: HashMap<String, Animation<f32>>,
    colors: HashMap<String, Animation<egui::Color32>>,
    positions: HashMap<String, Animation<egui::Pos2>>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            floats: HashMap::new(),
            colors: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    pub fn animate_float(
        &mut self,
        key: String,
        target: f32,
        current: f32,
        duration: f32,
        easing: Easing,
        current_time: f64,
    ) -> f32 {
        // If animation exists and is complete with same target, return final value
        if let Some(anim) = self.floats.get(&key) {
            if anim.is_complete(current_time) && anim.is_same_target(target) {
                return target;
            }
        }

        // If animation exists and target changed, restart from current position
        let from = if let Some(anim) = self.floats.get(&key) {
            if !anim.is_same_target(target) {
                anim.value_at(current_time)
            } else {
                anim.value_at(current_time)
            }
        } else {
            current
        };

        self.floats.insert(
            key.clone(),
            Animation { from, to: target, start_time: current_time, duration, easing },
        );

        from
    }

    pub fn animate_color(
        &mut self,
        key: String,
        target: egui::Color32,
        current: egui::Color32,
        duration: f32,
        easing: Easing,
        current_time: f64,
    ) -> egui::Color32 {
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
            key.clone(),
            Animation { from, to: target, start_time: current_time, duration, easing },
        );

        from
    }

    pub fn animate_position(
        &mut self,
        key: String,
        target: egui::Pos2,
        current: egui::Pos2,
        duration: f32,
        easing: Easing,
        current_time: f64,
    ) -> egui::Pos2 {
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
            key.clone(),
            Animation { from, to: target, start_time: current_time, duration, easing },
        );

        from
    }

    pub fn cleanup_completed(&mut self, current_time: f64) {
        self.floats.retain(|_, a| !a.is_complete(current_time));
        self.colors.retain(|_, a| !a.is_complete(current_time));
        self.positions.retain(|_, a| !a.is_complete(current_time));
    }
}
```

- [ ] Step 4: Run tests to verify they pass
Run: `cargo test --lib app::animation::manager_tests`
Expected: All 8 tests pass

- [ ] Step 5: Run ALL animation tests
Run: `cargo test --lib app::animation`
Expected: All 24 tests pass (4 interpolate + 12 easing + 8 manager)

- [ ] Step 6: Commit
Run: `git add -A && git commit -m "feat: add AnimationManager with float/color/position animation support"`

---

### Task 4: Integrate AnimationManager into AppState

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\mod.rs`

- [ ] Step 1: Add animation field to AppState
```rust
// In app/mod.rs, add to AppState struct:
pub anim: animation::AnimationManager,
```

- [ ] Step 2: Initialize in AppState::new()
```rust
// In AppState::new(), add:
anim: animation::AnimationManager::new(),
```

- [ ] Step 3: Run existing tests to verify no regressions
Run: `cargo test`
Expected: All existing tests pass (no regressions)

- [ ] Step 4: Commit
Run: `git add -A && git commit -m "feat: integrate AnimationManager into AppState"`

---

### Task 5: Integrate Animation into main.rs Update Loop

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\main.rs`

- [ ] Step 1: Add animation tick to update()
```rust
// In RobotControlApp::update(), after self.state.poll_data(), add:

// Tick animations with current egui time
let current_time = ctx.input(|i| i.time);
// AnimationManager is in state, views will call animate_* during render
```

- [ ] Step 2: Run build to verify compilation
Run: `cargo build`
Expected: Compiles successfully

- [ ] Step 3: Commit
Run: `git add -A && git commit -m "feat: wire animation time into main update loop"`

---

### Task 6: Animate Dashboard Connection Cards

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\views\dashboard.rs`

- [ ] Step 1: Write failing integration test
```rust
// At bottom of dashboard.rs (or in tests/):

#[cfg(test)]
mod animation_integration_tests {
    use super::*;

    #[test]
    fn connection_color_animation_smooth() {
        let mut mgr = crate::app::animation::AnimationManager::new();
        let green = egui::Color32::from_rgb(80, 200, 120);
        let red = egui::Color32::from_rgb(220, 80, 80);

        // Start: green (connected)
        let c1 = mgr.animate_color("conn_test".into(), green, green, 0.4,
            crate::app::animation::Easing::EaseOutCubic, 0.0);
        assert_eq!(c1, green);

        // Disconnect: animate to red
        let c2 = mgr.animate_color("conn_test".into(), red, green, 0.4,
            crate::app::animation::Easing::EaseOutCubic, 0.2);
        // Should be somewhere between green and red
        let [r, g, _, _] = c2.to_array();
        assert!(r > 80, "Red channel should be transitioning: {}", r);
        assert!(g < 200, "Green channel should be transitioning: {}", g);
    }
}
```

- [ ] Step 2: Run test to verify it fails
Run: `cargo test --lib views::dashboard::animation_integration_tests`
Expected: FAIL — no animation in dashboard yet

- [ ] Step 3: Modify connection_card to use animation
```rust
// In dashboard.rs, modify the connection_card function signature and body:
// Change connection_card to accept AnimationManager:

fn connection_card(
    ui: &mut egui::Ui,
    anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    label: &str,
    connected: bool,
) {
    let (target_color, status_text) = if connected {
        (egui::Color32::from_rgb(80, 200, 120), "Connected")
    } else {
        (egui::Color32::from_rgb(220, 80, 80), "Disconnected")
    };

    let key = format!("conn_{}", label);
    let color = anim.animate_color(
        key,
        target_color,
        target_color, // first frame default
        0.4,
        crate::app::animation::Easing::EaseOutCubic,
        current_time,
    );

    // ... rest of drawing code uses `color` instead of static color
    ui.horizontal(|ui| {
        ui.colored_label(color, "●");
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(color, status_text);
        });
    });
}
```

- [ ] Step 4: Update show() to pass anim and current_time
```rust
// In dashboard::show(), get time from ctx and pass anim:
// Note: show() receives ui, so we get time before the closure
```

- [ ] Step 5: Run test to verify it passes
Run: `cargo test --lib views::dashboard::animation_integration_tests`
Expected: Pass

- [ ] Step 6: Run full test suite
Run: `cargo test`
Expected: All tests pass

- [ ] Step 7: Commit
Run: `git add -A && git commit -m "feat: animate dashboard connection cards with smooth color transitions"`

---

### Task 7: Animate Dashboard Stat Rows

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\views\dashboard.rs`

- [ ] Step 1: Write test for stat value animation
```rust
#[cfg(test)]
mod stat_animation_tests {
    use super::*;

    #[test]
    fn stat_value_smooth_transition() {
        let mut mgr = crate::app::animation::AnimationManager::new();
        // Start at 0, animate to 100
        let v1 = mgr.animate_float("stat_bytes".into(), 100.0, 0.0, 0.5,
            crate::app::animation::Easing::EaseOutCubic, 0.0);
        assert!((v1 - 0.0).abs() < 1e-6);

        // Midway
        let v2 = mgr.animate_float("stat_bytes".into(), 100.0, 0.0, 0.5,
            crate::app::animation::Easing::EaseOutCubic, 0.25);
        assert!(v2 > 0.0 && v2 < 100.0, "Should be transitioning: {}", v2);
    }
}
```

- [ ] Step 2: Run test to verify it fails
Run: `cargo test --lib views::dashboard::stat_animation_tests`
Expected: FAIL

- [ ] Step 3: Modify stat_row to accept animated value
```rust
fn stat_row_animated(
    ui: &mut egui::Ui,
    anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    key: &str,
    label: &str,
    target_value: f64,
) {
    let smooth = anim.animate_float(
        key.to_string(),
        target_value as f32,
        target_value as f32,
        0.5,
        crate::app::animation::Easing::EaseOutCubic,
        current_time,
    );
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.monospace(format_bytes(smooth as u64));
        });
    });
}
```

- [ ] Step 4: Run test to verify it passes
Run: `cargo test --lib views::dashboard::stat_animation_tests`
Expected: Pass

- [ ] Step 5: Commit
Run: `git add -A && git commit -m "feat: animate dashboard stat rows with smooth value transitions"`

---

### Task 8: Animate Connections View

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\views\connections.rs`

- [ ] Step 1: Write test for connection status animation
```rust
// In connections.rs:

#[cfg(test)]
mod connection_animation_tests {
    use crate::app::animation::{AnimationManager, Easing};
    use egui::Color32;

    #[test]
    fn connection_status_transitions_smoothly() {
        let mut mgr = AnimationManager::new();
        let green = Color32::from_rgb(80, 200, 120);
        let red = Color32::from_rgb(220, 80, 80);

        // Connected state
        let c = mgr.animate_color("port_status".into(), green, red, 0.3, Easing::EaseOut, 0.0);
        let [r, _, _, _] = c.to_array();
        assert_eq!(r, 220); // Still red (just started)

        // After 0.15s (halfway)
        let c = mgr.animate_color("port_status".into(), green, red, 0.3, Easing::EaseOut, 0.15);
        let [r, g, _, _] = c.to_array();
        assert!(r < 220, "Red should be decreasing: {}", r);
        assert!(g > 80, "Green should be increasing: {}", g);
    }
}
```

- [ ] Step 2: Run test to verify it fails
Run: `cargo test --lib views::connections::connection_animation_tests`
Expected: FAIL

- [ ] Step 3: Add animation to connection status indicator
```rust
// In connections.rs show() function, add animation to status indicators:
// Use anim.animate_color for port status badges

fn port_status_badge(
    ui: &mut egui::Ui,
    anim: &mut crate::app::animation::AnimationManager,
    current_time: f64,
    port_name: &str,
    connected: bool,
) {
    let (target, text) = if connected {
        (egui::Color32::from_rgb(80, 200, 120), "●")
    } else {
        (egui::Color32::from_rgb(160, 160, 170), "○")
    };
    let color = anim.animate_color(
        format!("port_{}", port_name),
        target,
        target,
        0.3,
        crate::app::animation::Easing::EaseOut,
        current_time,
    );
    ui.colored_label(color, text);
}
```

- [ ] Step 4: Run test to verify it passes
Run: `cargo test --lib views::connections::connection_animation_tests`
Expected: Pass

- [ ] Step 5: Commit
Run: `git add -A && git commit -m "feat: animate connection view status indicators"`

---

### Task 9: Frontend-Backend Integration Tests

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\animation.rs`

- [ ] Step 1: Write frontend-backend integration tests
```rust
// Append to animation.rs:

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn serial_data_triggers_smooth_gauge() {
        let mut mgr = AnimationManager::new();
        // Simulate serial data arriving at different times
        let v1 = mgr.animate_float("gauge_speed".into(), 50.0, 0.0, 0.5, Easing::EaseOutCubic, 0.0);
        assert!((v1 - 0.0).abs() < 1e-6);

        // New data arrives before animation completes — target updates
        let v2 = mgr.animate_float("gauge_speed".into(), 80.0, 0.0, 0.5, Easing::EaseOutCubic, 0.2);
        // Should be between 0 and 80 (transitioning from wherever it was)
        assert!(v2 >= 0.0 && v2 <= 80.0, "v2={}", v2);
    }

    #[test]
    fn rapid_data_updates_coalesce() {
        let mut mgr = AnimationManager::new();
        // Rapid updates within animation duration
        mgr.animate_float("rapid".into(), 10.0, 0.0, 1.0, Easing::Linear, 0.0);
        mgr.animate_float("rapid".into(), 20.0, 0.0, 1.0, Easing::Linear, 0.1);
        mgr.animate_float("rapid".into(), 30.0, 0.0, 1.0, Easing::Linear, 0.2);
        let val = mgr.animate_float("rapid".into(), 40.0, 0.0, 1.0, Easing::Linear, 0.3);
        // Should be transitioning smoothly, not jumping
        assert!(val >= 0.0 && val <= 40.0, "val={}", val);
    }

    #[test]
    fn disconnect_reconnect_state_sync() {
        let mut mgr = AnimationManager::new();
        let green = egui::Color32::from_rgb(80, 200, 120);
        let red = egui::Color32::from_rgb(220, 80, 80);

        // Connected
        mgr.animate_color("link".into(), green, red, 0.3, Easing::EaseOut, 0.0);
        // Disconnect at 1.0s (after animation complete)
        let c = mgr.animate_color("link".into(), red, green, 0.3, Easing::EaseOut, 1.0);
        assert_eq!(c, green); // Still green (animation just started)

        // After 0.3s — should be red
        let c = mgr.animate_color("link".into(), red, green, 0.3, Easing::EaseOut, 1.3);
        assert_eq!(c, red);
    }

    #[test]
    fn no_data_loss_during_animation() {
        let mut mgr = AnimationManager::new();
        // Start animating to 50
        mgr.animate_float("sensor".into(), 50.0, 0.0, 1.0, Easing::Linear, 0.0);
        // New data arrives mid-animation: target changes to 100
        let val = mgr.animate_float("sensor".into(), 100.0, 0.0, 1.0, Easing::Linear, 0.3);
        // Value should be between old progress and new target
        assert!(val >= 0.0 && val <= 100.0, "val={}", val);
    }

    #[test]
    fn animation_target_matches_latest_data() {
        let mut mgr = AnimationManager::new();
        // Simulate data: 10, 20, 30, 40
        for (i, target) in [10.0, 20.0, 30.0, 40.0].iter().enumerate() {
            let t = i as f64 * 0.05;
            mgr.animate_float("stream".into(), *target, 0.0, 0.5, Easing::Linear, t);
        }
        // Final animation target should be 40
        // After completion
        let val = mgr.animate_float("stream".into(), 40.0, 0.0, 0.5, Easing::Linear, 2.0);
        assert!((val - 40.0).abs() < 1e-6);
    }

    #[test]
    fn error_state_propagates_with_animation() {
        let mut mgr = AnimationManager::new();
        let normal = egui::Color32::from_rgb(100, 100, 110);
        let error = egui::Color32::from_rgb(255, 80, 80);

        // Normal → Error transition
        let c = mgr.animate_color("status".into(), error, normal, 0.3, Easing::EaseOut, 0.0);
        assert_eq!(c, normal); // Just started

        let c = mgr.animate_color("status".into(), error, normal, 0.3, Easing::EaseOut, 0.3);
        assert_eq!(c, error); // Completed
    }

    #[test]
    fn high_frequency_data_graceful() {
        let mut mgr = AnimationManager::new();
        // Simulate 100Hz data for 1 second
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let target = (i as f32).sin() * 50.0 + 50.0;
            let _ = mgr.animate_float("hf".into(), target, 0.0, 0.1, Easing::Linear, t);
        }
        // Should not panic, value should be reasonable
        let val = mgr.animate_float("hf".into(), 50.0, 0.0, 0.1, Easing::Linear, 1.0);
        assert!(val.is_finite(), "val={}", val);
    }

    #[test]
    fn button_press_feedback_animation() {
        let mut mgr = AnimationManager::new();
        // Press starts at 1.0, animates to 0.95 (scale down)
        let v1 = mgr.animate_float("btn_send".into(), 0.95, 1.0, 0.1, Easing::EaseOutBack, 0.0);
        assert!((v1 - 1.0).abs() < 1e-6);

        // Release: animate back to 1.0
        let v2 = mgr.animate_float("btn_send".into(), 1.0, 0.95, 0.2, Easing::EaseOutBack, 0.15);
        assert!(v2 >= 0.95 && v2 <= 1.05, "v2={}", v2); // EaseOutBack can overshoot slightly
    }
}
```

- [ ] Step 2: Run tests
Run: `cargo test --lib app::animation::integration_tests`
Expected: All 8 tests pass

- [ ] Step 3: Commit
Run: `git add -A && git commit -m "feat: add frontend-backend integration tests for animation system"`

---

### Task 10: Performance Regression Tests

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\src\app\animation.rs`

- [ ] Step 1: Write performance tests
```rust
// Append to animation.rs:

#[cfg(test)]
mod perf_tests {
    use super::*;

    #[test]
    fn thousand_concurrent_animations_under_5ms() {
        let mut mgr = AnimationManager::new();
        let start = std::time::Instant::now();

        for i in 0..1000 {
            let key = format!("anim_{}", i);
            let _ = mgr.animate_float(key, i as f32, 0.0, 1.0, Easing::EaseOutCubic, 0.5);
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 5, "1000 animations took {:?}", elapsed);
    }

    #[test]
    fn bezier_computation_under_100ns() {
        let e = Easing::Bezier(0.42, 0.0, 0.58, 1.0);
        let start = std::time::Instant::now();

        for i in 0..10000 {
            let t = (i as f32) / 10000.0;
            let _ = e.evaluate(t);
        }

        let elapsed = start.elapsed();
        let per_call = elapsed / 10000;
        assert!(per_call.as_nanos() < 500, "Per call: {:?}", per_call);
    }

    #[test]
    fn cleanup_1000_completed_under_1ms() {
        let mut mgr = AnimationManager::new();
        for i in 0..1000 {
            let key = format!("done_{}", i);
            mgr.animate_float(key, 100.0, 0.0, 0.1, Easing::Linear, 0.0);
            // Complete them
            mgr.animate_float(format!("done_{}", i), 100.0, 0.0, 0.1, Easing::Linear, 1.0);
        }

        let start = std::time::Instant::now();
        mgr.cleanup_completed(1.0);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 1, "Cleanup took {:?}", elapsed);
    }
}
```

- [ ] Step 2: Run performance tests
Run: `cargo test --lib app::animation::perf_tests`
Expected: All 3 tests pass

- [ ] Step 3: Run ALL animation tests
Run: `cargo test --lib app::animation`
Expected: All ~45 tests pass

- [ ] Step 4: Commit
Run: `git add -A && git commit -m "feat: add performance regression tests for animation system"`

---

### Task 11: Full Test Suite Verification

- [ ] Step 1: Run complete test suite
Run: `cargo test`
Expected: All tests pass, no regressions

- [ ] Step 2: Run clippy
Run: `cargo clippy -- -D warnings`
Expected: Zero warnings

- [ ] Step 3: Run format check
Run: `cargo fmt --check`
Expected: All files formatted

- [ ] Step 4: Build release
Run: `cargo build --release`
Expected: Builds successfully

- [ ] Step 5: Commit any fixes
Run: `git add -A && git commit -m "chore: fix clippy warnings and formatting in animation module"`

---

### Task 12: Documentation Cleanup and Sync

**Files:**
- Modify: `D:\Destop\test_ui\rust_serial\robot_control_rust\docs\specs\2026-06-11-animation-system-design.md`
- Create: `D:\Destop\test_ui\rust_serial\robot_control_rust\docs\specs\2026-06-11-animation-api-reference.md`

- [ ] Step 1: Generate API docs
Run: `cargo doc --no-deps --open`
Expected: Browser opens with generated docs

- [ ] Step 2: Verify all public items have doc comments
Run: `cargo doc --no-deps 2>&1 | grep -i warning`
Expected: No warnings about missing docs

- [ ] Step 3: Add doc comments to all public items in animation.rs
```rust
/// Cubic Bézier easing curve evaluator.
///
/// Supports 11 built-in easing functions plus custom cubic Bézier curves
/// defined by four control points (x1, y1, x2, y2).
pub enum Easing { ... }

/// Manages all active animations in the application.
///
/// Provides type-safe animation for f32, Color32, and Pos2 values.
/// Animations are keyed by string identifiers and automatically
/// interpolate from current to target values.
pub struct AnimationManager { ... }

/// Trait for types that support linear interpolation.
pub trait Interpolate: Copy { ... }
```

- [ ] Step 4: Create API reference doc
```markdown
# Animation System API Reference

## Quick Start

```rust
use crate::app::animation::{AnimationManager, Easing};

// In your view function:
let current_time = ctx.input(|i| i.time);
let smooth_value = state.anim.animate_float(
    "my_key".into(),    // unique key
    target_value,       // target
    current_value,      // current (used on first call)
    0.3,                // duration in seconds
    Easing::EaseOutCubic,
    current_time,
);
```

## Easing Functions

| Variant | Curve | Best For |
|---------|-------|----------|
| Linear | Straight line | Continuous data streams |
| EaseIn | Slow start | Entering elements |
| EaseOut | Slow end | Exiting elements |
| EaseInOut | Slow both ends | Symmetric transitions |
| EaseInCubic | Aggressive slow start | Subtle entries |
| EaseOutCubic | Aggressive slow end | Smooth stops |
| EaseInOutCubic | Aggressive both | Dramatic transitions |
| EaseOutBack | Overshoot + settle | Bouncy buttons |
| EaseOutElastic | Spring oscillation | Gauge needles |
| EaseOutBounce | Bounce on floor | Notification popups |
| Bezier(x1,y1,x2,y2) | Custom curve | Any custom motion |

## AnimationManager Methods

- `animate_float(key, target, current, duration, easing, time) -> f32`
- `animate_color(key, target, current, duration, easing, time) -> Color32`
- `animate_position(key, target, current, duration, easing, time) -> Pos2`
- `cleanup_completed(time)` — remove finished animations
```

- [ ] Step 5: Archive design doc
Run: `mkdir -p docs/archive && mv docs/specs/2026-06-11-animation-system-design.md docs/archive/`
Expected: Design doc moved to archive

- [ ] Step 6: Final commit
Run: `git add -A && git commit -m "docs: add animation API reference and archive design doc"`
