"""Core constants — centralized magic number management.

All simulation platform constants defined here as single source of truth.
Modules should import from here instead of hardcoding values.

Usage:
    from sim_platform.core.constants import MOTOR_EPS_L, DEFAULT_V_BUS

Tuning Guide:
  - MOTOR_EPS_L: Increase if your motor has very small inductance (<1uH).
    Decrease if you need finer resolution for air-core motors.
  - MOTOR_EPS_J: Usually fine at 1e-15. Increase only for ultra-light rotors.
  - DEFAULT_I_MAX: Maximum current for overflow protection. Increase for
    high-power motors (>200A). Affects PMSMAdvanced copper_loss calculation.
  - MAX_TOTAL_STEPS: DoS guard. Increase for long simulations (>50M steps).
"""

# ── Numerical safety ─────────────────────────────────────────
# These prevent division by near-zero and floating-point underflow.
# Adjust if your application uses extreme parameter values.

NUMERIC_EPS: float = 1e-12       # General-purpose epsilon for zero-divide guards
MOTOR_EPS_L: float = 1e-9       # Minimum inductance [H] (1 nH).
                                 # Ld/Lq clamped to this at init.
                                 # If Ld < MOTOR_EPS_L, electrical dynamics freeze.
MOTOR_EPS_J: float = 1e-15      # Minimum inertia [kg·m²].
                                 # J clamped to this at init.
                                 # If J < MOTOR_EPS_J, mechanical dynamics blow up.
PWM_EPS_V: float = 1e-12        # Minimum bus voltage [V] for SVPWM.
                                 # If v_bus < PWM_EPS_V, SVPWM returns 50% duty.

# ── Default motor parameters ─────────────────────────────────
# These are used when parameters are not specified or are invalid.
# Typical small servo motor values (100W class).

DEFAULT_V_BUS: float = 48.0      # Default DC bus voltage [V]
DEFAULT_V_BUS_BLDC: float = 24.0 # Default BLDC bus voltage [V]
DEFAULT_DT_S: float = 50e-6     # Default simulation time step [s].
                                 # 50us = 20kHz switching frequency.
                                 # Reduce to 10us for high-speed motors (>10kRPM).
DEFAULT_I_MAX: float = 200.0    # Default maximum current [A].
                                 # Used for overflow protection in PMSMAdvanced.
                                 # Increase for high-power motors.
DEFAULT_FLUX_PM: float = 0.03   # Default PM flux linkage [Wb]
DEFAULT_RS: float = 0.1         # Default stator resistance [Ω]
DEFAULT_LD: float = 0.5e-3      # Default d-axis inductance [H]
DEFAULT_LQ: float = 1.0e-3      # Default q-axis inductance [H]
DEFAULT_J: float = 1e-3         # Default rotor inertia [kg·m²]
DEFAULT_B: float = 1e-4         # Default viscous friction [N·m·s/rad].
                                 # Set to 0 for lossless model (causes unbounded growth).
                                 # Set to 0.01+ for realistic steady-state behavior.
DEFAULT_PP: int = 4              # Default pole pairs

# ── Resource limits ──────────────────────────────────────────

MAX_TOTAL_STEPS: int = 1_000_000_000   # Maximum simulation steps (DoS guard)
MAX_HISTORY: int = 10000               # DataBus history ring buffer size
MAX_EVENTS: int = 50000                # DataBus event queue cap
MAX_MODULE_ID_LEN: int = 256           # Maximum module ID length
MAX_MODEL_ID_LEN: int = 256            # Maximum model ID length
MAX_CONFIG_SIZE: int = 10 * 1024 * 1024  # Maximum config file size (10 MB)
MAX_DATA_POINTS: int = 500_000         # Maximum plot data points

# ── Thermal parameters ───────────────────────────────────────
# Material constants for thermal modeling

THERMAL_ALPHA_CU: float = 0.00393   # Copper temperature coefficient [1/K]
                                    # Rs(T) = Rs(T_ref) * (1 + alpha_cu * (T - T_ref))
THERMAL_ALPHA_MAG_NDFEB: float = -0.0012  # NdFeB magnet temperature coefficient [1/K]
                                    # flux(T) = flux(T_ref) * (1 + alpha_mag * (T - T_ref))
THERMAL_EPS_C: float = 1e-3         # Minimum thermal capacitance [J/K]
                                    # C_th clamped to this at init to prevent division by zero
