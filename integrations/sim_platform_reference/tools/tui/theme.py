"""Design tokens and CSS for the sim_platform TUI.

Provides a unified color palette, spacing system, and comprehensive CSS
styles that support modern card-based layouts, validated forms, and
enhanced visualization components.

Color scheme: Catppuccin Mocha (modern dark theme)
"""

# ════════════════════════════════════════════════════════════
#  Design Tokens
# ════════════════════════════════════════════════════════════

# Color palette - Catppuccin Mocha inspired
COLORS = {
    # Base colors
    "base": "#1E1E2E",          # Main background
    "mantle": "#181825",        # Darker background
    "crust": "#11111B",         # Darkest background

    # Surface colors
    "surface0": "#313244",      # Elevated surface
    "surface1": "#45475A",      # Higher elevation
    "surface2": "#585B70",      # Highest elevation

    # Overlay colors
    "overlay0": "#6C7086",      # Subtle overlay
    "overlay1": "#7F849C",      # Medium overlay
    "overlay2": "#9399B2",      # Strong overlay

    # Text colors
    "text": "#CDD6F4",          # Primary text
    "subtext1": "#BAC2DE",      # Secondary text
    "subtext0": "#A6ADC8",      # Tertiary text

    # Accent colors
    "lavender": "#B4BEFE",      # Soft accent
    "blue": "#89B4FA",          # Primary action
    "sapphire": "#74C7EC",      # Info
    "sky": "#89DCEB",           # Light info
    "teal": "#94E2D5",          # Success light
    "green": "#A6E3A1",         # Success
    "yellow": "#F9E2AF",        # Warning
    "peach": "#FAB387",         # Warning dark
    "maroon": "#EBA0AC",        # Error light
    "red": "#F38BA8",           # Error
    "mauve": "#CBA6F7",         # Special
    "pink": "#F5C2E7",          # Accent
    "flamingo": "#F2CDCD",      # Accent light
    "rosewater": "#F5E0DC",     # Accent subtle
}

# Spacing scale (in Textual units)
SPACING = {
    "xs": "0 1",
    "sm": "0 2",
    "md": "1 2",
    "lg": "2 3",
    "xl": "3 4",
}

# Border styles
BORDERS = {
    "thin": "tall $surface1",
    "rounded": "round $surface1",
    "thick": "thick $surface1",
    "focus": "tall $blue",
}


# ════════════════════════════════════════════════════════════
#  Global CSS
# ════════════════════════════════════════════════════════════

TUI_CSS = """
/* ── Variables (Catppuccin Mocha) ── */
$base: #1E1E2E;
$mantle: #181825;
$crust: #11111B;
$surface0: #313244;
$surface1: #45475A;
$surface2: #585B70;
$overlay0: #6C7086;
$overlay1: #7F849C;
$overlay2: #9399B2;
$text: #CDD6F4;
$subtext1: #BAC2DE;
$subtext0: #A6ADC8;
$lavender: #B4BEFE;
$blue: #89B4FA;
$sapphire: #74C7EC;
$sky: #89DCEB;
$teal: #94E2D5;
$green: #A6E3A1;
$yellow: #F9E2AF;
$peach: #FAB387;
$maroon: #EBA0AC;
$red: #F38BA8;
$mauve: #CBA6F7;
$pink: #F5C2E7;
$flamingo: #F2CDCD;
$rosewater: #F5E0DC;

/* ── Global ── */
Screen {
    background: $base;
    color: $text;
}

/* ── Header ── */
Header {
    background: $mantle;
    color: $text;
    border-bottom: solid $surface0;
    padding: 0 1;
}
Header .header--title {
    color: $blue;
    text-style: bold;
}
Header .header--subtitle {
    color: $subtext0;
}
Header .header--clock {
    color: $subtext1;
}

/* ── Footer ── */
Footer {
    background: $mantle;
    color: $subtext0;
    border-top: solid $surface0;
    padding: 0 1;
}
Footer .footer--key {
    color: $blue;
    text-style: bold;
}
Footer .footer--description {
    color: $subtext1;
}

/* ── Main container ── */
.main-container, .config-container, .run-container,
.results-container, .scan-container {
    padding: 1 2;
    height: 100%;
}

/* ── App title ── */
#app-title {
    text-style: bold;
    text-align: center;
    padding: 0 0 1 0;
    color: $blue;
    text-align: center;
}
#app-version {
    text-align: center;
    color: $overlay1;
}

/* ── Section titles ── */
.section-title {
    padding: 1 0;
    text-style: bold;
    color: $lavender;
}

/* ── Buttons ── */
.button-row {
    height: 3;
    align: center middle;
    margin: 1 0;
}
.button-row Button {
    margin: 0 1;
    min-width: 16;
}
Button {
    margin: 0;
    background: $surface0;
    color: $text;
    border: none;
    padding: 0 2;
}
Button:hover {
    background: $surface1;
    color: $text;
}
Button:focus {
    background: $surface1;
    border: tall $blue;
    color: $text;
}
Button.-primary {
    background: $blue;
    color: $crust;
}
Button.-primary:hover {
    background: $lavender;
    color: $crust;
}
Button.-primary:focus {
    background: $lavender;
    border: tall $blue;
    color: $crust;
}
Button.-success {
    background: $green;
    color: $crust;
}
Button.-success:hover {
    background: $teal;
    color: $crust;
}
Button.-warning {
    background: $yellow;
    color: $crust;
}
Button.-error {
    background: $red;
    color: $crust;
}
Button:disabled {
    background: $surface0;
    color: $overlay0;
}

/* ── Select / Input / Label ── */
Label {
    padding: 1 0 0 0;
    text-style: bold;
    color: $subtext1;
}
Input {
    margin: 0 0 0 0;
    background: $surface0;
    color: $text;
    border: tall $surface1;
}
Input:focus {
    background: $surface0;
    border: tall $blue;
    color: $text;
}
Select {
    margin: 0 0 0 0;
    background: $surface0;
    color: $text;
    border: tall $surface1;
}
Select:focus {
    background: $surface0;
    border: tall $blue;
    color: $text;
}

/* ── Validation state classes ── */
.input-valid {
    border: tall $green;
}
.input-invalid {
    border: tall $red;
}
.validation-msg {
    color: $red;
    padding: 0 0 0 1;
    height: auto;
}
.validation-msg-valid {
    color: $green;
    padding: 0 0 0 1;
    height: auto;
}

/* ── InfoCard widget ── */
InfoCard {
    background: $surface0;
    border: round $surface1;
    padding: 1 2;
    margin: 0 0 1 0;
    height: auto;
}
InfoCard:hover {
    background: $surface1;
    border: round $blue;
}
InfoCard:focus {
    background: $surface1;
    border: tall $blue;
}
.card-title {
    text-style: bold;
    color: $blue;
    padding: 0 0 0 0;
}
.card-value {
    text-style: bold;
    color: $text;
    padding: 0 0 0 0;
    text-align: right;
}
.card-description {
    color: $subtext0;
    padding: 0 0 0 0;
}
.card-icon {
    text-style: bold;
    color: $mauve;
    width: 3;
}

/* ── StatCard widget ── */
StatCard {
    background: $surface0;
    border: round $surface1;
    padding: 1 2;
    margin: 0 1 0 0;
    width: 1fr;
    height: auto;
}
.stat-label {
    color: $subtext0;
    text-style: bold;
    padding: 0 0 0 0;
    text-align: center;
}
.stat-value {
    text-style: bold;
    color: $text;
    padding: 0 0 0 0;
    text-align: center;
}
.stat-unit {
    color: $overlay1;
    padding: 0 0 0 0;
    text-align: center;
}
.stat-good {
    color: $green;
}
.stat-warn {
    color: $yellow;
}
.stat-bad {
    color: $red;
}

/* ── SparkLine widget ── */
SparkLine {
    background: $surface0;
    border: round $surface1;
    padding: 0 1;
    height: auto;
    min-height: 5;
}
.sparkline-title {
    color: $subtext0;
    text-style: bold;
    padding: 0 0 0 0;
}
.sparkline-chart {
    color: $blue;
    padding: 0 0 0 0;
}
.sparkline-min, .sparkline-max {
    color: $overlay1;
    padding: 0 0 0 0;
}

/* ── HelpPanel widget ── */
HelpPanel {
    background: $surface0;
    border: round $blue;
    padding: 1 2;
    margin: 0 0 1 0;
    height: auto;
    max-height: 16;
}
HelpPanel.hidden {
    display: none;
}
.help-title {
    text-style: bold;
    color: $blue;
    padding: 0 0 1 0;
}
.help-section {
    text-style: bold;
    color: $text;
    padding: 1 0 0 0;
}
.help-item {
    color: $subtext0;
    padding: 0 0 0 2;
}
.help-key {
    text-style: bold;
    color: $mauve;
}
.help-desc {
    color: $subtext1;
}

/* ── Preset scenario cards container ── */
.preset-grid {
    layout: grid;
    grid-size: 2 2;
    grid-gutter: 1;
    margin: 1 0;
}

/* ── Progress / Log ── */
ProgressBar {
    margin: 1 0;
    background: $surface0;
}
ProgressBar .bar--bar {
    color: $blue;
}
ProgressBar .bar--complete {
    color: $green;
}
RichLog {
    border: solid $surface1;
    background: $mantle;
    height: 60%;
    margin: 1 0;
}

/* ── DataTable ── */
DataTable {
    height: 60%;
    margin: 1 0;
    background: $surface0;
}
DataTable > .datatable--header {
    background: $surface1;
    color: $text;
    text-style: bold;
}
DataTable > .datatable--row {
    background: $surface0;
    color: $text;
}
DataTable > .datatable--row:hover {
    background: $surface1;
}

/* ── Error/Confirm dialog ── */
#error-dialog, #confirm-dialog {
    grid-size: 1;
    grid-gutter: 1;
    padding: 2 4;
    width: 60;
    height: auto;
    background: $surface0;
    border: thick $red;
    margin: 4 8;
}
#confirm-dialog {
    border: thick $yellow;
}
#err-title, #confirm-title {
    text-style: bold;
    text-align: center;
    padding: 0 0 1 0;
    color: $text;
}
#err-msg, #confirm-msg {
    text-align: center;
    color: $subtext1;
}
#err-detail {
    color: $overlay1;
    text-align: center;
}

/* ── Run screen ── */
#run-title {
    padding: 0 0 1 0;
    color: $green;
}
#run-stats {
    text-align: center;
    padding: 0 0 1 0;
    color: $subtext1;
}
#run-log {
    height: 50%;
    border: solid $surface1;
    background: $mantle;
}

/* ── Run multi-stage progress ── */
.stage-indicator {
    padding: 0 0 1 0;
    text-align: center;
}
.stage-active {
    color: $blue;
    text-style: bold;
}
.stage-done {
    color: $green;
}
.stage-pending {
    color: $overlay0;
}
.run-stats-panel {
    background: $surface0;
    border: round $surface1;
    padding: 1 2;
    margin: 0 0 1 0;
    height: auto;
}
.run-stats-title {
    color: $subtext0;
    text-style: bold;
    padding: 0 0 0 0;
}

/* ── Results ── */
#results-title {
    padding: 0 0 1 0;
    color: $green;
}
#results-stats {
    text-align: center;
    text-style: bold;
    padding: 0 0 1 0;
    color: $subtext1;
}

/* ── Config / Scan ── */
#config-title, #scan-title {
    padding: 0 0 1 0;
    color: $yellow;
}
#scan-progress {
    margin: 1 0;
}
#scan-log {
    height: 50%;
}
#scan-values {
    margin: 0 0 1 0;
}

/* ── Scrollbar ── */
.scrollbar--horizontal,
.scrollbar--vertical {
    background: $surface0;
}
.scrollbar--horizontal:hover,
.scrollbar--vertical:hover {
    background: $surface1;
}
.scrollbar--thumb {
    background: $overlay0;
}
.scrollbar--thumb:hover {
    background: $overlay1;
}

/* ── Tabs ── */
Tabs {
    background: $mantle;
}
Tabs > .tabs--tab {
    background: $surface0;
    color: $subtext0;
    border: none;
    padding: 0 2;
}
Tabs > .tabs--tab:hover {
    background: $surface1;
    color: $text;
}
Tabs > .tabs--tab.-active {
    background: $blue;
    color: $crust;
    text-style: bold;
}
"""
