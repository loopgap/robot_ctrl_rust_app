"""Physics constraints and validation for simulation parameters.

Enforces physical relationships between motor, controller, and operating
point parameters BEFORE simulation starts. Prevents non-physical configs
that would produce garbage results.

All constraints are based on fundamental physics:
- Motor: V = IR + L*dI/dt + Ke*omega
- Torque: T = 1.5 * Pp * (flux_pm * iq + (Ld-Lq) * id * iq)
- Mechanical: J * dw/dt = T_em - T_load - B*omega
- Voltage limit: V_bus >= sqrt(Vd^2 + Vq^2)
- Current limit: |I| < I_max (thermal constraint)
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ConstraintViolation:
    """A single constraint violation with severity and fix suggestion."""
    parameter: str
    current_value: float
    limit: float
    message: str
    severity: str  # "error", "warning", "info"
    fix_suggestion: str


class PhysicsValidator:
    """Validates simulation parameters against physical constraints.

    Usage:
        validator = PhysicsValidator()
        violations = validator.validate(config)
        for v in violations:
            print(f"[{v.severity}] {v.message}")
    """

    # Physical constants
    _V_BUS_MIN = 1.0        # Minimum bus voltage [V]
    _V_BUS_MAX = 1000.0     # Maximum bus voltage [V]
    _RS_MIN = 1e-4          # Minimum stator resistance [Ohm]
    _RS_MAX = 100.0         # Maximum stator resistance [Ohm]
    _L_MIN = 1e-7           # Minimum inductance [H]
    _L_MAX = 1.0            # Maximum inductance [H]
    _FLUX_MIN = 1e-5        # Minimum flux linkage [Wb]
    _FLUX_MAX = 10.0        # Maximum flux linkage [Wb]
    _J_MIN = 1e-8           # Minimum inertia [kg*m^2]
    _J_MAX = 100.0          # Maximum inertia [kg*m^2]
    _B_MIN = 0.0            # Minimum friction [N*m*s]
    _B_MAX = 10.0           # Maximum friction [N*m*s]
    _PP_MIN = 1             # Minimum pole pairs
    _PP_MAX = 50            # Maximum pole pairs
    _DT_MIN = 1e-7          # Minimum time step [s]
    _DT_MAX = 1e-1          # Maximum time step [s]
    _SPEED_MAX = 10000.0    # Maximum mechanical speed [rad/s]
    _CURRENT_MAX = 1000.0   # Maximum phase current [A]
    _TORQUE_MAX = 1000.0    # Maximum torque [N*m]

    def validate(self, config: dict) -> list[ConstraintViolation]:
        """Validate all parameters and return list of violations."""
        violations = []

        # Motor parameters
        mp = config.get("motor_params", {})
        violations.extend(self._validate_motor(mp))

        # Controller parameters
        foc = config.get("foc", {})
        violations.extend(self._validate_controller(foc))

        # Speed loop
        spd = config.get("speed_pi", {})
        violations.extend(self._validate_speed_loop(spd))

        # Time parameters
        dt_c = config.get("dt_c", 50e-6)
        dt_s = config.get("dt_s", 1e-3)
        duration = config.get("duration_s", 1.5)
        violations.extend(self._validate_time(dt_c, dt_s, duration))

        # Operating point
        speed_ref = config.get("speed_ref", 100.0)
        load = config.get("load_torque", 0.0)
        bat = config.get("battery", {})
        violations.extend(self._validate_operating_point(
            speed_ref, load, bat, mp
        ))

        # Cross-parameter constraints
        violations.extend(self._validate_cross_constraints(config))

        return violations

    def _validate_motor(self, mp: dict) -> list[ConstraintViolation]:
        """Validate motor parameters."""
        v = []

        Rs = mp.get("Rs", 0.1)
        Ld = mp.get("Ld", 0.5e-3)
        Lq = mp.get("Lq", 1.0e-3)
        flux_pm = mp.get("flux_pm", 0.03)
        J = mp.get("J", 0.001)
        _B = mp.get("B", 0.0001)
        Pp = mp.get("Pp", 4)

        if Rs <= self._RS_MIN:
            v.append(ConstraintViolation(
                "Rs", Rs, self._RS_MIN,
                f"定子电阻 Rs={Rs:.4f}Ω 过小，会导致电流环不稳定",
                "error", f"建议 Rs >= {self._RS_MIN}Ω"
            ))
        if Rs > self._RS_MAX:
            v.append(ConstraintViolation(
                "Rs", Rs, self._RS_MAX,
                f"定子电阻 Rs={Rs:.1f}Ω 过大，铜损过高",
                "warning", f"建议 Rs <= {self._RS_MAX}Ω"
            ))

        if Ld < self._L_MIN:
            v.append(ConstraintViolation(
                "Ld", Ld, self._L_MIN,
                f"d轴电感 Ld={Ld:.2e}H 过小，电流纹波会极大",
                "error", f"建议 Ld >= {self._L_MIN:.0e}H"
            ))
        if Lq < self._L_MIN:
            v.append(ConstraintViolation(
                "Lq", Lq, self._L_MIN,
                f"q轴电感 Lq={Lq:.2e}H 过小，电流纹波会极大",
                "error", f"建议 Lq >= {self._L_MIN:.0e}H"
            ))

        if flux_pm < self._FLUX_MIN:
            v.append(ConstraintViolation(
                "flux_pm", flux_pm, self._FLUX_MIN,
                f"永磁磁链 flux_pm={flux_pm:.4f}Wb 过小，反电动势可忽略",
                "warning", f"建议 flux_pm >= {self._FLUX_MIN:.0e}Wb"
            ))

        if J < self._J_MIN:
            v.append(ConstraintViolation(
                "J", J, self._J_MIN,
                f"转动惯量 J={J:.2e}kg·m² 过小，加速度会极大",
                "warning", f"建议 J >= {self._J_MIN:.0e}kg·m²"
            ))

        if Pp < self._PP_MIN or Pp > self._PP_MAX:
            v.append(ConstraintViolation(
                "Pp", Pp, self._PP_MIN,
                f"极对数 Pp={Pp} 超出合理范围 [{self._PP_MIN}, {self._PP_MAX}]",
                "error", f"建议 Pp 在 {self._PP_MIN}-{self._PP_MAX} 之间"
            ))

        return v

    def _validate_controller(self, foc: dict) -> list[ConstraintViolation]:
        """Validate FOC controller gains."""
        v = []

        for key in ["kp_id", "ki_id", "kp_iq", "ki_iq"]:
            val = foc.get(key, 5.0 if "kp" in key else 500.0)
            if val <= 0:
                v.append(ConstraintViolation(
                    key, val, 0,
                    f"控制器增益 {key}={val} 必须为正数",
                    "error", f"建议 {key} > 0"
                ))
            if val > 1e6:
                v.append(ConstraintViolation(
                    key, val, 1e6,
                    f"控制器增益 {key}={val:.0f} 过大，可能导致数值溢出",
                    "warning", f"建议 {key} <= 1e6"
                ))

        return v

    def _validate_speed_loop(self, spd: dict) -> list[ConstraintViolation]:
        """Validate speed loop PI gains."""
        v = []

        kp = spd.get("kp", 0.05)
        ki = spd.get("ki", 0.5)

        if kp <= 0:
            v.append(ConstraintViolation(
                "spd_kp", kp, 0,
                f"速度环 Kp={kp} 必须为正数",
                "error", "建议 Kp > 0"
            ))
        if ki <= 0:
            v.append(ConstraintViolation(
                "spd_ki", ki, 0,
                f"速度环 Ki={ki} 必须为正数",
                "error", "建议 Ki > 0"
            ))

        return v

    def _validate_time(self, dt_c: float, dt_s: float, duration: float) -> list[ConstraintViolation]:
        """Validate time parameters."""
        v = []

        if dt_c < self._DT_MIN:
            v.append(ConstraintViolation(
                "dt_c", dt_c, self._DT_MIN,
                f"电流环步长 dt_c={dt_c:.2e}s 过小，计算量极大",
                "warning", f"建议 dt_c >= {self._DT_MIN:.0e}s"
            ))
        if dt_c > self._DT_MAX:
            v.append(ConstraintViolation(
                "dt_c", dt_c, self._DT_MAX,
                f"电流环步长 dt_c={dt_c:.4e}s 过大，精度不足",
                "error", f"建议 dt_c <= {self._DT_MAX:.0e}s"
            ))
        if dt_s < dt_c:
            v.append(ConstraintViolation(
                "dt_s", dt_s, dt_c,
                f"速度环步长 dt_s={dt_s:.2e}s 小于电流环步长 dt_c={dt_c:.2e}s",
                "error", "速度环步长必须 >= 电流环步长"
            ))
        if duration < 0.01:
            v.append(ConstraintViolation(
                "duration", duration, 0.01,
                f"仿真时长 {duration:.3f}s 过短，可能无法收敛",
                "warning", "建议 duration >= 0.01s"
            ))

        return v

    def _validate_operating_point(
        self, speed_ref: float, load: float, bat: dict, mp: dict
    ) -> list[ConstraintViolation]:
        """Validate operating point against motor capabilities."""
        v = []

        v_bus = bat.get("voltage", 48.0)
        flux_pm = mp.get("flux_pm", 0.03)
        Pp = mp.get("Pp", 4)
        Rs = mp.get("Rs", 0.1)

        # Back-EMF at target speed
        omega_e = speed_ref * Pp
        back_emf = flux_pm * omega_e

        if back_emf > v_bus * 0.9:
            v.append(ConstraintViolation(
                "speed_ref", speed_ref, v_bus * 0.9 / (flux_pm * Pp),
                f"反电动势 ({back_emf:.1f}V) 接近母线电压 ({v_bus:.1f}V)，"
                f"无法达到目标转速 {speed_ref:.0f} rad/s",
                "error",
                f"降低转速至 {v_bus * 0.9 / (flux_pm * Pp):.0f} rad/s 或提高电压"
            ))

        # Torque capability
        i_max = v_bus / max(Rs, 0.001)
        torque_max = 1.5 * Pp * flux_pm * i_max

        if load > torque_max * 0.8:
            v.append(ConstraintViolation(
                "load_torque", load, torque_max * 0.8,
                f"负载转矩 ({load:.2f}N·m) 接近最大转矩 ({torque_max:.1f}N·m)，"
                f"可能无法驱动",
                "warning",
                f"建议负载 <= {torque_max * 0.8:.1f}N·m"
            ))

        return v

    def _validate_cross_constraints(self, config: dict) -> list[ConstraintViolation]:
        """Validate cross-parameter relationships."""
        v = []

        mp = config.get("motor_params", {})
        foc = config.get("foc", {})
        dt_c = config.get("dt_c", 50e-6)

        # Current loop bandwidth check
        Ld = mp.get("Ld", 0.5e-3)
        Lq = mp.get("Lq", 1.0e-3)
        _Rs = mp.get("Rs", 0.1)
        kp_iq = foc.get("kp_iq", 5.0)

        # Current loop bandwidth: BW = kp / L
        bw_id = kp_iq / max(Ld, 1e-9)
        _bw_iq = kp_iq / max(Lq, 1e-9)
        nyquist = 1.0 / (2.0 * dt_c)

        if bw_id > nyquist * 0.5:
            v.append(ConstraintViolation(
                "kp_iq/Ld", bw_id, nyquist * 0.5,
                f"d轴电流环带宽 ({bw_id:.0f} rad/s) 接近奈奎斯特频率 ({nyquist:.0f} rad/s)，"
                f"可能导致混叠",
                "warning",
                "降低 kp_iq 或增大 dt_c"
            ))

        return v

    def get_summary(self, violations: list[ConstraintViolation]) -> str:
        """Generate a human-readable summary of violations."""
        if not violations:
            return "✓ 所有物理约束检查通过"

        errors = [v for v in violations if v.severity == "error"]
        warnings = [v for v in violations if v.severity == "warning"]

        lines = []
        if errors:
            lines.append(f"❌ {len(errors)} 个错误:")
            for v in errors:
                lines.append(f"  • {v.message}")
                lines.append(f"    → {v.fix_suggestion}")
        if warnings:
            lines.append(f"⚠️ {len(warnings)} 个警告:")
            for v in warnings:
                lines.append(f"  • {v.message}")
                lines.append(f"    → {v.fix_suggestion}")

        return "\n".join(lines)
