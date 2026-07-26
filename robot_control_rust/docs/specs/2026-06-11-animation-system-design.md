# Animation System Design

**Goal:** Add a comprehensive non-linear animation and Bézier curve system to the Robot Control Rust GUI
**Architecture:** egui-native + lightweight `AnimationManager` module (single file)
**Tech Stack:** Rust, egui, eframe

## 1. Problem Statement

The current GUI has zero animations. All state transitions are instantaneous:
- Connection status colors jump between green/red
- Tab switches have no transition
- Dashboard statistics update without visual feedback
- Gauge/indicator values snap to new positions
- Error/status messages appear/disappear without animation
- Button interactions have no hover/press feedback beyond egui defaults

This creates a jarring, unpolished user experience, especially when real-time data streams (serial/network) drive rapid UI updates.

## 2. Constraints

- **Real-time data streams must not be blocked** by animation computation
- **egui is immediate-mode** — no retained scene graph; every frame redraws from scratch
- **Animation must decouple from data rate** — data arrives at arbitrary frequencies, animation runs at display refresh rate
- **Single file module** — ~300 lines, no sub-module directory

## 3. Architecture

```
robot_control_rust/src/
├── app/
│   ├── mod.rs              ← (existing) AppState
│   └── animation.rs        ← NEW: AnimationManager + Easing + BezierCurve + Interpolate trait
├── views/
│   ├── ui_kit.rs           ← MODIFY: draw_* functions accept animation params
│   ├── dashboard.rs        ← MODIFY: connection cards, stat rows with animations
│   ├── connections.rs      ← MODIFY: connect/disconnect transitions
│   ├── pid_control.rs      ← MODIFY: slider value smoothing
│   └── ...                 ← MODIFY: other views as needed
└── main.rs                 ← MODIFY: integrate AnimationManager into RobotControlApp
```

### 3.1 Data Flow

```
Serial/Network Thread ──Arc<Mutex<DataBus>>──▶ AppState
                                                    │
                                                    ▼
                                              AnimationManager
                                              (keyed animations)
                                                    │
                                                    ▼ (per frame)
                                              egui View Layer
                                              (lerped values)
```

- Data threads write to `DataBus` via `Arc<Mutex<>>` (existing pattern)
- `AnimationManager` lives in `AppState`, updated each frame with `ctx.input(|i| i.time)`
- Views request animated values via `anim.animate_float("key", target, duration, easing, ctx)`
- AnimationManager returns interpolated current value; egui renders it

### 3.2 Core Types

```rust
// animation.rs

pub trait Interpolate: Copy {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Interpolate for f32 { ... }
impl Interpolate for egui::Color32 { ... }  // RGBA channel-wise lerp
impl Interpolate for egui::Pos2 { ... }     // 2D point lerp
impl Interpolate for egui::Vec2 { ... }     // 2D vector lerp

pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseOutBack,        // overshoot bounce
    EaseOutElastic,     // elastic spring
    EaseOutBounce,      // bounce on floor
    Bezier(f32, f32, f32, f32),  // custom cubic Bézier control points
}

pub struct Animation<T: Interpolate> {
    from: T,
    to: T,
    start_time: f64,
    duration: f32,
    easing: Easing,
}

pub struct AnimationManager {
    floats: HashMap<String, Animation<f32>>,
    colors: HashMap<String, Animation<egui::Color32>>,
    positions: HashMap<String, Animation<egui::Pos2>>,
    vectors: HashMap<String, Animation<egui::Vec2>>,
}
```

## 4. Bézier Curve Implementation

```rust
impl Easing {
    pub fn evaluate(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }
            Easing::Bezier(x1, y1, x2, y2) => {
                // Newton-Raphson solve for t given x, then evaluate y
                let mut t_est = t;
                for _ in 0..8 {
                    let x_est = cubic_bezier_x(*x1, *x2, t_est);
                    let dx = cubic_bezier_dx(*x1, *x2, t_est);
                    if dx.abs() < 1e-10 { break; }
                    t_est -= (x_est - t) / dx;
                    t_est = t_est.clamp(0.0, 1.0);
                }
                cubic_bezier_y(*y1, *y2, t_est)
            }
            // ... other variants
        }
    }
}
```

## 5. Animation Scenarios

| Scenario | Type | Easing | Duration | Key |
|----------|------|--------|----------|-----|
| Button hover | Color | EaseOut | 0.15s | `btn_hover_{id}` |
| Button press | Scale | EaseOutBack | 0.2s | `btn_press_{id}` |
| Tab switch | Pos + Alpha | EaseInOutCubic | 0.3s | `tab_transition` |
| Panel expand/collapse | Height | EaseOutCubic | 0.25s | `panel_{id}` |
| Modal popup | Scale + Alpha | EaseOutBack | 0.35s | `modal_{id}` |
| Connection status | Color | EaseOut | 0.4s | `conn_status_{name}` |
| Gauge pointer | Angle | EaseOutElastic | 0.8s | `gauge_{id}` |
| Data value update | Float | EaseOutCubic | 0.5s | `data_{channel}` |
| Real-time waveform | Float | Linear | continuous | `wave_{channel}` |
| Error toast | Pos | EaseInOut | 0.3+2+0.3s | `toast_error` |
| Loading pulse | Alpha | Sine | looping | `loading_pulse` |

## 6. Testing Strategy (5-Layer, ~45 tests)

### Layer 1: Easing Function Unit Tests (~15)
- Boundary: t=0 → start, t=1 → end, t=0.5 → expected
- Monotonicity for EaseIn/Out variants
- Bézier accuracy to 1e-6
- EaseOutBack overshoots > 1.0
- EaseOutBounce at boundaries
- Custom Bézier roundtrip (evaluate → solve → evaluate)

### Layer 2: AnimationManager Logic Tests (~12)
- New animation returns `from` value
- Animation completes at `to` value
- Midpoint interpolation correct
- Key conflict handling (overwrite vs reject)
- Multiple independent animations
- Time advancement progresses animations
- Completed animations auto-cleanup
- Float/Color/Position type coverage

### Layer 3: End-to-End Scenario Tests (~5)
- Dashboard connection color transition
- Tab switch cleans old animations
- Rapid state changes coalesce correctly
- Animation survives repaint cycle
- No animation leaks after navigation

### Layer 4: Performance Regression Tests (~3)
- 1000 concurrent animations < 1ms
- Bézier computation < 100ns
- Zero allocation during tick

### Layer 5: Frontend-Backend Integration Tests (~10)
- Serial data update triggers smooth gauge animation
- Network state change drives connection card animation
- DataBus publish → UI subscribes → animation triggers
- Button click → animation feedback → serial command sent
- Slider change → smooth animation → backend parameter updated
- Disconnect/reconnect state fully synchronized
- High-frequency data (>100Hz) degrades gracefully
- Error state propagates to UI with animation
- Animation target matches latest data (not stale)
- No data loss during animation playback

## 7. Quality Assurance

### Pre-commit hooks
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test` (all 45+ animation tests must pass)
- New public APIs require doc comments

### Pre-push hooks
- `cargo test --release`
- Performance regression tests pass

### CI Pipeline
- Full test suite
- Code coverage ≥ 80% for `animation.rs`
- Clippy zero warnings
- Doc completeness check

### Periodic Review
| Frequency | Check | Method |
|-----------|-------|--------|
| Per commit | Format + clippy + tests | git hook |
| Weekly | Coverage report | CI |
| Monthly | Performance benchmarks | benchmark suite |
| Quarterly | Doc-implementation sync | Manual audit |

## 8. Documentation Lifecycle

| Document | Lifecycle | Cleanup |
|----------|-----------|---------|
| This design doc | Archive after implementation | Move to `docs/archive/` |
| API docs | Auto-generated | `cargo doc` |
| CHANGELOG | Permanent | Compress to version summary |
| Test reports | CI-generated | Keep last 30 days |
| Temp notes | Delete after implementation | git hook reminder |

## 9. Implementation Order

1. `animation.rs` — Core module (Easing, Bézier, AnimationManager, Interpolate)
2. Unit tests — Layer 1 + Layer 2
3. `ui_kit.rs` — Integrate animation into draw_* functions
4. `dashboard.rs` — First animated view (connection cards, stats)
5. `connections.rs` — Connect/disconnect transitions
6. `main.rs` — Wire AnimationManager into RobotControlApp
7. Integration tests — Layer 3 + Layer 5
8. Performance tests — Layer 4
9. Documentation cleanup — compress, archive, sync
