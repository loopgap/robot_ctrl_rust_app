"""TUI custom widgets.

Provides reusable UI components for the sim_platform TUI:
- InfoCard: Clickable scenario cards with icon and description
- StatCard: Metric display cards with conditional color coding
- ValidatedInput: Input widget with real-time validation and visual feedback
- HelpPanel: Collapsible help panel with keyboard shortcuts
- SparkLine: ASCII sparkline chart for time-series data
- ErrorDialog / ConfirmDialog: Modal dialog screens
"""

from .cards import InfoCard, StatCard
from .dialogs import ConfirmDialog, ErrorDialog
from .help_panel import HelpPanel
from .sparkline import SparkLine
from .validators import ValidatedInput

__all__ = [
    "InfoCard",
    "StatCard",
    "ValidatedInput",
    "HelpPanel",
    "SparkLine",
    "ErrorDialog",
    "ConfirmDialog",
]
