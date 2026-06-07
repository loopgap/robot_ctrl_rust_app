use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::consts::PI;

pub const DEFAULT_DT_NS: u64 = 50_000;
pub const DEFAULT_SPEED_LOOP_NS: u64 = 1_000_000;
pub const MAX_TOTAL_STEPS: u64 = 1_000_000_000;
pub const NUMERIC_EPS: f64 = 1.0e-12;
pub const MOTOR_EPS_L: f64 = 1.0e-9;
pub const MOTOR_EPS_J: f64 = 1.0e-12;
pub const DEFAULT_I_MAX: f64 = 10_000.0;

pub fn guard_numeric(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

pub fn guard_positive(value: f64, fallback: f64, min_value: f64) -> f64 {
    guard_numeric(value, fallback).max(min_value)
}

pub fn guard_range(value: f64, low: f64, high: f64, fallback: f64) -> f64 {
    guard_numeric(value, fallback).clamp(low, high)
}

pub fn safe_divide(numerator: f64, denominator: f64, fallback: f64) -> f64 {
    if denominator.abs() < NUMERIC_EPS {
        return fallback;
    }
    guard_numeric(numerator / denominator, fallback)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FidelityLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverType {
    ForwardEuler,
    BackwardEuler,
    RungeKutta,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorModelConfig {
    pub model_id: String,
    pub model_type: String,
    pub fidelity: FidelityLevel,
    pub parameters: HashMap<String, f64>,
    pub depends_on: Vec<String>,
}

impl MotorModelConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.model_id.trim().is_empty() {
            return Err("model_id must not be empty".into());
        }
        if !self
            .model_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(format!("invalid model_id: {}", self.model_id));
        }
        for (key, value) in &self.parameters {
            if !value.is_finite() {
                return Err(format!("parameter {key} must be finite"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmsmParams {
    pub rs: f64,
    pub ld: f64,
    pub lq: f64,
    pub flux_pm: f64,
    pub j: f64,
    pub b: f64,
    pub pole_pairs: u32,
}

impl Default for PmsmParams {
    fn default() -> Self {
        Self {
            rs: 0.1,
            ld: 0.5e-3,
            lq: 1.0e-3,
            flux_pm: 0.03,
            j: 0.001,
            b: 0.0001,
            pole_pairs: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocParams {
    pub kp_id: f64,
    pub ki_id: f64,
    pub kp_iq: f64,
    pub ki_iq: f64,
    pub speed_kp: f64,
    pub speed_ki: f64,
    pub v_bus: f64,
}

impl Default for FocParams {
    fn default() -> Self {
        Self {
            kp_id: 5.0,
            ki_id: 500.0,
            kp_iq: 5.0,
            ki_iq: 500.0,
            speed_kp: 0.05,
            speed_ki: 0.5,
            v_bus: 48.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub name: String,
    pub duration_s: f64,
    pub dt_ns: u64,
    pub speed_loop_ns: u64,
    pub speed_ref: f64,
    pub load_torque: f64,
    pub motor: PmsmParams,
    pub foc: FocParams,
    pub solver: SolverType,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            name: "PMSM FOC Step Response".into(),
            duration_s: 1.5,
            dt_ns: DEFAULT_DT_NS,
            speed_loop_ns: DEFAULT_SPEED_LOOP_NS,
            speed_ref: 100.0,
            load_torque: 0.0,
            motor: PmsmParams::default(),
            foc: FocParams::default(),
            solver: SolverType::ForwardEuler,
        }
    }
}

impl SimulationConfig {
    pub fn total_steps(&self) -> Result<u64, String> {
        self.validate()?;
        Ok((self.duration_s * 1.0e9 / self.dt_ns as f64).ceil() as u64)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("simulation name must not be empty".into());
        }
        if !self.duration_s.is_finite() || self.duration_s <= 0.0 {
            return Err("duration_s must be finite and positive".into());
        }
        if self.dt_ns == 0 {
            return Err("dt_ns must be positive".into());
        }
        if self.speed_loop_ns == 0 {
            return Err("speed_loop_ns must be positive".into());
        }
        if !self.speed_ref.is_finite() || !self.load_torque.is_finite() {
            return Err("scenario numeric inputs must be finite".into());
        }
        let total_steps = (self.duration_s * 1.0e9 / self.dt_ns as f64).ceil() as u64;
        if total_steps > MAX_TOTAL_STEPS {
            return Err(format!(
                "total steps {total_steps} exceeds maximum {MAX_TOTAL_STEPS}"
            ));
        }
        for (name, value) in [
            ("rs", self.motor.rs),
            ("ld", self.motor.ld),
            ("lq", self.motor.lq),
            ("flux_pm", self.motor.flux_pm),
            ("j", self.motor.j),
            ("b", self.motor.b),
            ("kp_id", self.foc.kp_id),
            ("ki_id", self.foc.ki_id),
            ("kp_iq", self.foc.kp_iq),
            ("ki_iq", self.foc.ki_iq),
            ("speed_kp", self.foc.speed_kp),
            ("speed_ki", self.foc.speed_ki),
            ("v_bus", self.foc.v_bus),
        ] {
            if !value.is_finite() {
                return Err(format!("{name} must be finite"));
            }
        }
        if self.motor.ld <= 0.0 || self.motor.lq <= 0.0 || self.motor.j <= 0.0 {
            return Err("motor inductance and inertia must be positive".into());
        }
        if self.foc.v_bus <= 0.0 {
            return Err("v_bus must be positive".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SimulationSample {
    pub time_s: f64,
    pub speed_ref: f64,
    pub omega_m: f64,
    pub torque_nm: f64,
    pub id_a: f64,
    pub iq_a: f64,
    pub phase_current_a: f64,
    pub temperature_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub final_speed: f64,
    pub speed_error_pct: f64,
    pub peak_torque: f64,
    pub peak_current: f64,
    pub final_iq: f64,
    pub max_temperature_c: f64,
    pub settled: bool,
    pub cancelled: bool,
    pub steps_executed: u64,
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            final_speed: 0.0,
            speed_error_pct: 100.0,
            peak_torque: 0.0,
            peak_current: 0.0,
            final_iq: 0.0,
            max_temperature_c: 25.0,
            settled: false,
            cancelled: false,
            steps_executed: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRunResult {
    pub config: SimulationConfig,
    pub samples: Vec<SimulationSample>,
    pub metrics: SimulationMetrics,
    pub energy_audits: Vec<EnergyAudit>,
}

impl SimulationRunResult {
    pub fn to_csv(&self) -> String {
        let mut out = String::from(
            "time_s,speed_ref,omega_m,torque_nm,id_a,iq_a,phase_current_a,temperature_c\n",
        );
        for s in &self.samples {
            out.push_str(&format!(
                "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}\n",
                s.time_s,
                s.speed_ref,
                s.omega_m,
                s.torque_nm,
                s.id_a,
                s.iq_a,
                s.phase_current_a,
                s.temperature_c
            ));
        }
        out
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnergyAudit {
    pub power_input_j: f64,
    pub mechanical_output_j: f64,
    pub thermal_loss_j: f64,
    pub stored_energy_j: f64,
    pub imbalance_j: f64,
}

impl EnergyAudit {
    pub fn imbalance_pct(&self) -> f64 {
        let total = self.power_input_j.abs().max(NUMERIC_EPS);
        (self.imbalance_j.abs() / total) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub source: String,
    pub signal_type: String,
    pub timestamp_ns: u64,
    pub value: f64,
    pub unit: String,
    pub quality: f64,
}

impl Signal {
    pub fn new(source: &str, signal_type: &str, timestamp_ns: u64, value: f64, unit: &str) -> Self {
        let finite = value.is_finite();
        Self {
            source: source.into(),
            signal_type: signal_type.into(),
            timestamp_ns,
            value: if finite { value } else { 0.0 },
            unit: unit.into(),
            quality: if finite { 1.0 } else { 0.0 },
        }
    }
}

#[derive(Debug, Default)]
pub struct DataBus {
    registered_modules: HashSet<String>,
    topic_acls: HashMap<String, HashSet<String>>,
    latest: HashMap<String, Signal>,
}

impl DataBus {
    pub fn register_module(&mut self, module_id: &str) -> Result<(), String> {
        validate_module_id(module_id)?;
        self.registered_modules.insert(module_id.into());
        Ok(())
    }

    pub fn allow_topic(&mut self, topic: &str, module_id: &str) -> Result<(), String> {
        validate_module_id(module_id)?;
        self.topic_acls
            .entry(topic.into())
            .or_default()
            .insert(module_id.into());
        Ok(())
    }

    pub fn publish(&mut self, topic: &str, module_id: &str, signal: Signal) -> Result<(), String> {
        self.authorize(topic, module_id)?;
        self.latest.insert(topic.into(), signal);
        Ok(())
    }

    pub fn read_latest(&self, topic: &str) -> Option<&Signal> {
        self.latest.get(topic)
    }

    fn authorize(&self, topic: &str, module_id: &str) -> Result<(), String> {
        if !self.registered_modules.contains(module_id) {
            return Err(format!("unregistered module {module_id}"));
        }
        if let Some(allowed) = self.topic_acls.get(topic) {
            if !allowed.contains(module_id) {
                return Err(format!("module {module_id} is not authorized for {topic}"));
            }
        }
        Ok(())
    }
}

fn validate_module_id(module_id: &str) -> Result<(), String> {
    if module_id.len() > 96 || !module_id.contains("://") {
        return Err("module id must use scheme://name and stay under 96 bytes".into());
    }
    if !module_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '_' | '-' | '.'))
    {
        return Err(format!("invalid module id: {module_id}"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockMode {
    Offline,
    Realtime,
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalClock {
    pub mode: ClockMode,
    pub sim_time_ns: u64,
    pub diverged: bool,
}

impl Default for GlobalClock {
    fn default() -> Self {
        Self {
            mode: ClockMode::Offline,
            sim_time_ns: 0,
            diverged: false,
        }
    }
}

impl GlobalClock {
    pub fn advance(&mut self, dt_ns: u64) {
        self.sim_time_ns = self.sim_time_ns.saturating_add(dt_ns);
    }

    pub fn sim_time_s(&self) -> f64 {
        self.sim_time_ns as f64 * 1.0e-9
    }

    pub fn mark_diverged(&mut self) {
        self.diverged = true;
    }

    pub fn reset(&mut self) {
        self.sim_time_ns = 0;
        self.diverged = false;
    }
}

pub struct ModelRegistry {
    model_ids: HashSet<String>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            model_ids: HashSet::new(),
        }
    }

    pub fn register(&mut self, model: &MotorModelConfig) -> Result<(), String> {
        model.validate()?;
        if !self.model_ids.insert(model.model_id.clone()) {
            return Err(format!("duplicate model id {}", model.model_id));
        }
        Ok(())
    }
}

pub struct FaultEvent {
    pub at_time_s: f64,
    pub magnitude: f64,
}

#[derive(Default)]
pub struct Orchestrator {
    pub clock: GlobalClock,
    pub bus: DataBus,
    pub registry: ModelRegistry,
    faults: VecDeque<FaultEvent>,
}

impl Orchestrator {
    pub fn schedule_fault(&mut self, fault: FaultEvent) -> Result<(), String> {
        if !fault.at_time_s.is_finite() || fault.at_time_s < 0.0 || !fault.magnitude.is_finite() {
            return Err("fault time and magnitude must be finite".into());
        }
        let pos = self
            .faults
            .iter()
            .position(|existing| existing.at_time_s > fault.at_time_s)
            .unwrap_or(self.faults.len());
        self.faults.insert(pos, fault);
        Ok(())
    }

    pub fn due_fault_magnitude(&mut self) -> f64 {
        let mut total = 0.0;
        while let Some(fault) = self.faults.front() {
            if fault.at_time_s > self.clock.sim_time_s() {
                break;
            }
            total += self.faults.pop_front().map(|f| f.magnitude).unwrap_or(0.0);
        }
        total
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn clarke_transform(ia: f64, ib: f64, _ic: f64) -> (f64, f64) {
    (ia, (ia + 2.0 * ib) / 3.0_f64.sqrt())
}

pub fn park_transform(alpha: f64, beta: f64, theta: f64) -> (f64, f64) {
    let (sin_t, cos_t) = theta.sin_cos();
    (alpha * cos_t + beta * sin_t, -alpha * sin_t + beta * cos_t)
}

pub fn inverse_park(vd: f64, vq: f64, theta: f64) -> (f64, f64) {
    let (sin_t, cos_t) = theta.sin_cos();
    (vd * cos_t - vq * sin_t, vd * sin_t + vq * cos_t)
}

pub fn svpwm(v_alpha: f64, v_beta: f64, v_bus: f64) -> (f64, f64, f64) {
    let v_bus = guard_positive(v_bus, 48.0, 0.1);
    let va = v_alpha;
    let vb = -0.5 * v_alpha + 3.0_f64.sqrt() * 0.5 * v_beta;
    let vc = -0.5 * v_alpha - 3.0_f64.sqrt() * 0.5 * v_beta;
    let max_abs = va.abs().max(vb.abs()).max(vc.abs());
    let scale = if max_abs > v_bus * 0.5 {
        (v_bus * 0.5) / max_abs
    } else {
        1.0
    };
    (
        (0.5 + va * scale / v_bus).clamp(0.0, 1.0),
        (0.5 + vb * scale / v_bus).clamp(0.0, 1.0),
        (0.5 + vc * scale / v_bus).clamp(0.0, 1.0),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct PIController {
    pub kp: f64,
    pub ki: f64,
    pub ts: f64,
    pub out_min: f64,
    pub out_max: f64,
    integral: f64,
    prev_output: f64,
}

impl PIController {
    pub fn new(kp: f64, ki: f64, ts: f64, out_min: f64, out_max: f64) -> Self {
        let (out_min, out_max) = if out_min <= out_max {
            (out_min, out_max)
        } else {
            (out_max, out_min)
        };
        Self {
            kp: guard_numeric(kp, 1.0),
            ki: guard_numeric(ki, 0.0),
            ts: guard_positive(ts, 1.0e-3, NUMERIC_EPS),
            out_min: guard_numeric(out_min, -1.0e6),
            out_max: guard_numeric(out_max, 1.0e6),
            integral: 0.0,
            prev_output: 0.0,
        }
    }

    pub fn update(&mut self, setpoint: f64, measurement: f64) -> f64 {
        if !setpoint.is_finite() || !measurement.is_finite() {
            return self.prev_output;
        }
        let error = setpoint - measurement;
        let p_term = self.kp * error;
        let i_term = self.integral + self.ki * self.ts * error;
        let output = (p_term + i_term).clamp(self.out_min, self.out_max);
        self.integral = (output - p_term).clamp(self.out_min, self.out_max);
        self.prev_output = guard_numeric(output, self.prev_output);
        self.prev_output
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_output = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct PmsmDqModel {
    pub params: PmsmParams,
    pub dt: f64,
    pub id: f64,
    pub iq: f64,
    pub omega_m: f64,
    pub theta_e: f64,
    pub torque: f64,
    pub ia: f64,
    pub ib: f64,
    pub ic: f64,
}

impl PmsmDqModel {
    pub fn new(params: PmsmParams, dt_ns: u64) -> Self {
        let mut params = params;
        params.rs = guard_numeric(params.rs, 0.1);
        params.ld = guard_positive(params.ld, 0.5e-3, MOTOR_EPS_L);
        params.lq = guard_positive(params.lq, 1.0e-3, MOTOR_EPS_L);
        params.flux_pm = guard_numeric(params.flux_pm, 0.03);
        params.j = guard_positive(params.j, 0.001, MOTOR_EPS_J);
        params.b = guard_numeric(params.b, 0.0);
        params.pole_pairs = params.pole_pairs.max(1);
        Self {
            params,
            dt: (dt_ns as f64 * 1.0e-9).max(NUMERIC_EPS),
            id: 0.0,
            iq: 0.0,
            omega_m: 0.0,
            theta_e: 0.0,
            torque: 0.0,
            ia: 0.0,
            ib: 0.0,
            ic: 0.0,
        }
    }

    pub fn omega_e(&self) -> f64 {
        self.params.pole_pairs as f64 * self.omega_m
    }

    pub fn torque_em(&self) -> f64 {
        let id = self.id.clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        let iq = self.iq.clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        1.5 * self.params.pole_pairs as f64
            * (self.params.flux_pm * iq + (self.params.ld - self.params.lq) * id * iq)
    }

    pub fn step(&mut self, vd: f64, vq: f64, tl: f64, dt: Option<f64>) {
        let vd = guard_numeric(vd, 0.0);
        let vq = guard_numeric(vq, 0.0);
        let tl = guard_numeric(tl, 0.0);
        let dt = dt
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(self.dt)
            .max(NUMERIC_EPS);
        self.id = guard_numeric(self.id, 0.0);
        self.iq = guard_numeric(self.iq, 0.0);
        self.omega_m = guard_numeric(self.omega_m, 0.0);
        let we = self.omega_e();
        let did = (vd - self.params.rs * self.id + we * self.params.lq * self.iq) / self.params.ld;
        let diq =
            (vq - self.params.rs * self.iq - we * (self.params.ld * self.id + self.params.flux_pm))
                / self.params.lq;
        self.id += guard_numeric(did, 0.0) * dt;
        self.iq += guard_numeric(diq, 0.0) * dt;
        self.id = self.id.clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        self.iq = self.iq.clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        self.torque = guard_numeric(self.torque_em(), 0.0);
        let dw = (self.torque - tl - self.params.b * self.omega_m) / self.params.j;
        self.omega_m += guard_numeric(dw, 0.0) * dt;
        self.omega_m = guard_numeric(self.omega_m, 0.0);
        self.theta_e =
            (self.theta_e + self.params.pole_pairs as f64 * self.omega_m * dt).rem_euclid(2.0 * PI);
    }

    pub fn update_abc_currents(&mut self) -> (f64, f64, f64) {
        let (sin_t, cos_t) = self.theta_e.sin_cos();
        let alpha = self.id * cos_t - self.iq * sin_t;
        let beta = self.id * sin_t + self.iq * cos_t;
        self.ia = alpha;
        self.ib = -0.5 * alpha + 3.0_f64.sqrt() * 0.5 * beta;
        self.ic = -0.5 * alpha - 3.0_f64.sqrt() * 0.5 * beta;
        (self.ia, self.ib, self.ic)
    }

    pub fn step_abc(&mut self, va: f64, vb: f64, vc: f64, tl: f64, dt: Option<f64>) {
        if !(va.is_finite() && vb.is_finite() && vc.is_finite()) {
            return;
        }
        let alpha = va;
        let beta = (va + 2.0 * vb) / 3.0_f64.sqrt();
        let (vd, vq) = park_transform(alpha, beta, self.theta_e);
        self.step(vd, vq, tl, dt);
    }

    pub fn reset(&mut self) {
        self.id = 0.0;
        self.iq = 0.0;
        self.omega_m = 0.0;
        self.theta_e = 0.0;
        self.torque = 0.0;
        self.ia = 0.0;
        self.ib = 0.0;
        self.ic = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct PmsmAdvanced {
    pub base: PmsmDqModel,
    pub saturation_gain: f64,
    pub iron_loss_coeff: f64,
    pub temperature_c: f64,
}

impl PmsmAdvanced {
    pub fn new(params: PmsmParams, dt_ns: u64) -> Self {
        Self {
            base: PmsmDqModel::new(params, dt_ns),
            saturation_gain: 0.02,
            iron_loss_coeff: 0.0001,
            temperature_c: 25.0,
        }
    }

    pub fn step(&mut self, vd: f64, vq: f64, tl: f64, dt: f64) {
        let current_mag = (self.base.id * self.base.id + self.base.iq * self.base.iq).sqrt();
        let derate = 1.0 / (1.0 + self.saturation_gain * current_mag);
        let original_flux = self.base.params.flux_pm;
        self.base.params.flux_pm = original_flux * derate;
        self.base.step(vd, vq, tl, Some(dt));
        self.base.params.flux_pm = original_flux;
        let copper = self.base.params.rs * current_mag * current_mag;
        let iron = self.iron_loss_coeff * self.base.omega_e().abs();
        self.temperature_c += (copper + iron - 0.08 * (self.temperature_c - 25.0)) * dt;
        self.temperature_c = guard_numeric(self.temperature_c, 25.0).clamp(-40.0, 220.0);
    }
}

pub struct AverageInverter {
    pub v_bus_nominal: f64,
}

impl AverageInverter {
    pub fn new(v_bus_nominal: f64) -> Self {
        Self {
            v_bus_nominal: guard_positive(v_bus_nominal, 48.0, 0.1),
        }
    }

    pub fn step(&self, duty_a: f64, duty_b: f64, duty_c: f64, v_bus: f64) -> (f64, f64, f64) {
        let bus = guard_positive(v_bus, self.v_bus_nominal, 0.1);
        (
            (guard_range(duty_a, 0.0, 1.0, 0.5) - 0.5) * bus,
            (guard_range(duty_b, 0.0, 1.0, 0.5) - 0.5) * bus,
            (guard_range(duty_c, 0.0, 1.0, 0.5) - 0.5) * bus,
        )
    }
}

pub struct RintBattery {
    pub voltage_nominal: f64,
    pub resistance: f64,
}

impl RintBattery {
    pub fn voltage(&self, current_a: f64) -> f64 {
        (guard_positive(self.voltage_nominal, 48.0, 0.1)
            - guard_numeric(current_a, 0.0).abs() * guard_positive(self.resistance, 0.05, 0.0))
        .max(1.0)
    }
}

pub struct CurrentSensor {
    pub bias: f64,
    pub limit: f64,
}

impl CurrentSensor {
    pub fn read_abc(&self, ia: f64, ib: f64, ic: f64) -> (f64, f64, f64) {
        let limit = self.limit.abs().max(1.0);
        (
            (guard_numeric(ia, 0.0) + self.bias).clamp(-limit, limit),
            (guard_numeric(ib, 0.0) + self.bias).clamp(-limit, limit),
            (guard_numeric(ic, 0.0) + self.bias).clamp(-limit, limit),
        )
    }
}

pub struct Encoder {
    pub angle_bias: f64,
    pub speed_bias: f64,
}

impl Encoder {
    pub fn read_angle(&self, theta: f64) -> f64 {
        (guard_numeric(theta, 0.0) + self.angle_bias).rem_euclid(2.0 * PI)
    }

    pub fn read_speed(&self, speed: f64) -> f64 {
        guard_numeric(speed, 0.0) + self.speed_bias
    }
}

pub struct FocController {
    pi_id: PIController,
    pi_iq: PIController,
    pub v_bus: f64,
    pub vd_ref: f64,
    pub vq_ref: f64,
    pub duty_a: f64,
    pub duty_b: f64,
    pub duty_c: f64,
}

impl FocController {
    pub fn new(params: &FocParams, ts: f64) -> Self {
        let v_bus = guard_positive(params.v_bus, 48.0, 0.1);
        Self {
            pi_id: PIController::new(params.kp_id, params.ki_id, ts, -v_bus, v_bus),
            pi_iq: PIController::new(params.kp_iq, params.ki_iq, ts, -v_bus, v_bus),
            v_bus,
            vd_ref: 0.0,
            vq_ref: 0.0,
            duty_a: 0.5,
            duty_b: 0.5,
            duty_c: 0.5,
        }
    }

    pub fn update(
        &mut self,
        ia: f64,
        ib: f64,
        ic: f64,
        theta_e: f64,
        id_ref: f64,
        iq_ref: f64,
    ) -> (f64, f64, f64) {
        if !(ia.is_finite()
            && ib.is_finite()
            && ic.is_finite()
            && theta_e.is_finite()
            && id_ref.is_finite()
            && iq_ref.is_finite())
        {
            return (self.duty_a, self.duty_b, self.duty_c);
        }
        let (alpha, beta) = clarke_transform(ia, ib, ic);
        let (id_meas, iq_meas) = park_transform(alpha, beta, theta_e);
        self.vd_ref = self.pi_id.update(id_ref, id_meas);
        self.vq_ref = self.pi_iq.update(iq_ref, iq_meas);
        let (v_alpha, v_beta) = inverse_park(self.vd_ref, self.vq_ref, theta_e);
        let duties = svpwm(v_alpha, v_beta, self.v_bus);
        self.duty_a = duties.0;
        self.duty_b = duties.1;
        self.duty_c = duties.2;
        duties
    }
}

pub struct SpeedController {
    pi: PIController,
}

impl SpeedController {
    pub fn new(kp: f64, ki: f64, ts: f64) -> Self {
        Self {
            pi: PIController::new(kp, ki, ts, -200.0, 200.0),
        }
    }

    pub fn update(&mut self, speed_ref: f64, speed_meas: f64) -> f64 {
        self.pi.update(speed_ref, speed_meas)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HallState {
    H1H2,
    H1H3,
    H2H3,
    H2H1,
    H3H1,
    H3H2,
}

#[derive(Debug, Clone)]
pub struct BldcModel {
    pub rs: f64,
    pub ls: f64,
    pub ke: f64,
    pub kt: f64,
    pub j: f64,
    pub b: f64,
    pub pole_pairs: u32,
    pub ia: f64,
    pub ib: f64,
    pub ic: f64,
    pub omega_m: f64,
    pub theta_e: f64,
    pub torque: f64,
}

impl BldcModel {
    pub fn new() -> Self {
        Self {
            rs: 0.1,
            ls: 1.0e-3,
            ke: 0.01,
            kt: 0.01,
            j: 1.0e-4,
            b: 0.0001,
            pole_pairs: 1,
            ia: 0.0,
            ib: 0.0,
            ic: 0.0,
            omega_m: 0.0,
            theta_e: 0.0,
            torque: 0.0,
        }
    }

    pub fn hall_state(&self) -> HallState {
        let deg = self.theta_e.rem_euclid(2.0 * PI).to_degrees();
        if deg < 60.0 {
            HallState::H1H2
        } else if deg < 120.0 {
            HallState::H1H3
        } else if deg < 180.0 {
            HallState::H2H3
        } else if deg < 240.0 {
            HallState::H2H1
        } else if deg < 300.0 {
            HallState::H3H1
        } else {
            HallState::H3H2
        }
    }

    pub fn step(&mut self, v_bus: f64, load: f64, dt: f64) {
        let v_bus = guard_numeric(v_bus, 0.0);
        let dt = guard_positive(dt, 50.0e-6, NUMERIC_EPS);
        let (ea, eb, ec) = trapezoidal_emf(self.theta_e);
        let (pa, pb, pc) = match self.hall_state() {
            HallState::H1H2 => (1.0, -1.0, 0.0),
            HallState::H1H3 => (1.0, 0.0, -1.0),
            HallState::H2H3 => (0.0, 1.0, -1.0),
            HallState::H2H1 => (-1.0, 1.0, 0.0),
            HallState::H3H1 => (-1.0, 0.0, 1.0),
            HallState::H3H2 => (0.0, -1.0, 1.0),
        };
        let omega_e = self.pole_pairs as f64 * self.omega_m;
        let dia = (pa * v_bus * 0.5 - self.ke * omega_e * ea - self.rs * self.ia) / self.ls;
        let dib = (pb * v_bus * 0.5 - self.ke * omega_e * eb - self.rs * self.ib) / self.ls;
        let dic = (pc * v_bus * 0.5 - self.ke * omega_e * ec - self.rs * self.ic) / self.ls;
        self.ia = (self.ia + guard_numeric(dia, 0.0) * dt).clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        self.ib = (self.ib + guard_numeric(dib, 0.0) * dt).clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        self.ic = (self.ic + guard_numeric(dic, 0.0) * dt).clamp(-DEFAULT_I_MAX, DEFAULT_I_MAX);
        self.torque = guard_numeric(self.kt * (self.ia * ea + self.ib * eb + self.ic * ec), 0.0);
        let dw = (self.torque - guard_numeric(load, 0.0) - self.b * self.omega_m) / self.j;
        self.omega_m = guard_numeric(self.omega_m + guard_numeric(dw, 0.0) * dt, 0.0);
        self.theta_e = (self.theta_e + omega_e * dt).rem_euclid(2.0 * PI);
    }
}

impl Default for BldcModel {
    fn default() -> Self {
        Self::new()
    }
}

fn trapezoidal_emf(theta: f64) -> (f64, f64, f64) {
    fn shape(deg: f64) -> f64 {
        let a = deg.rem_euclid(360.0);
        if a < 60.0 {
            a / 60.0
        } else if a < 180.0 {
            1.0
        } else if a < 240.0 {
            1.0 - (a - 180.0) / 60.0
        } else if a < 300.0 {
            -1.0
        } else {
            -1.0 + (a - 300.0) / 60.0
        }
    }
    let deg = theta.to_degrees();
    (shape(deg), shape(deg - 120.0), shape(deg - 240.0))
}

#[derive(Debug, Clone)]
pub struct ImDqModel {
    pub rs: f64,
    pub rr: f64,
    pub ls: f64,
    pub lr: f64,
    pub lm: f64,
    pub j: f64,
    pub b: f64,
    pub ids: f64,
    pub iqs: f64,
    pub psi_rd: f64,
    pub psi_rq: f64,
    pub omega_m: f64,
    pub torque: f64,
}

impl ImDqModel {
    pub fn new() -> Self {
        Self {
            rs: 0.5,
            rr: 0.5,
            ls: 0.01,
            lr: 0.01,
            lm: 0.009,
            j: 0.01,
            b: 0.001,
            ids: 0.0,
            iqs: 0.0,
            psi_rd: 0.0,
            psi_rq: 0.0,
            omega_m: 0.0,
            torque: 0.0,
        }
    }

    pub fn step(&mut self, vsd: f64, vsq: f64, omega_e: f64, load: f64, dt: f64) {
        let dt = guard_positive(dt, 50.0e-6, NUMERIC_EPS);
        let sigma =
            (1.0 - self.lm * self.lm / (self.ls * self.lr).max(MOTOR_EPS_L)).clamp(0.05, 1.0);
        let sigma_ls = (sigma * self.ls).max(MOTOR_EPS_L);
        let tr = self.lr / self.rr.max(NUMERIC_EPS);
        let slip = guard_numeric(omega_e, 0.0) - self.omega_m;
        let dids = (guard_numeric(vsd, 0.0) - self.rs * self.ids + omega_e * sigma_ls * self.iqs)
            / sigma_ls;
        let diqs = (guard_numeric(vsq, 0.0) - self.rs * self.iqs - omega_e * sigma_ls * self.ids)
            / sigma_ls;
        self.ids += guard_numeric(dids, 0.0) * dt;
        self.iqs += guard_numeric(diqs, 0.0) * dt;
        self.psi_rd += (self.lm * self.ids - self.psi_rd) / tr * dt + slip * self.psi_rq * dt;
        self.psi_rq += (self.lm * self.iqs - self.psi_rq) / tr * dt - slip * self.psi_rd * dt;
        self.torque = 1.5
            * (self.lm / self.lr.max(MOTOR_EPS_L))
            * (self.psi_rd * self.iqs - self.psi_rq * self.ids);
        let dw = (self.torque - guard_numeric(load, 0.0) - self.b * self.omega_m)
            / self.j.max(MOTOR_EPS_J);
        self.omega_m = guard_numeric(self.omega_m + guard_numeric(dw, 0.0) * dt, 0.0);
    }
}

impl Default for ImDqModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ThermalNode {
    pub temperature_c: f64,
    pub ambient_c: f64,
    pub thermal_resistance: f64,
    pub thermal_capacity: f64,
}

impl ThermalNode {
    pub fn step(&mut self, power_loss_w: f64, dt: f64) -> f64 {
        let cooling =
            (self.temperature_c - self.ambient_c) / self.thermal_resistance.max(NUMERIC_EPS);
        let dtemp =
            (guard_numeric(power_loss_w, 0.0) - cooling) / self.thermal_capacity.max(NUMERIC_EPS);
        self.temperature_c = guard_numeric(
            self.temperature_c + dtemp * dt.max(NUMERIC_EPS),
            self.ambient_c,
        )
        .clamp(-273.15, 500.0);
        self.temperature_c
    }
}

pub struct MpcController {
    pub horizon: usize,
    pub control_min: f64,
    pub control_max: f64,
}

impl MpcController {
    pub fn solve<F>(&self, x0: f64, x_ref: f64, model: F) -> (f64, f64)
    where
        F: Fn(f64, f64) -> f64,
    {
        let steps = 21usize;
        let mut best_u = 0.0;
        let mut best_cost = f64::INFINITY;
        for i in 0..steps {
            let t = i as f64 / (steps - 1) as f64;
            let u = self.control_min + (self.control_max - self.control_min) * t;
            let mut x = guard_numeric(x0, 0.0);
            let mut cost = 0.0;
            for _ in 0..self.horizon.max(1) {
                x = model(x, u);
                let error = guard_numeric(x_ref, 0.0) - x;
                cost += error * error + 0.01 * u * u;
            }
            if cost < best_cost {
                best_cost = cost;
                best_u = u;
            }
        }
        (best_u, best_cost)
    }
}

pub struct EkfEstimator {
    pub x: [f64; 4],
    pub p_diag: [f64; 4],
    pub process_noise: f64,
    pub measurement_noise: f64,
}

impl Default for EkfEstimator {
    fn default() -> Self {
        Self {
            x: [0.0; 4],
            p_diag: [0.1; 4],
            process_noise: 0.01,
            measurement_noise: 0.1,
        }
    }
}

impl EkfEstimator {
    pub fn estimate(
        &mut self,
        vd: f64,
        vq: f64,
        ia: f64,
        ib: f64,
        ic: f64,
        omega_encoder: f64,
    ) -> [f64; 4] {
        let (alpha, beta) = clarke_transform(ia, ib, ic);
        let (id_meas, iq_meas) = park_transform(alpha, beta, self.x[3]);
        let z = [
            id_meas,
            iq_meas,
            guard_numeric(omega_encoder, 0.0),
            self.x[3],
        ];
        let u_scale = (guard_numeric(vd, 0.0).abs() + guard_numeric(vq, 0.0).abs()) * 0.0001;
        self.x[0] += u_scale - 0.05 * self.x[0];
        self.x[1] += u_scale - 0.05 * self.x[1];
        self.x[2] = guard_numeric(omega_encoder, self.x[2]);
        for (i, z_i) in z.iter().enumerate() {
            self.p_diag[i] += self.process_noise;
            let k = self.p_diag[i] / (self.p_diag[i] + self.measurement_noise);
            self.x[i] += k * (guard_numeric(*z_i, self.x[i]) - self.x[i]);
            self.p_diag[i] *= 1.0 - k;
            self.x[i] = guard_numeric(self.x[i], 0.0);
        }
        self.x[3] = self.x[3].rem_euclid(2.0 * PI);
        self.x
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanPoint {
    pub param_name: String,
    pub param_value: f64,
    pub final_speed: f64,
    pub speed_error_pct: f64,
    pub peak_torque: f64,
    pub settled: bool,
}

pub fn run_pmsm_foc(config: &SimulationConfig) -> Result<SimulationRunResult, String> {
    run_pmsm_foc_with_hooks(config, || false, |_| {})
}

pub fn run_pmsm_foc_with_hooks<C, P>(
    config: &SimulationConfig,
    mut should_cancel: C,
    mut on_progress: P,
) -> Result<SimulationRunResult, String>
where
    C: FnMut() -> bool,
    P: FnMut(f32),
{
    config.validate()?;
    let total_steps = config.total_steps()?;
    let dt = config.dt_ns as f64 * 1.0e-9;
    let speed_ratio = (config.speed_loop_ns / config.dt_ns).max(1);
    let sample_stride = (total_steps / 800).max(1);

    let battery = RintBattery {
        voltage_nominal: config.foc.v_bus,
        resistance: 0.05,
    };
    let inverter = AverageInverter::new(config.foc.v_bus);
    let mut motor = PmsmDqModel::new(config.motor.clone(), config.dt_ns);
    let current_sensor = CurrentSensor {
        bias: 0.01,
        limit: DEFAULT_I_MAX,
    };
    let encoder = Encoder {
        angle_bias: 0.0,
        speed_bias: 0.0,
    };
    let mut foc = FocController::new(&config.foc, dt);
    let mut speed = SpeedController::new(
        config.foc.speed_kp,
        config.foc.speed_ki,
        config.speed_loop_ns as f64 * 1.0e-9,
    );
    let mut thermal = ThermalNode {
        temperature_c: 25.0,
        ambient_c: 25.0,
        thermal_resistance: 1.2,
        thermal_capacity: 15.0,
    };
    let mut orchestrator = Orchestrator::default();
    orchestrator.bus.register_module("sim://pmsm").ok();
    orchestrator
        .bus
        .allow_topic("motor/speed", "sim://pmsm")
        .ok();

    let mut samples = Vec::new();
    let mut iq_ref = 0.0;
    let mut peak_torque: f64 = 0.0;
    let mut peak_current: f64 = 0.0;
    let mut max_temperature_c: f64 = thermal.temperature_c;
    let mut cancelled = false;
    let mut steps_executed = 0u64;
    let mut energy_audits = Vec::new();

    for step in 0..total_steps {
        if should_cancel() {
            cancelled = true;
            break;
        }
        if step % speed_ratio == 0 {
            let speed_meas = encoder.read_speed(motor.omega_m);
            iq_ref = speed.update(config.speed_ref, speed_meas);
        }
        let (ia_m, ib_m, ic_m) = current_sensor.read_abc(motor.ia, motor.ib, motor.ic);
        let theta_m = encoder.read_angle(motor.theta_e);
        let (da, db, dc) = foc.update(ia_m, ib_m, ic_m, theta_m, 0.0, iq_ref);
        let bus_voltage = battery.voltage((ia_m.abs() + ib_m.abs() + ic_m.abs()) / 3.0);
        let (va, vb, vc) = inverter.step(da, db, dc, bus_voltage);
        motor.step_abc(va, vb, vc, config.load_torque, Some(dt));
        motor.update_abc_currents();
        let current_mag = (motor.ia * motor.ia + motor.ib * motor.ib + motor.ic * motor.ic).sqrt();
        let temp = thermal.step(motor.params.rs * current_mag * current_mag, dt);
        max_temperature_c = max_temperature_c.max(temp);
        peak_torque = peak_torque.max(motor.torque.abs());
        peak_current = peak_current.max(current_mag.abs());
        orchestrator.clock.advance(config.dt_ns);
        orchestrator
            .bus
            .publish(
                "motor/speed",
                "sim://pmsm",
                Signal::new(
                    "sim://pmsm",
                    "omega_m",
                    orchestrator.clock.sim_time_ns,
                    motor.omega_m,
                    "rad/s",
                ),
            )
            .ok();
        if step % sample_stride == 0 || step + 1 == total_steps {
            samples.push(SimulationSample {
                time_s: step as f64 * dt,
                speed_ref: config.speed_ref,
                omega_m: motor.omega_m,
                torque_nm: motor.torque,
                id_a: motor.id,
                iq_a: motor.iq,
                phase_current_a: current_mag,
                temperature_c: temp,
            });
        }
        if step % 100 == 0 {
            on_progress(step as f32 / total_steps.max(1) as f32);
        }
        if step % 1000 == 0 {
            let electrical = bus_voltage * (ia_m.abs() + ib_m.abs() + ic_m.abs()) / 3.0 * dt;
            let mechanical = motor.torque.abs() * motor.omega_m.abs() * dt;
            let thermal_loss = motor.params.rs * current_mag * current_mag * dt;
            let audit = EnergyAudit {
                power_input_j: electrical,
                mechanical_output_j: mechanical,
                thermal_loss_j: thermal_loss,
                stored_energy_j: 0.5 * motor.params.j * motor.omega_m * motor.omega_m,
                imbalance_j: electrical - mechanical - thermal_loss,
            };
            if audit.imbalance_pct().is_finite() {
                energy_audits.push(audit);
            }
        }
        steps_executed = step + 1;
    }

    on_progress(1.0);
    let final_speed = motor.omega_m;
    let speed_error_pct =
        ((final_speed - config.speed_ref).abs() / config.speed_ref.abs().max(1.0)) * 100.0;
    let metrics = SimulationMetrics {
        final_speed,
        speed_error_pct,
        peak_torque,
        peak_current,
        final_iq: motor.iq,
        max_temperature_c,
        settled: speed_error_pct < 5.0,
        cancelled,
        steps_executed,
    };
    Ok(SimulationRunResult {
        config: config.clone(),
        samples,
        metrics,
        energy_audits,
    })
}

pub fn run_parameter_scan(
    base: &SimulationConfig,
    param_name: &str,
    values: &[f64],
) -> Result<Vec<ScanPoint>, String> {
    if values.is_empty() {
        return Err("scan values must not be empty".into());
    }
    let mut points = Vec::with_capacity(values.len());
    for &value in values {
        if !value.is_finite() {
            return Err("scan value must be finite".into());
        }
        let mut config = base.clone();
        match param_name {
            "speed" => config.speed_ref = value,
            "load" => config.load_torque = value,
            "kp_id" => config.foc.kp_id = value,
            "ki_id" => config.foc.ki_id = value,
            "kp_iq" => config.foc.kp_iq = value,
            "ki_iq" => config.foc.ki_iq = value,
            "spd_kp" => config.foc.speed_kp = value,
            "spd_ki" => config.foc.speed_ki = value,
            _ => return Err(format!("unsupported scan parameter: {param_name}")),
        }
        let result = run_pmsm_foc(&config)?;
        points.push(ScanPoint {
            param_name: param_name.into(),
            param_value: value,
            final_speed: result.metrics.final_speed,
            speed_error_pct: result.metrics.speed_error_pct,
            peak_torque: result.metrics.peak_torque,
            settled: result.metrics.settled,
        });
    }
    points.sort_by(|a, b| a.param_value.total_cmp(&b.param_value));
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_finite_slice(values: &[f64]) {
        for value in values {
            assert!(value.is_finite(), "non-finite value: {value}");
        }
    }

    #[test]
    fn config_rejects_invalid_dt_and_step_dos() {
        let mut config = SimulationConfig {
            dt_ns: 0,
            ..SimulationConfig::default()
        };
        assert!(config.validate().is_err());
        config.dt_ns = 1;
        config.duration_s = 2_000.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn model_registry_rejects_duplicates() {
        let model = MotorModelConfig {
            model_id: "pmsm".into(),
            model_type: "motor".into(),
            fidelity: FidelityLevel::L2,
            parameters: HashMap::new(),
            depends_on: Vec::new(),
        };
        let mut registry = ModelRegistry::new();
        registry.register(&model).unwrap();
        assert!(registry.register(&model).is_err());
    }

    #[test]
    fn data_bus_is_default_deny_for_unknown_module() {
        let mut bus = DataBus::default();
        bus.register_module("sim://pmsm").unwrap();
        bus.allow_topic("motor/speed", "sim://pmsm").unwrap();
        let signal = Signal::new("sim://evil", "omega", 0, 1.0, "rad/s");
        assert!(bus.publish("motor/speed", "sim://evil", signal).is_err());
    }

    #[test]
    fn pmsm_foc_run_has_real_metrics_and_samples() {
        let result = run_pmsm_foc(&SimulationConfig::default()).unwrap();
        assert!(result.samples.len() > 10);
        assert!(result.metrics.steps_executed > 1_000);
        assert!(result.metrics.final_speed.is_finite());
        assert!(result.metrics.peak_torque > 0.0);
        assert!(result.metrics.peak_current > 0.0);
        assert!(result.metrics.max_temperature_c >= 25.0);
        assert!(!result.metrics.cancelled);
    }

    #[test]
    fn pmsm_foc_matches_reference_envelope() {
        let result = run_pmsm_foc(&SimulationConfig::default()).unwrap();
        assert!(result.metrics.final_speed > 20.0);
        assert!(result.metrics.final_speed < 220.0);
        assert!(result.metrics.speed_error_pct < 100.0);
    }

    #[test]
    fn run_can_be_cancelled_without_fake_success() {
        let mut calls = 0u32;
        let result = run_pmsm_foc_with_hooks(
            &SimulationConfig::default(),
            || {
                calls += 1;
                calls > 100
            },
            |_| {},
        )
        .unwrap();
        assert!(result.metrics.cancelled);
        assert!(result.metrics.steps_executed < SimulationConfig::default().total_steps().unwrap());
    }

    #[test]
    fn bldc_step_is_finite_and_hall_progresses() {
        let mut model = BldcModel::new();
        for _ in 0..100 {
            model.step(24.0, 0.0, 50e-6);
        }
        assert_finite_slice(&[model.ia, model.ib, model.ic, model.omega_m, model.torque]);
        assert!(matches!(
            model.hall_state(),
            HallState::H1H2
                | HallState::H1H3
                | HallState::H2H3
                | HallState::H2H1
                | HallState::H3H1
                | HallState::H3H2
        ));
    }

    #[test]
    fn im_model_vector_step_stays_finite() {
        let mut model = ImDqModel::new();
        for _ in 0..100 {
            model.step(3.0, 5.0, 100.0, 0.1, 50e-6);
        }
        assert_finite_slice(&[
            model.ids,
            model.iqs,
            model.psi_rd,
            model.psi_rq,
            model.omega_m,
            model.torque,
        ]);
    }

    #[test]
    fn thermal_model_heats_and_cools_with_finite_limits() {
        let mut node = ThermalNode {
            temperature_c: 25.0,
            ambient_c: 25.0,
            thermal_resistance: 1.0,
            thermal_capacity: 10.0,
        };
        let hot = node.step(100.0, 0.1);
        assert!(hot > 25.0);
        let cooler = node.step(0.0, 5.0);
        assert!(cooler < hot + 1.0);
    }

    #[test]
    fn mpc_prefers_control_that_moves_toward_reference() {
        let mpc = MpcController {
            horizon: 5,
            control_min: -10.0,
            control_max: 10.0,
        };
        let (u, cost) = mpc.solve(0.0, 5.0, |x, u| x + 0.1 * u);
        assert!(u > 0.0);
        assert!(cost.is_finite());
    }

    #[test]
    fn ekf_guards_bad_measurements() {
        let mut ekf = EkfEstimator::default();
        let state = ekf.estimate(f64::NAN, 1.0, f64::INFINITY, 0.0, 0.0, 5.0);
        assert_finite_slice(&state);
    }

    #[test]
    fn scan_returns_sorted_non_empty_results() {
        let config = SimulationConfig {
            duration_s: 0.15,
            ..Default::default()
        };
        let scan = run_parameter_scan(&config, "speed", &[120.0, 60.0]).unwrap();
        assert_eq!(scan.len(), 2);
        assert!(scan[0].param_value < scan[1].param_value);
        assert!(scan.iter().all(|p| p.final_speed.is_finite()));
    }

    #[test]
    fn exports_include_headers_and_json_metrics() {
        let config = SimulationConfig {
            duration_s: 0.05,
            ..Default::default()
        };
        let result = run_pmsm_foc(&config).unwrap();
        assert!(result.to_csv().starts_with("time_s,speed_ref"));
        let json = result.to_json().unwrap();
        assert!(json.contains("final_speed"));
    }
}
