# UI/UX 设计标准规范 v1.0

> 基于 robot_control_rust 项目现有代码分析 + 行业最佳实践制定

---

## 1. 字体设计标准

### 1.1 当前状态分析

| 字体样式 | 当前值 | 问题 |
|---------|--------|------|
| `Small` | 13.5px | 未与缩放联动 |
| `Body` | 15.5px | 基准合理 |
| `Button` | 15.0px | 与 Body 差异过小 |
| `Monospace` | 14.5px | 终端/日志场景合理 |
| `Heading` | 24.0px | 层级跳跃过大(15.5→24.0) |

**硬编码问题**：`ui_kit.rs` 中 `RichText::new(text).size(13.0)` 散布各处，未使用主题字体系统。

### 1.2 设计标准

#### 字体缩放系统

```
基准缩放因子 = ui_scale_percent / 100.0（当前范围 100%-220%）

推荐扩展：
  MIN_UI_SCALE_PERCENT = 80   （高分辨率屏幕紧凑模式）
  MAX_UI_SCALE_PERCENT = 250  （低视力用户可访问性）
  DEFAULT_UI_SCALE_PERCENT = 120（当前 150 偏高，120 更适合桌面默认）
  UI_SCALE_STEP_PERCENT = 10
```

#### 字体 Token 系统（替代硬编码）

| Token | 基准值 | 缩放规则 | 用途 |
|-------|--------|---------|------|
| `--font-caption` | 11.0px | `× scale` | 辅助说明、时间戳 |
| `--font-small` | 13.0px | `× scale` | 次要信息、标签 |
| `--font-body` | 15.0px | `× scale` | 正文默认 |
| `--font-button` | 15.0px | `× scale` | 按钮文本 |
| `--font-subheading` | 18.0px | `× scale` | 卡片标题 |
| `--font-heading` | 22.0px | `× scale` | 页面标题 |
| `--font-display` | 28.0px | `× scale` | 仪表盘大数字 |
| `--font-mono` | 14.0px | `× scale` | 终端、日志、数据 |

#### 字体自适应规则

```rust
// 推荐实现：统一字体解析函数
fn resolve_font_size(token: FontToken, scale_percent: u32) -> f32 {
    let base = token.base_px();
    let scale = scale_percent as f32 / 100.0;
    (base * scale).round()
}
```

- 所有 `RichText::new(...).size(...)` 调用 MUST 使用 token 而非字面量
- 字体大小 MUST 随 `ui_scale_percent` 同步变化
- 最小可读字号：11px（缩放后），低于此值 MUST 裁剪

#### 字体兼容性要求

| 要求 | 规范 |
|------|------|
| CJK 字体回退 | 当前 `try_load_cjk_font()` 已实现，MUST 保留 |
| 等宽字体 | MUST 使用 `FontFamily::Monospace`，用于代码/数据 |
| 行高 | `line_height = font_size × 1.4`（当前未显式设置，依赖 egui 默认） |
| 字重 | 正文 Regular(400)，标题 SemiBold(600)，当前仅用 `strong()` |

---

## 2. 颜色设计标准

### 2.1 当前状态分析

**暗色主题对比度审计**（基于 `AppTheme::dark()`）：

| 组合 | 前景 | 背景 | 对比度 | WCAG |
|------|------|------|--------|------|
| 主文本/bg_dark | `(220,220,230)` | `(22,28,38)` | ~12.5:1 | ✅ AAA |
| 次文本/bg_dark | `(200,210,220)` | `(22,28,38)` | ~10.2:1 | ✅ AAA |
| 弱文本/bg_dark | `(140,150,160)` | `(22,28,38)` | ~5.1:1 | ⚠️ AA仅大文本 |
| 状态OK/bg_dark | `(46,160,67)` | `(22,28,38)` | ~5.8:1 | ✅ AA |
| 状态错误/bg_dark | `(255,100,100)` | `(22,28,38)` | ~6.2:1 | ✅ AA |
| 强调蓝/bg_dark | `(88,166,255)` | `(22,28,38)` | ~7.1:1 | ✅ AAA |
| 连接色/bg_dark | `(46,160,67)` | `(22,28,38)` | ~5.8:1 | ✅ AA |

**亮色主题对比度审计**（基于 `AppTheme::light()`）：

| 组合 | 前景 | 背景 | 对比度 | WCAG |
|------|------|------|--------|------|
| 主文本/bg_dark | `(30,30,40)` | `(240,240,245)` | ~14.8:1 | ✅ AAA |
| 弱文本/bg_dark | `(120,120,130)` | `(240,240,245)` | ~4.2:1 | ⚠️ 仅AA大文本 |
| 状态OK/bg_dark | `(40,160,60)` | `(240,240,245)` | ~4.8:1 | ⚠️ 边缘 |

### 2.2 设计标准

#### 对比度要求

| 场景 | 最低对比度 | 标准 |
|------|-----------|------|
| 正文本（<18px） | **4.5:1** | WCAG AA |
| 大文本（≥18px 或 ≥14px bold） | **3:1** | WCAG AA |
| 关键操作文本 | **7:1** | WCAG AAA（推荐） |
| 图标/图形 | **3:1** | WCAG AA |
| 弱化/禁用文本 | **3:1** | 最低可接受 |
| 状态指示（仅颜色） | MUST 有文字/图标辅助 | 色盲可访问 |

#### 颜色 Token 系统

```rust
// 推荐：AppTheme 扩展为语义化 Token
pub struct ColorTokens {
    // 表面层
    pub surface_base: Color32,      // 最底层背景
    pub surface_card: Color32,      // 卡片背景
    pub surface_input: Color32,     // 输入框背景
    pub surface_elevated: Color32,  // 浮层/弹窗

    // 文本层
    pub text_primary: Color32,      // 主文本 ≥7:1
    pub text_secondary: Color32,    // 次文本 ≥4.5:1
    pub text_tertiary: Color32,     // 弱文本 ≥3:1
    pub text_on_accent: Color32,    // 强调色上的文本

    // 交互层
    pub accent: Color32,            // 主强调色
    pub accent_hover: Color32,      // 悬停态（亮度+10%）
    pub accent_active: Color32,     // 按下态（亮度-10%）
    pub accent_disabled: Color32,   // 禁用态（opacity 40%）

    // 状态层
    pub status_success: Color32,
    pub status_warning: Color32,
    pub status_error: Color32,
    pub status_info: Color32,

    // 边框层
    pub border_default: Color32,
    pub border_focus: Color32,      // = accent
    pub border_error: Color32,      // = status_error
}
```

#### 主题切换规则

| 规则 | 实现要求 |
|------|---------|
| 切换时机 | 用户手动切换 OR 跟随系统（推荐新增选项） |
| 过渡动画 | 颜色插值 200ms，`Easing::EaseInOutCubic` |
| 切换范围 | ALL 颜色 token MUST 同步切换，禁止残留 |
| 状态保持 | 切换后焦点位置、滚动位置、展开状态 MUST 保持 |
| 持久化 | 暗/亮偏好 MUST 存入 `UserPreferences`（当前已实现） |

#### 颜色一致性规则

- 同一语义（如"连接成功"）在整个应用中 MUST 使用完全相同的 RGB 值
- 禁止在视图代码中出现 `Color32::from_rgb(...)` 字面量——MUST 通过 `AppTheme`/`ColorTokens` 引用
- 当前违规：`dashboard.rs` 中 `status_color()` 函数直接构造颜色，应迁移到主题

---

## 3. 布局设计标准

### 3.1 当前状态分析

```rust
// apply_page_style 和 apply_theme 中的间距设置：
spacing.item_spacing = egui::vec2(14.0, 12.0);   // 不一致：主题设 12.0×10.0
spacing.button_padding = egui::vec2(12.0, 8.0);   // 不一致：主题设 14.0×8.0
spacing.interact_size.y = 34.0;
spacing.text_edit_width = 260.0;                   // 不一致：主题设 260.0（一致）
spacing.combo_width = 240.0;                       // 不一致：主题设 260.0
spacing.slider_width = 300.0;
```

**问题**：`apply_page_style`（ui_kit.rs:284）和 `apply_theme`（main.rs）的间距值冲突。

### 3.2 设计标准

#### 响应式断点

```rust
// 推荐引入的断点系统
pub enum Breakpoint {
    Compact,    // width < 900px   — 移动端/小窗口
    Medium,     // 900-1440px      — 标准桌面
    Wide,       // > 1440px        — 大屏/多面板
}

fn detect_breakpoint(ctx: &egui::Context) -> Breakpoint {
    let width = ctx.screen_rect().width();
    if width < 900.0 { Breakpoint::Compact }
    else if width < 1440.0 { Breakpoint::Medium }
    else { Breakpoint::Wide }
}
```

| 断点 | 布局策略 |
|------|---------|
| Compact | 单列布局，侧边栏折叠为底部 Tab 栏 |
| Medium | 双列布局，侧边栏常驻（当前默认） |
| Wide | 三列布局，右侧面板可展开 |

#### 间距系统（8px 网格）

所有间距 MUST 是 8 的倍数，确保像素对齐：

| Token | 值 | 用途 |
|-------|-----|------|
| `--space-xxs` | 4px | 紧凑内联间距 |
| `--space-xs` | 8px | 元素内间距 |
| `--space-sm` | 12px | 紧凑元素间距 |
| `--space-md` | 16px | 标准间距（卡片内边距、窗口边距） |
| `--space-lg` | 24px | 区块间距 |
| `--space-xl` | 32px | 页面区域分隔 |
| `--space-xxl` | 48px | 大节间距 |

```rust
// 统一间距配置（消除 apply_page_style / apply_theme 冲突）
pub const SPACING: SpacingTokens = SpacingTokens {
    item: Vec2::new(16.0, 12.0),       // 统一
    button_padding: Vec2::new(16.0, 8.0),
    interact_h: 36.0,                   // 比当前 34.0 更易点击
    text_edit_w: 280.0,
    combo_w: 260.0,
    slider_w: 300.0,
    window_margin: 16.0,
    card_padding: 16.0,
    section_gap: 24.0,
};
```

#### 视觉层次规则

| 层级 | 实现方式 | 规则 |
|------|---------|------|
| L0 — 页面背景 | `surface_base` 纯色 | 最暗/最亮 |
| L1 — 卡片/面板 | `surface_card` + 圆角(8px) + 微阴影 | 与 L0 对比度 ≤ 5% |
| L2 — 输入/按钮 | `surface_input` + 边框 | 交互态边框变色 |
| L3 — 浮层/弹窗 | `surface_elevated` + 阴影 | z-order 最高 |

- 卡片圆角统一 8px（当前 `corner_radius(8.0)` 已一致 ✅）
- 内边距统一 16px（当前 `Margin::symmetric(16, 10)` 基本合理）
- 区块标题 `section_title` 使用 `--font-subheading` + 强调色左边框

---

## 4. 交互设计标准

### 4.1 当前状态分析

```rust
// animation.rs 中的动画实现：
// - 支持 f32, Color32, Pos2 插值 ✅
// - 支持 10 种缓动函数 + 自定义贝塞尔 ✅
// - 但 duration 在调用侧硬编码：0.3s（status_badge）、0.5s（animated_value_text）
// - toast 淡出 0.5s 硬编码
```

### 4.2 设计标准

#### 动画时长 Token

| Token | 值 | 用途 |
|-------|-----|------|
| `--duration-instant` | 100ms | 微交互（hover 反馈、toggle） |
| `--duration-fast` | 150ms | 按钮按下、颜色闪烁 |
| `--duration-normal` | 250ms | 展开/折叠、面板切换 |
| `--duration-smooth` | 300ms | 页面过渡、toast 出现 |
| `--duration-slow` | 500ms | 复杂布局变化、数据加载过渡 |

```rust
// 推荐：动画时长枚举替代硬编码
pub enum Duration {
    Instant,  // 100ms
    Fast,     // 150ms
    Normal,   // 250ms
    Smooth,   // 300ms
    Slow,     // 500ms
}

impl Duration {
    pub fn secs(self) -> f64 {
        match self {
            Self::Instant => 0.10,
            Self::Fast    => 0.15,
            Self::Normal  => 0.25,
            Self::Smooth  => 0.30,
            Self::Slow    => 0.50,
        }
    }
}
```

#### 缓动函数规范

| 场景 | 推荐缓动 | 当前实现 |
|------|---------|---------|
| 进入（元素出现） | `EaseOutCubic` | ✅ 已使用 |
| 退出（元素消失） | `EaseInCubic` | ⚠️ 未统一 |
| 状态切换 | `EaseInOutCubic` | ⚠️ 未统一 |
| 弹性反馈 | `EaseOutBack` | 可选 |
| 进度条 | `Linear` | 当前未用于进度 |

#### 反馈机制要求

| 交互 | 反馈方式 | 时长 |
|------|---------|------|
| 按钮 hover | 背景色变亮 10% | instant |
| 按钮 press | 背景色变暗 10% + 微缩放(0.97) | instant |
| 按钮 disabled | opacity 40% + 禁止光标 | — |
| 输入框 focus | 边框变为 accent 色 + glow | fast |
| 输入框 error | 边框变为 error 色 + shake(2px, 200ms) | fast |
| 状态切换成功 | 绿色 pulse 2 次 | 300ms×2 |
| 状态切换失败 | 红色 pulse 3 次 | 300ms×3 |
| Toast 通知 | slide-in + auto-dismiss 3s + fade-out 0.5s | smooth |
| 数据加载 | spinner + 文字 pulse | 持续 |
| 连接断开 | 灰色 dot + 文字变化 | normal |

#### 状态变化规则

```
状态转换 MUST 满足：
1. 可见性：用户 MUST 能看到从状态 A → 状态 B 的过渡
2. 可逆性：所有用户操作 MUST 可撤销（连接断开→重连，缩放→恢复）
3. 即时性：操作反馈 MUST 在 100ms 内开始
4. 持久性：状态变化 MUST 在下一帧渲染前生效
```

---

## 5. 可访问性标准

### 5.1 当前状态分析

- 键盘快捷键：`show_shortcuts` 功能存在但未详查覆盖范围
- 屏幕阅读器：egui 原生支持有限，当前无 ARIA 标注
- 动态类型：通过 `ui_scale_percent` 实现（100-220%），但最小值偏高

### 5.2 设计标准

#### 键盘导航要求

| 要求 | 规范 |
|------|------|
| Tab 顺序 | MUST 遵循视觉从左到右、从上到下的顺序 |
| 焦点可见 | 所有可交互元素 MUST 有可见焦点环（accent 色 2px outline） |
| Enter/Space | MUST 激活按钮和链接 |
| Escape | MUST 关闭弹窗/浮层，取消当前操作 |
| Arrow keys | MUST 在 Tab 页之间导航、列表项之间移动 |
| Ctrl+Tab | MUST 切换主标签页 |
| 全局快捷键 | MUST 提供快捷键面板（当前 `show_shortcuts` 已有基础） |

```rust
// 推荐：焦点环样式
fn focus_ring_style() -> Stroke {
    Stroke::new(2.0, theme.accent)  // 2px accent 色实线
}
```

#### 屏幕阅读器支持

egui 的可访问性支持有限，以下为最低要求：

| 要求 | 实现方式 |
|------|---------|
| 语义标签 | 所有 `ui.label()` MUST 使用有意义的文本（避免纯图标无文字） |
| 状态播报 | 状态变化 MUST 伴随文本更新（如 "已连接" → "已断开"） |
| 图标替代 | 所有图标 MUST 有文字标签或 tooltip（当前 `IconKind` 未关联标签） |
| 表单标注 | 输入框 MUST 有可见 label（非仅 placeholder） |

#### 动态类型支持

```rust
// 推荐的缩放范围扩展
pub const MIN_UI_SCALE_PERCENT: u32 = 80;   // 当前 100，扩展支持高分屏紧凑模式
pub const MAX_UI_SCALE_PERCENT: u32 = 250;  // 当前 220，扩展支持低视力用户
pub const DEFAULT_UI_SCALE_PERCENT: u32 = 120; // 当前 150，降低默认值
```

| 规则 | 要求 |
|------|------|
| 缩放范围 | 80%-250%，步进 10% |
| 最小触控目标 | 缩放后 ≥ 44×44px（WCAG 2.5.5） |
| 文本重排 | 缩放后文本 MUST 不截断，允许换行 |
| 布局不溢出 | 缩放至 200% 时 MUST 无水平滚动（正文区域） |
| 缩放持久化 | 缩放偏好 MUST 存入 `UserPreferences`（当前已实现 ✅） |

---

## 6. 实施优先级

| 优先级 | 改进项 | 工作量 | 影响 |
|--------|--------|--------|------|
| **P0** | 消除 `apply_page_style`/`apply_theme` 间距冲突 | 0.5h | 一致性 |
| **P0** | 提取硬编码字体大小为 Token | 2h | 可维护性 |
| **P0** | 提取硬编码颜色为 Theme 引用 | 3h | 主题一致性 |
| **P1** | 扩展缩放范围 80-250% | 0.5h | 可访问性 |
| **P1** | 引入动画时长 Token | 1h | 交互一致性 |
| **P1** | 添加焦点环样式 | 1h | 键盘可访问性 |
| **P2** | 响应式断点系统 | 4h | 自适应布局 |
| **P2** | 完整的 ColorTokens 结构 | 2h | 主题系统 |
| **P3** | 按钮交互反馈（hover/press 色变） | 2h | 交互品质 |
| **P3** | 状态过渡动画统一化 | 3h | 视觉流畅度 |

---

## 7. 当前代码违规清单

| 文件 | 行 | 违规 | 修复方式 |
|------|-----|------|---------|
| `ui_kit.rs` | 117 | `RichText::new(text).size(13.0)` 硬编码 | 使用 `FontToken::Small` |
| `ui_kit.rs` | 185-186 | `size(14.0)`, `size(13.0)` 硬编码 | 使用 Token |
| `ui_kit.rs` | 206 | `size(13.0)` 硬编码 | 使用 Token |
| `ui_kit.rs` | 215-224 | `size(32.0)`, `size(15.0)`, `size(12.0)` 硬编码 | 使用 Token |
| `ui_kit.rs` | 250 | `size(14.0)` 硬编码 | 使用 Token |
| `ui_kit.rs` | 277-279 | `size(13.0)` 硬编码 | 使用 Token |
| `ui_kit.rs` | 286 | `vec2(14.0, 12.0)` 与主题冲突 | 使用统一 SpacingToken |
| `ui_kit.rs` | 287 | `vec2(12.0, 8.0)` 与主题冲突 | 使用统一 SpacingToken |
| `ui_kit.rs` | 289 | `260.0` 与主题 `240.0` 冲突 | 使用统一 SpacingToken |
| `main.rs` | ~100 | `MIN=100, MAX=220` 范围偏窄 | 扩展至 80-250 |
| `dashboard.rs` | `status_color()` | 直接构造颜色而非使用主题 | 迁移到 AppTheme |

---

*本规范基于 robot_control_rust v0.2.1 代码库分析生成，建议随版本迭代持续更新。*
