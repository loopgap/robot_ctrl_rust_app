"""Modern dark theme for PySide6 GUI.

Design system blending Material Design 3 (M3) tonal palette with
Apple HIG semantic color principles:

- M3: tonal surfaces, 4px grid, rounded containers, focus rings
- Apple: semantic colors, subtle borders (not shadows) in dark mode,
  generous whitespace, clear typography hierarchy
- Glassmorphism: translucent card surfaces with subtle edge highlights

Color provenance:
- Surface tones: M3 Neutral palette (tonal elevation)
- Accent blue: Apple system blue (#0A84FF) for interactive elements
- Status colors: Apple system green/red/yellow
- Text hierarchy: Apple label/secondary/tertiary opacity model
"""

from __future__ import annotations

# ── Design Tokens (M3 + Apple HIG hybrid) ─────────────────
# Surfaces follow M3 tonal elevation model:
#   base → surface-1 → surface-2 → surface-3 (higher = more luminance)
#
# Text follows Apple's label hierarchy:
#   primary (100%) → secondary (55%) → tertiary (25%) → quaternary (10%)

COLORS = {
    # ── Surface / Background (M3 Neutral 6/10/12/17) ──────
    "bg_base": "#0F1117",       # Deepest background (M3 Neutral 6)
    "bg_surface": "#161A22",    # Card / panel surface (M3 Neutral 10)
    "bg_elevated": "#1C2028",   # Elevated surface (M3 Neutral 12)
    "bg_overlay": "#242830",    # Overlay / hover state (M3 Neutral 17)
    "bg_input": "#1A1E26",      # Input field background

    # ── Borders (Apple: subtle white-alpha, not opaque) ────
    "border_subtle": "rgba(255, 255, 255, 0.06)",
    "border_default": "rgba(255, 255, 255, 0.10)",
    "border_emphasis": "rgba(255, 255, 255, 0.15)",
    "border_focus": "rgba(10, 132, 255, 0.50)",  # Apple blue focus ring

    # ── Text (Apple label hierarchy) ───────────────────────
    "text_primary": "#F5F5F7",       # Label (100% opacity)
    "text_secondary": "rgba(245, 245, 247, 0.55)",  # Secondary label
    "text_tertiary": "rgba(245, 245, 247, 0.25)",   # Tertiary label
    "text_disabled": "rgba(245, 245, 247, 0.15)",   # Quaternary label

    # ── Accent (Apple system blue for interactive) ─────────
    "accent": "#0A84FF",         # Apple blue — interactive elements
    "accent_hover": "#409CFF",   # Lighter blue on hover
    "accent_pressed": "#0060CC", # Darker blue on press
    "accent_surface": "rgba(10, 132, 255, 0.12)",  # Blue tinted surface
    "accent_surface_hover": "rgba(10, 132, 255, 0.18)",

    # ── Status (Apple system colors) ───────────────────────
    "green": "#30D158",     # Apple green — success
    "green_surface": "rgba(48, 209, 88, 0.12)",
    "red": "#FF453A",       # Apple red — error
    "red_surface": "rgba(255, 69, 58, 0.12)",
    "yellow": "#FFD60A",    # Apple yellow — warning
    "yellow_surface": "rgba(255, 214, 10, 0.12)",
    "orange": "#FF9F0A",    # Apple orange — caution
    "teal": "#64D2FF",      # Apple teal — info
    "purple": "#BF5AF2",    # Apple purple — special
    "pink": "#FF375F",      # Apple pink — accent

    # ── Chart palette (high contrast on dark bg) ───────────
    "chart_speed": "#64D2FF",      # Teal — speed curve
    "chart_ref": "#30D158",        # Green — reference line
    "chart_torque": "#FF9F0A",     # Orange — torque curve
    "chart_grid": "rgba(255, 255, 255, 0.06)",  # Subtle grid
    "chart_axis_label": "rgba(245, 245, 247, 0.45)",

    # ── Scrollbar ──────────────────────────────────────────
    "scroll_track": "rgba(255, 255, 255, 0.03)",
    "scroll_handle": "rgba(255, 255, 255, 0.12)",
    "scroll_handle_hover": "rgba(255, 255, 255, 0.20)",

    # ── Glassmorphism (translucent surfaces) ───────────────
    "glass_bg": "rgba(22, 26, 34, 0.70)",
    "glass_border": "rgba(255, 255, 255, 0.08)",
    "glass_highlight": "rgba(255, 255, 255, 0.04)",
}

# ── Spacing (M3 4px grid) ─────────────────────────────────
SPACING = {
    "xs": "4px",
    "sm": "8px",
    "md": "12px",
    "lg": "16px",
    "xl": "24px",
    "xxl": "32px",
}

# ── Radii (M3 shape scale) ────────────────────────────────
RADIUS = {
    "xs": "4px",    # Small elements (chips, badges)
    "sm": "6px",    # Inputs, buttons
    "md": "8px",    # Cards, panels
    "lg": "12px",   # Large cards, dialogs
    "xl": "16px",   # Hero cards
    "full": "9999px",  # Pills
}

# ── Typography (Apple-inspired hierarchy) ─────────────────
TYPOGRAPHY = {
    "font_family": '"Inter", "Segoe UI", "Microsoft YaHei UI", -apple-system, sans-serif',
    "font_mono": '"JetBrains Mono", "Cascadia Code", "Consolas", monospace',
    "size_xs": "11px",
    "size_sm": "12px",
    "size_body": "14px",
    "size_md": "15px",
    "size_lg": "16px",
    "size_xl": "20px",
    "size_xxl": "24px",
    "weight_normal": "400",
    "weight_medium": "500",
    "weight_semibold": "600",
    "weight_bold": "700",
}

# ── Animation Tokens (for transitions and state changes) ──
ANIMATION = {
    "duration_instant": 100,     # Immediate feedback (ms)
    "duration_fast": 200,        # Quick transitions
    "duration_default": 300,     # Standard UI transitions
    "duration_smooth": 400,      # Smoother state changes
    "duration_slow": 600,        # Emphasis transitions
    "duration_glacial": 1000,    # Guided tour emphasis
    "easing_default": "cubic-bezier(0.4, 0.0, 0.2, 1)",  # OutCubic
    "easing_enter": "cubic-bezier(0.0, 0.0, 0.2, 1)",    # Ease-out
    "easing_exit": "cubic-bezier(0.4, 0.0, 1.0, 1.0)",   # Ease-in
}


def get_stylesheet() -> str:
    """Return the complete modern dark QSS stylesheet."""
    c = COLORS
    s = SPACING
    r = RADIUS
    t = TYPOGRAPHY

    return f"""
/* ════════════════════════════════════════════════════════════
   SIM_PLATFORM MODERN DARK THEME
   Design: M3 tonal surfaces + Apple semantic colors
   ════════════════════════════════════════════════════════════ */

/* ── Global Reset ────────────────────────────────────────── */
* {{
    font-family: {t['font_family']};
    font-size: {t['size_body']};
}}

QWidget {{
    background-color: {c['bg_base']};
    color: {c['text_primary']};
    selection-background-color: {c['accent']};
    selection-color: #FFFFFF;
}}

QMainWindow {{
    background-color: {c['bg_base']};
}}

/* ── Menu Bar ────────────────────────────────────────────── */
QMenuBar {{
    background-color: {c['bg_surface']};
    color: {c['text_primary']};
    border-bottom: 1px solid {c['border_subtle']};
    padding: {s['xs']} 0;
    font-size: {t['size_body']};
}}
QMenuBar::item {{
    background: transparent;
    padding: {s['sm']} {s['lg']};
    border-radius: {r['xs']};
}}
QMenuBar::item:selected {{
    background-color: {c['bg_overlay']};
    color: {c['text_primary']};
}}
QMenu {{
    background-color: {c['bg_elevated']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['md']};
    padding: {s['xs']} 0;
}}
QMenu::item {{
    padding: {s['sm']} {s['xl']};
    border-radius: {r['xs']};
    margin: 0 {s['xs']};
}}
QMenu::item:selected {{
    background-color: {c['accent_surface']};
    color: {c['text_primary']};
}}
QMenu::separator {{
    height: 1px;
    background: {c['border_subtle']};
    margin: {s['xs']} {s['lg']};
}}

/* ── Tool Bar ────────────────────────────────────────────── */
QToolBar {{
    background-color: {c['bg_surface']};
    border-bottom: 1px solid {c['border_subtle']};
    padding: {s['xs']} {s['sm']};
    spacing: {s['xs']};
}}
QToolBar QToolButton {{
    background-color: transparent;
    color: {c['text_secondary']};
    border: 1px solid transparent;
    border-radius: {r['sm']};
    padding: {s['sm']} {s['md']};
    font-weight: {t['weight_medium']};
    font-size: {t['size_sm']};
}}
QToolBar QToolButton:hover {{
    background-color: {c['bg_overlay']};
    color: {c['text_primary']};
    border-color: {c['border_subtle']};
}}
QToolBar QToolButton:pressed {{
    background-color: {c['bg_elevated']};
}}
QToolBar QToolButton:disabled {{
    color: {c['text_disabled']};
}}
QToolBar::separator {{
    width: 1px;
    background: {c['border_subtle']};
    margin: {s['xs']} {s['sm']};
}}

/* ── Status Bar ──────────────────────────────────────────── */
QStatusBar {{
    background-color: {c['bg_surface']};
    color: {c['text_secondary']};
    border-top: 1px solid {c['border_subtle']};
    font-size: {t['size_sm']};
    padding: {s['xs']} {s['md']};
}}
QStatusBar QLabel {{
    color: {c['text_secondary']};
    font-size: {t['size_sm']};
}}

/* ── Dock Widget ─────────────────────────────────────────── */
QDockWidget {{
    titlebar-close-icon: none;
    titlebar-normal-icon: none;
    font-weight: {t['weight_semibold']};
    color: {c['text_primary']};
    font-size: {t['size_md']};
}}
QDockWidget::title {{
    background-color: {c['bg_surface']};
    padding: {s['md']} {s['lg']};
    border-bottom: 1px solid {c['border_subtle']};
    text-align: left;
    letter-spacing: 0.5px;
}}
QDockWidget QWidget {{
    background-color: transparent;
}}

/* ── Tab Widget (Apple segmented control style) ──────────── */
QTabWidget::pane {{
    border: 1px solid {c['border_subtle']};
    background-color: {c['bg_base']};
    border-radius: {r['md']};
    margin-top: -1px;
}}
QTabBar {{
    background: transparent;
}}
QTabBar::tab {{
    background-color: transparent;
    color: {c['text_secondary']};
    padding: {s['md']} {s['xl']};
    border: none;
    border-bottom: 2px solid transparent;
    font-weight: {t['weight_medium']};
    font-size: {t['size_body']};
    margin-right: {s['xs']};
}}
QTabBar::tab:selected {{
    color: {c['accent']};
    border-bottom: 2px solid {c['accent']};
}}
QTabBar::tab:hover {{
    color: {c['text_primary']};
    background-color: {c['bg_overlay']};
    border-radius: {r['xs']} {r['xs']} 0 0;
}}

/* ── Group Box (M3 outlined card) ────────────────────────── */
QGroupBox {{
    background-color: {c['bg_surface']};
    border: 1px solid {c['border_default']};
    border-radius: {r['md']};
    margin-top: {s['lg']};
    padding-top: {s['xl']};
    font-weight: {t['weight_semibold']};
    font-size: {t['size_sm']};
    color: {c['text_secondary']};
    letter-spacing: 0.3px;
    text-transform: uppercase;
}}
QGroupBox::title {{
    subcontrol-origin: margin;
    subcontrol-position: top left;
    padding: 0 {s['sm']};
    color: {c['text_secondary']};
}}
QGroupBox QWidget {{
    background-color: transparent;
}}

/* ── Labels ──────────────────────────────────────────────── */
QLabel {{
    color: {c['text_primary']};
    background: transparent;
}}
QLabel[class="stat-value"] {{
    font-size: {t['size_xl']};
    font-weight: {t['weight_bold']};
    color: {c['text_primary']};
    letter-spacing: -0.5px;
}}
QLabel[class="stat-label"] {{
    color: {c['text_secondary']};
    font-size: {t['size_xs']};
    font-weight: {t['weight_medium']};
    letter-spacing: 0.5px;
    text-transform: uppercase;
}}
QLabel[class="stat-unit"] {{
    color: {c['text_tertiary']};
    font-size: {t['size_xs']};
}}

/* ── Combo Box ───────────────────────────────────────────── */
QComboBox {{
    background-color: {c['bg_input']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['sm']};
    padding: {s['sm']} {s['md']};
    font-size: {t['size_body']};
    min-height: 20px;
}}
QComboBox:hover {{
    border-color: {c['border_emphasis']};
}}
QComboBox:focus {{
    border-color: {c['accent']};
    border-width: 2px;
    padding: 7px 15px;  /* compensate for border width */
}}
QComboBox::drop-down {{
    border: none;
    width: 28px;
}}
QComboBox::down-arrow {{
    image: none;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-top: 5px solid {c['text_secondary']};
    margin-right: 8px;
}}
QComboBox QAbstractItemView {{
    background-color: {c['bg_elevated']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['sm']};
    padding: {s['xs']};
    selection-background-color: {c['accent_surface']};
    selection-color: {c['text_primary']};
    outline: none;
}}

/* ── Spin Box ────────────────────────────────────────────── */
QDoubleSpinBox, QSpinBox {{
    background-color: {c['bg_input']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['sm']};
    padding: {s['sm']} {s['md']};
    font-size: {t['size_body']};
}}
QDoubleSpinBox:hover, QSpinBox:hover {{
    border-color: {c['border_emphasis']};
}}
QDoubleSpinBox:focus, QSpinBox:focus {{
    border-color: {c['accent']};
    border-width: 2px;
    padding: 7px 15px;
}}
QDoubleSpinBox::up-button, QSpinBox::up-button,
QDoubleSpinBox::down-button, QSpinBox::down-button {{
    background-color: transparent;
    border: none;
    width: 20px;
}}

/* ── Line Edit ───────────────────────────────────────────── */
QLineEdit {{
    background-color: {c['bg_input']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['sm']};
    padding: {s['sm']} {s['md']};
    font-size: {t['size_body']};
}}
QLineEdit:hover {{
    border-color: {c['border_emphasis']};
}}
QLineEdit:focus {{
    border-color: {c['accent']};
    border-width: 2px;
    padding: 7px 15px;
}}

/* ── Text Edit / Log ─────────────────────────────────────── */
QTextEdit {{
    background-color: {c['bg_surface']};
    color: {c['text_primary']};
    border: 1px solid {c['border_subtle']};
    border-radius: {r['md']};
    font-family: {t['font_mono']};
    font-size: {t['size_sm']};
    padding: {s['md']};
}}

/* ── Push Button (M3 filled/tonal/text variants) ─────────── */
QPushButton {{
    background-color: {c['bg_overlay']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['sm']};
    padding: {s['sm']} {s['lg']};
    font-weight: {t['weight_semibold']};
    font-size: {t['size_body']};
}}
QPushButton:hover {{
    background-color: {c['bg_elevated']};
    border-color: {c['border_emphasis']};
}}
QPushButton:pressed {{
    background-color: {c['bg_surface']};
}}
QPushButton:disabled {{
    background-color: {c['bg_surface']};
    color: {c['text_disabled']};
    border-color: {c['border_subtle']};
}}

/* Primary button (M3 filled — accent blue) */
QPushButton[class="primary"] {{
    background-color: {c['accent']};
    color: #FFFFFF;
    border: none;
    font-weight: {t['weight_bold']};
}}
QPushButton[class="primary"]:hover {{
    background-color: {c['accent_hover']};
}}
QPushButton[class="primary"]:pressed {{
    background-color: {c['accent_pressed']};
}}

/* Danger button (Apple red) */
QPushButton[class="danger"] {{
    background-color: {c['red']};
    color: #FFFFFF;
    border: none;
}}
QPushButton[class="danger"]:hover {{
    background-color: #E03E36;
}}
QPushButton[class="danger"]:pressed {{
    background-color: #C43530;
}}

/* ── Progress Bar (M3 linear indicator) ──────────────────── */
QProgressBar {{
    background-color: {c['bg_overlay']};
    border: none;
    border-radius: {r['full']};
    text-align: center;
    color: {c['text_secondary']};
    height: 6px;
    font-size: 0px;  /* hide text for clean look */
}}
QProgressBar::chunk {{
    background-color: {c['accent']};
    border-radius: {r['full']};
}}

/* ── Table Widget ────────────────────────────────────────── */
QTableWidget {{
    background-color: {c['bg_surface']};
    color: {c['text_primary']};
    gridline-color: {c['border_subtle']};
    border: 1px solid {c['border_subtle']};
    border-radius: {r['md']};
    font-size: {t['size_body']};
}}
QTableWidget::item {{
    padding: {s['sm']} {s['md']};
    border-bottom: 1px solid {c['border_subtle']};
}}
QTableWidget::item:selected {{
    background-color: {c['accent_surface']};
    color: {c['text_primary']};
}}
QHeaderView::section {{
    background-color: {c['bg_elevated']};
    color: {c['text_secondary']};
    padding: {s['sm']} {s['md']};
    border: none;
    border-bottom: 1px solid {c['border_default']};
    border-right: 1px solid {c['border_subtle']};
    font-weight: {t['weight_semibold']};
    font-size: {t['size_sm']};
    letter-spacing: 0.3px;
    text-transform: uppercase;
}}

/* ── Scroll Bar (minimal macOS style) ────────────────────── */
QScrollBar:vertical {{
    background: {c['scroll_track']};
    width: 8px;
    border: none;
    margin: 0;
}}
QScrollBar::handle:vertical {{
    background: {c['scroll_handle']};
    border-radius: {r['full']};
    min-height: 32px;
}}
QScrollBar::handle:vertical:hover {{
    background: {c['scroll_handle_hover']};
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical,
QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {{
    height: 0;
    background: none;
}}
QScrollBar:horizontal {{
    background: {c['scroll_track']};
    height: 8px;
    border: none;
    margin: 0;
}}
QScrollBar::handle:horizontal {{
    background: {c['scroll_handle']};
    border-radius: {r['full']};
    min-width: 32px;
}}
QScrollBar::handle:horizontal:hover {{
    background: {c['scroll_handle_hover']};
}}
QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal,
QScrollBar::add-page:horizontal, QScrollBar::sub-page:horizontal {{
    width: 0;
    background: none;
}}

/* ── Dialog ──────────────────────────────────────────────── */
QDialog {{
    background-color: {c['bg_surface']};
    border-radius: {r['lg']};
}}

/* ── Stat Card (glassmorphism-inspired) ──────────────────── */
QFrame[class="stat-card"] {{
    background-color: {c['glass_bg']};
    border: 1px solid {c['glass_border']};
    border-radius: {r['lg']};
    padding: {s['md']};
}}
QFrame[class="stat-card"]:hover {{
    border-color: {c['border_emphasis']};
    background-color: {c['bg_elevated']};
}}

/* ── Focus Ring (Apple-style) ────────────────────────────── */
QComboBox:focus, QDoubleSpinBox:focus, QSpinBox:focus,
QLineEdit:focus {{
    outline: none;
}}

/* ── Tooltip ─────────────────────────────────────────────── */
QToolTip {{
    background-color: {c['bg_elevated']};
    color: {c['text_primary']};
    border: 1px solid {c['border_default']};
    border-radius: {r['sm']};
    padding: {s['sm']} {s['md']};
    font-size: {t['size_sm']};
}}

/* ── Checkbox / Radio ────────────────────────────────────── */
QCheckBox, QRadioButton {{
    color: {c['text_primary']};
    spacing: {s['sm']};
    background: transparent;
}}
QCheckBox::indicator, QRadioButton::indicator {{
    width: 16px;
    height: 16px;
}}
"""
