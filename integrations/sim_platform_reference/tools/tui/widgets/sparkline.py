"""SparkLine ASCII chart widget for the sim_platform TUI.

Renders a compact sparkline chart using ASCII characters, suitable for
displaying time-series trends (e.g., speed, torque) inline in the terminal.
"""

import math

from textual.reactive import reactive
from textual.widgets import Static

# ASCII characters for sparkline rendering, from low to high.
# Uses block characters for a clean terminal look.
SPARK_CHARS = [" ", " ", ".", ":", "-", "=", "+", "*", "#", "%", "@"]


def _render_sparkline(
    data: list[float],
    width: int = 60,
    chars: list[str] = SPARK_CHARS,
) -> str:
    """Render a list of floats as an ASCII sparkline string.

    Args:
        data: List of float values to plot.
        width: Desired output character width (will be downsampled if needed).
        chars: List of characters ordered from low to high intensity.

    Returns:
        A string of sparkline characters.
    """
    if not data:
        return ""

    # Filter out non-finite values
    finite_data = [v for v in data if math.isfinite(v)]
    if not finite_data:
        return ""

    min_val = min(finite_data)
    max_val = max(finite_data)
    value_range = max_val - min_val

    # Downsample or upsample to target width
    n = len(finite_data)
    if n <= width:
        sampled = finite_data
    else:
        # Simple averaging downsample
        step = n / width
        sampled = []
        for i in range(width):
            start = int(i * step)
            end = min(int((i + 1) * step), n)
            chunk = finite_data[start:end]
            if chunk:
                sampled.append(sum(chunk) / len(chunk))
            else:
                sampled.append(finite_data[-1])

    # Map values to characters
    num_chars = len(chars)
    result = []
    for val in sampled:
        if value_range == 0:
            idx = num_chars // 2
        else:
            normalized = (val - min_val) / value_range
            idx = int(normalized * (num_chars - 1))
        idx = max(0, min(idx, num_chars - 1))
        result.append(chars[idx])

    return "".join(result)


class SparkLine(Static):
    """A compact ASCII sparkline chart widget.

    Displays a time-series as a single-line sparkline with optional
    title, min/max labels, and configurable width.

    Attributes:
        chart_title: Title displayed above the sparkline.
        data: The data points to render.
        chart_width: Character width for the sparkline.
        min_label: Text for the minimum value label.
        max_label: Text for the maximum value label.
    """

    chart_title: str
    data: reactive[list[float]] = reactive(list)
    chart_width: int
    min_label: reactive[str] = reactive("")
    max_label: reactive[str] = reactive("")

    def __init__(
        self,
        data: list[float] | None = None,
        title: str = "Sparkline",
        width: int = 60,
        id: str | None = None,
        classes: str | None = None,
    ):
        """Initialize SparkLine.

        Args:
            data: Initial data points (list of floats).
            title: Title text displayed above the chart.
            width: Character width for the sparkline.
            id: Optional Textual widget id.
            classes: Optional CSS class names.
        """
        super().__init__(id=id, classes=classes)
        self.chart_title = title
        self.data = data or []
        self.chart_width = width

    def compose(self):
        """SparkLine is rendered as a Static; compose returns nothing."""
        return []

    def render(self) -> str:
        """Render the sparkline as a styled string.

        Returns:
            Multi-line string with title, sparkline, and min/max labels.
        """
        lines = []

        # Title
        if self.chart_title:
            lines.append(f"[bold]{self.chart_title}[/]")

        # Sparkline
        if not self.data:
            lines.append("[dim](no data)[/]")
        else:
            spark_str = _render_sparkline(self.data, self.chart_width)
            lines.append(f"[cyan]{spark_str}[/]")

            # Min / Max labels
            finite_data = [v for v in self.data if math.isfinite(v)]
            if finite_data:
                min_val = min(finite_data)
                max_val = max(finite_data)
                min_lbl = self.min_label or f"{min_val:.1f}"
                max_lbl = self.max_label or f"{max_val:.1f}"
                # Right-align min, left-align max
                padding = max(0, self.chart_width - len(min_lbl) - len(max_lbl))
                lines.append(
                    f"[dim]{min_lbl}{' ' * padding}{max_lbl}[/]"
                )

        return "\n".join(lines)

    def update_data(
        self,
        data: list[float],
        title: str | None = None,
        min_label: str = "",
        max_label: str = "",
    ) -> None:
        """Update the sparkline data and re-render.

        Args:
            data: New list of float data points.
            title: Optional new title (keeps current if None).
            min_label: Optional custom minimum label.
            max_label: Optional custom maximum label.
        """
        self.data = data
        if title is not None:
            self.chart_title = title
        self.min_label = min_label
        self.max_label = max_label
        self.refresh()
