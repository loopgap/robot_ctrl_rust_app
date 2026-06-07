"""Help content data for the sim_platform TUI.

Each HELP_* dict maps section titles to lists of (key, description) tuples
that are rendered by the HelpPanel widget.
"""


# Type alias for help entries: list of (shortcut_key, description)
HelpEntries = list[tuple[str, str]]
HelpContent = dict[str, HelpEntries]

# ════════════════════════════════════════════════════════════
#  Main Dashboard Help
# ════════════════════════════════════════════════════════════

HELP_MAIN: HelpContent = {
    "Keyboard Shortcuts": [
        ("R", "Start a new simulation run"),
        ("C", "Open parameter configuration"),
        ("S", "Open the parameter scanner"),
        ("F1", "Toggle this help panel"),
        ("Q", "Quit the application"),
    ],
    "Preset Scenarios": [
        ("Step Response", "100 rad/s step input, measures settling time"),
        ("Ramp Test", "Smooth acceleration 0 → 100 rad/s over duration"),
        ("Load Disturbance", "0.3 N·m load torque applied at t=0.5s"),
        ("Voltage Sag", "20V bus voltage sag, tests controller recovery"),
    ],
    "Navigation": [
        ("Tab / Shift+Tab", "Cycle through interactive elements"),
        ("Enter / Space", "Activate selected button"),
        ("Escape", "Return to previous screen"),
        ("Ctrl+H", "Return to home (main dashboard)"),
    ],
}


# ════════════════════════════════════════════════════════════
#  Configuration Screen Help
# ════════════════════════════════════════════════════════════

HELP_CONFIG: HelpContent = {
    "Keyboard Shortcuts": [
        ("R", "Run simulation with current settings"),
        ("F1", "Toggle this help panel"),
        ("Escape", "Return to main dashboard"),
    ],
    "Parameters": [
        ("Speed Reference", "Target motor speed in rad/s (range: 5–500)"),
        ("Duration", "Simulation time span in seconds (range: 0.1–60)"),
        ("Load Torque", "External load in N·m (range: 0–10)"),
    ],
    "Motor Presets": [
        ("Small PMSM (200W drone)", "Lightweight motor for UAV applications"),
        ("Medium PMSM (2kW e-bike)", "Mid-range motor for electric bicycles"),
        ("Large PMSM (20kW EV)", "High-power motor for electric vehicles"),
    ],
    "Validation": [
        ("Real-time", "Fields are validated as you type; red border = invalid"),
        ("Range checks", "Numeric values are bounded to safe operating ranges"),
        ("NaN/Inf", "Non-finite values are rejected and replaced with defaults"),
    ],
}


# ════════════════════════════════════════════════════════════
#  Run Screen Help
# ════════════════════════════════════════════════════════════

HELP_RUN: HelpContent = {
    "Keyboard Shortcuts": [
        ("Escape", "View results (if complete) or go back to config"),
        ("Q", "Quit the application"),
    ],
    "Progress Stages": [
        ("Init", "Models and sensors are initialized"),
        ("Simulate", "Main simulation loop is running"),
        ("Log", "Saving results to HDF5 file"),
        ("Plot", "Generating matplotlib plot (if enabled)"),
    ],
    "Statistics Panel": [
        ("Speed", "Current motor speed in rad/s"),
        ("Torque", "Current output torque in N·m"),
        ("FPS", "Simulation steps per second (throughput)"),
        ("Progress", "Percentage of simulation completed"),
    ],
}


# ════════════════════════════════════════════════════════════
#  Scan Screen Help
# ════════════════════════════════════════════════════════════

HELP_SCAN: HelpContent = {
    "Keyboard Shortcuts": [
        ("S", "Start the parameter scan"),
        ("Escape", "Return to main dashboard"),
    ],
    "Scannable Parameters": [
        ("Speed Reference", "Test multiple target speeds"),
        ("FOC kp_id", "Sweep d-axis proportional gain"),
        ("FOC ki_id", "Sweep d-axis integral gain"),
        ("Speed Loop Kp", "Sweep speed controller proportional gain"),
        ("Load Torque", "Test multiple load conditions"),
    ],
    "Values Input": [
        ("Format", "Comma-separated numbers, e.g. 50, 100, 150, 200"),
        ("Minimum", "At least 2 values are required"),
        ("Validation", "NaN and Inf values are rejected"),
    ],
}
