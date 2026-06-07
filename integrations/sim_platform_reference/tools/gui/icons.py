"""Custom SVG-based icon system for sim_platform GUI.

Replaces all emoji-based icons with a cohesive industrial-themed
icon set. Uses raw SVG rendering via QPixmap for pixel-perfect
display at any DPI.

Icon categories:
- Application: app icon, window identity
- Navigation: home, chart, log, results
- Actions: run, pause, stop, save, open, export
- Status: success, warning, error, info, pending
- Hardware: motor, battery, inverter, sensor
- Controls: settings, filter, search, reset

Design principles:
- Consistent 24x24 viewBox
- 2px stroke width for outlines
- Industrial blue-gray palette
- Sharp, technical aesthetic (no rounded whimsy)
"""

from __future__ import annotations

from PySide6.QtCore import QRectF, Qt
from PySide6.QtGui import (
    QColor,
    QIcon,
    QPainter,
    QPen,
    QPixmap,
)

# QSvgRenderer may be in different locations depending on PySide6 version
try:
    from PySide6.QtSvg import QSvgRenderer
except ImportError:
    try:
        from PySide6.QtSvgWidgets import QSvgRenderer
    except ImportError:
        QSvgRenderer = None


# ── Color Palette ─────────────────────────────────────────
ICON_ACTIVE = "#0A84FF"       # Primary action blue
ICON_DEFAULT = "#B0B8C8"      # Neutral steel blue
ICON_DISABLED = "#5A5E6A"     # Muted gray
ICON_SUCCESS = "#30D158"      # Apple green
ICON_WARNING = "#FFD60A"      # Apple yellow
ICON_ERROR = "#FF453A"        # Apple red
ICON_ACCENT = "#64D2FF"       # Teal highlight
ICON_SURFACE = "#242830"      # Dark surface for filled areas
ICON_WHITE = "#F5F5F7"        # Primary text on dark


# ── SVG Icon Definitions (24x24 viewBox) ───────────────────

_SVG_ICONS: dict[str, str] = {
    # ═════════════════════════════════════════════════════════
    # Navigation
    # ═════════════════════════════════════════════════════════
    "home": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <polyline points="9 22 9 12 15 12 15 22" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "chart": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "log": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <polyline points="14 2 14 8 20 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <line x1="8" y1="13" x2="16" y2="13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="8" y1="17" x2="16" y2="17" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="8" y1="9" x2="10" y2="9" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "results": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" stroke-width="2"/>
      <line x1="3" y1="9" x2="21" y2="9" stroke="currentColor" stroke-width="2"/>
      <line x1="9" y1="21" x2="9" y2="9" stroke="currentColor" stroke-width="2"/>
    </svg>''',

    # ═════════════════════════════════════════════════════════
    # Actions
    # ═════════════════════════════════════════════════════════
    "run": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polygon points="6 3 20 12 6 21 6 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="currentColor" fill-opacity="0.15"/>
    </svg>''',

    "pause": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <rect x="5" y="4" width="5" height="16" rx="1" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.15"/>
      <rect x="14" y="4" width="5" height="16" rx="1" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.15"/>
    </svg>''',

    "stop": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <rect x="4" y="4" width="16" height="16" rx="2" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.15"/>
    </svg>''',

    "restart": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polyline points="1 4 1 10 7 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M3.51 15a9 9 0 102.13-9.36L1 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "save": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <polyline points="17 21 17 13 7 13 7 21" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <polyline points="7 3 7 8 15 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "open": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2v11z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "export": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <polyline points="7 10 12 15 17 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <line x1="12" y1="15" x2="12" y2="3" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "new_config": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <line x1="12" y1="5" x2="12" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <rect x="3" y="3" width="18" height="18" rx="3" stroke="currentColor" stroke-width="2"/>
    </svg>''',

    "settings": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    # ═════════════════════════════════════════════════════════
    # Status
    # ═════════════════════════════════════════════════════════
    "success": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.1"/>
      <polyline points="7 12 11 16 17 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "warning": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="currentColor" fill-opacity="0.1"/>
      <line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <circle cx="12" cy="17" r="1" fill="currentColor"/>
    </svg>''',

    "error": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.1"/>
      <line x1="8" y1="8" x2="16" y2="16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="16" y1="8" x2="8" y2="16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "info": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.1"/>
      <line x1="12" y1="16" x2="12" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <circle cx="12" cy="8" r="1" fill="currentColor"/>
    </svg>''',

    "pending": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
      <polyline points="12 6 12 12 16 14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    # ═════════════════════════════════════════════════════════
    # Hardware / Simulation
    # ═════════════════════════════════════════════════════════
    "motor": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="2"/>
      <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.1"/>
      <path d="M12 4v-2M12 22v-2M4 12H2M22 12h-2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <path d="M6.34 6.34l-1.42-1.42M19.08 19.08l-1.42-1.42M17.66 6.34l1.42-1.42M4.92 19.08l1.42-1.42" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "battery": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <rect x="1" y="6" width="18" height="12" rx="2" stroke="currentColor" stroke-width="2"/>
      <line x1="23" y1="10" x2="23" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="7" y1="10" x2="7" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="10" y1="10" x2="10" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="13" y1="10" x2="13" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "inverter": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <rect x="2" y="4" width="20" height="16" rx="2" stroke="currentColor" stroke-width="2"/>
      <path d="M6 8l3 4-3 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <line x1="10" y1="12" x2="18" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <path d="M18 8l-3 4 3 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "sensor": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M12 1a3 3 0 00-3 3v8a3 3 0 006 0V4a3 3 0 00-3-3z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M19 10v2a7 7 0 01-14 0v-2" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <line x1="12" y1="19" x2="12" y2="23" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="8" y1="23" x2="16" y2="23" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "thermal": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M14 14.76V3.5a2.5 2.5 0 00-5 0v11.26a4.5 4.5 0 105 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    # ═════════════════════════════════════════════════════════
    # Controls
    # ═════════════════════════════════════════════════════════
    "filter": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "search": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
      <line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "delete": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polyline points="3 6 5 6 21 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "confirm": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polyline points="20 6 9 17 4 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "close": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    # ═════════════════════════════════════════════════════════
    # Simulation-specific
    # ═════════════════════════════════════════════════════════
    "simulation": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <path d="M12 2l9 4.5v7L12 18l-9-4.5v-7L12 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="currentColor" fill-opacity="0.1"/>
      <path d="M12 18v4M8 22h8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>''',

    "solver": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <polyline points="16 3 21 3 21 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <line x1="21" y1="3" x2="14" y2="10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <polyline points="8 21 3 21 3 16" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <line x1="3" y1="21" x2="10" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <path d="M14 14l1 1M9 9l2 2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    "params": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.1"/>
    </svg>''',

    "conflict": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
      <path d="M8 8l8 8M8 16l4-4 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <circle cx="12" cy="12" r="2" fill="currentColor"/>
    </svg>''',

    "tour": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
      <path d="M12 16v-4M12 8h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>''',

    # ═════════════════════════════════════════════════════════
    # App Icon (larger, more detailed)
    # ═════════════════════════════════════════════════════════
    "app_icon": '''
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none">
      <rect x="4" y="4" width="56" height="56" rx="12" stroke="currentColor" stroke-width="3" fill="currentColor" fill-opacity="0.08"/>
      <circle cx="32" cy="32" r="14" stroke="currentColor" stroke-width="2.5" fill="currentColor" fill-opacity="0.06"/>
      <circle cx="32" cy="32" r="5" stroke="currentColor" stroke-width="2" fill="currentColor" fill-opacity="0.12"/>
      <path d="M32 18v-4M32 50v-4M18 32h-4M50 32h-4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      <path d="M22.1 22.1l-2.83-2.83M44.73 44.73l-2.83-2.83M41.9 22.1l2.83-2.83M19.27 44.73l2.83-2.83" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
    </svg>''',
}


# ── Public API ─────────────────────────────────────────────

class SimIcons:
    """Centralized icon accessor with caching and color variants.

    Usage:
        from sim_platform.tools.gui.icons import SimIcons

        icons = SimIcons()
        button.setIcon(icons.get("run"))
        button.setIcon(icons.get("stop", color="#FF453A"))
        label.setPixmap(icons.pixmap("success", size=16))
    """

    _cache: dict[str, QIcon] = {}
    _pixmap_cache: dict[str, QPixmap] = {}
    _icon_colors: dict[str, str] = {
        "run": ICON_SUCCESS,
        "stop": ICON_ERROR,
        "pause": ICON_WARNING,
        "success": ICON_SUCCESS,
        "warning": ICON_WARNING,
        "error": ICON_ERROR,
    }

    def __init__(self, default_color: str = ICON_DEFAULT):
        self._default_color = default_color

    def get(self, name: str, color: str | None = None, size: int = 20) -> QIcon:
        """Get a QIcon with optional color override.

        Args:
            name: Icon name (key in _SVG_ICONS)
            color: Override color (CSS color string). None = default per icon type.
            size: Icon size in pixels.

        Returns:
            QIcon instance.
        """
        if color is None:
            color = self._icon_colors.get(name, self._default_color)

        cache_key = f"{name}_{color}_{size}"
        if cache_key in self._cache:
            return self._cache[cache_key]

        svg = _SVG_ICONS.get(name)
        if svg is None:
            # Return empty icon for missing
            pixmap = QPixmap(size, size)
            pixmap.fill(Qt.GlobalColor.transparent)
            return QIcon(pixmap)

        # Colorize the SVG
        colored_svg = svg.replace('stroke="currentColor"', f'stroke="{color}"')
        colored_svg = colored_svg.replace('fill="currentColor"', f'fill="{color}"')

        pixmap = _svg_to_pixmap(colored_svg, size, size)
        icon = QIcon(pixmap)
        self._cache[cache_key] = icon
        return icon

    def pixmap(self, name: str, color: str | None = None, size: int = 24) -> QPixmap:
        """Get a QPixmap directly.

        Args:
            name: Icon name.
            color: Override color.
            size: Icon size in pixels.

        Returns:
            QPixmap instance.
        """
        if color is None:
            color = self._icon_colors.get(name, self._default_color)

        cache_key = f"pm_{name}_{color}_{size}"
        if cache_key in self._pixmap_cache:
            return self._pixmap_cache[cache_key]

        svg = _SVG_ICONS.get(name, "")
        colored_svg = svg.replace('stroke="currentColor"', f'stroke="{color}"')
        colored_svg = colored_svg.replace('fill="currentColor"', f'fill="{color}"')

        pixmap = _svg_to_pixmap(colored_svg, size, size)
        self._pixmap_cache[cache_key] = pixmap
        return pixmap

    def app_icon(self, size: int = 64) -> QIcon:
        """Get the application icon."""
        return self.get("app_icon", color=ICON_ACCENT, size=size)

    @classmethod
    def clear_cache(cls):
        """Clear all icon caches (memory management)."""
        cls._cache.clear()
        cls._pixmap_cache.clear()


# ── Internal Helpers ───────────────────────────────────────

def _svg_to_pixmap(svg: str, width: int, height: int) -> QPixmap:
    """Render SVG to QPixmap using QSvgRenderer if available.

    Falls back to a simple placeholder if SVG rendering is not available.
    Uses devicePixelRatio-aware rendering for HiDPI displays.
    """
    from PySide6.QtWidgets import QApplication

    # HiDPI support
    if QApplication.instance():
        ratio = QApplication.instance().devicePixelRatio()
    else:
        ratio = 1.0

    scaled_width = int(width * ratio)
    scaled_height = int(height * ratio)

    pixmap = QPixmap(scaled_width, scaled_height)
    pixmap.fill(Qt.GlobalColor.transparent)
    pixmap.setDevicePixelRatio(ratio)

    if QSvgRenderer is not None:
        try:
            renderer = QSvgRenderer(svg.encode('utf-8'))
            painter = QPainter(pixmap)
            renderer.render(painter)
            painter.end()
            return pixmap
        except Exception:
            pass  # Fall through to fallback

    # Fallback: draw a simple placeholder icon
    painter = QPainter(pixmap)
    painter.setRenderHint(QPainter.RenderHint.Antialiasing)
    painter.setPen(QPen(QColor("#B0B8C8"), max(1, int(width * 0.08))))
    painter.setBrush(QColor(255, 255, 255, 30))
    margin = width * 0.15
    painter.drawRoundedRect(
        QRectF(margin, margin, width - 2 * margin, height - 2 * margin),
        2, 2,
    )
    painter.end()
    return pixmap


# ── Convenience module-level instance ──────────────────────

_default_icons = SimIcons()


def get_icon(name: str, color: str | None = None, size: int = 20) -> QIcon:
    """Module-level convenience function for getting icons."""
    return _default_icons.get(name, color, size)


def get_pixmap(name: str, color: str | None = None, size: int = 24) -> QPixmap:
    """Module-level convenience function for getting pixmaps."""
    return _default_icons.pixmap(name, color, size)


# ── Animation Icons (spinner variants) ─────────────────────

SPINNER_SVG = '''
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
  <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
</svg>'''
