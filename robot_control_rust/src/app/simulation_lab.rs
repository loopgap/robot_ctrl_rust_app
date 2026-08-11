use robot_control_core::error::{AppError, AppResult};
use robot_control_core::simulation::{
    run_parameter_scan, run_pmsm_foc_with_hooks, ScanPoint, SimulationConfig, SimulationRunResult,
};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

const DEFAULT_SCAN_VALUES: &str = "60,100,140";

#[derive(Debug)]
pub struct SimulationLabState {
    pub config: SimulationConfig,
    pub duration_text: String,
    pub dt_us_text: String,
    pub speed_ref_text: String,
    pub load_torque_text: String,
    pub scan_param_idx: usize,
    pub scan_values_text: String,
    pub progress: f32,
    pub status: String,
    pub running: bool,
    pub result: Option<SimulationRunResult>,
    pub scan_results: Vec<ScanPoint>,
    result_rx: Option<Receiver<Result<SimulationRunResult, String>>>,
    scan_rx: Option<Receiver<Result<Vec<ScanPoint>, String>>>,
    progress_rx: Option<Receiver<f32>>,
    cancel_tx: Option<Sender<()>>,
}

/// UI-facing error text: keep validation messages verbatim (no
/// "Validation error:" prefix) so status lines read the same as before.
fn ui_err_text(e: AppError) -> String {
    match e {
        AppError::Validation(msg) => msg,
        other => other.to_string(),
    }
}

impl SimulationLabState {
    pub fn new() -> Self {
        let config = SimulationConfig::default();
        Self {
            duration_text: format!("{:.2}", config.duration_s),
            dt_us_text: format!("{:.0}", config.dt_ns as f64 / 1_000.0),
            speed_ref_text: format!("{:.1}", config.speed_ref),
            load_torque_text: format!("{:.2}", config.load_torque),
            config,
            scan_param_idx: 0,
            scan_values_text: DEFAULT_SCAN_VALUES.into(),
            progress: 0.0,
            status: "Ready".into(),
            running: false,
            result: None,
            scan_results: Vec::new(),
            result_rx: None,
            scan_rx: None,
            progress_rx: None,
            cancel_tx: None,
        }
    }

    pub fn scan_params() -> &'static [(&'static str, &'static str)] {
        &[
            ("speed", "Speed reference"),
            ("load", "Load torque"),
            ("kp_id", "Id proportional gain"),
            ("ki_id", "Id integral gain"),
            ("kp_iq", "Iq proportional gain"),
            ("ki_iq", "Iq integral gain"),
            ("spd_kp", "Speed proportional gain"),
            ("spd_ki", "Speed integral gain"),
        ]
    }

    pub fn selected_scan_param(&self) -> &'static str {
        Self::scan_params()
            .get(self.scan_param_idx)
            .map(|(key, _)| *key)
            .unwrap_or("speed")
    }

    pub fn sync_config_from_text(&mut self) -> AppResult<()> {
        let duration_s = parse_finite_positive(&self.duration_text, "duration_s")?;
        let dt_us = parse_finite(&self.dt_us_text, "dt_us")?;
        let speed_ref = parse_finite(&self.speed_ref_text, "speed_ref")?;
        let load_torque = parse_finite(&self.load_torque_text, "load_torque")?;
        let dt_ns = (dt_us * 1_000.0).round() as u64;
        if dt_ns == 0 {
            return Err("dt_us must resolve to a positive nanosecond step".into());
        }
        self.config.duration_s = duration_s;
        self.config.dt_ns = dt_ns;
        self.config.speed_ref = speed_ref;
        self.config.load_torque = load_torque;
        self.config.validate().map_err(ui_err_text)?;
        Ok(())
    }

    pub fn start_run(&mut self) -> AppResult<()> {
        if self.running {
            return Err("simulation is already running".into());
        }
        self.sync_config_from_text()?;
        let config = self.config.clone();
        let (result_tx, result_rx) = mpsc::channel();
        let (progress_tx, progress_rx) = mpsc::channel();
        let (cancel_tx, cancel_rx) = mpsc::channel();
        thread::Builder::new()
            .name("simulation-lab-run".into())
            .spawn(move || {
                let outcome = run_pmsm_foc_with_hooks(
                    &config,
                    || cancel_rx.try_recv().is_ok(),
                    |value| {
                        let _ = progress_tx.send(value.clamp(0.0, 1.0));
                    },
                );
                let _ = result_tx.send(outcome.map_err(ui_err_text));
            })
            .map_err(|err| format!("failed to spawn simulation worker: {err}"))?;

        self.progress = 0.0;
        self.status = "Running".into();
        self.running = true;
        self.result_rx = Some(result_rx);
        self.scan_rx = None;
        self.progress_rx = Some(progress_rx);
        self.cancel_tx = Some(cancel_tx);
        Ok(())
    }

    pub fn start_scan(&mut self) -> AppResult<()> {
        if self.running {
            return Err("simulation is already running".into());
        }
        self.sync_config_from_text()?;
        let values = parse_scan_values(&self.scan_values_text)?;
        let config = self.config.clone();
        let param = self.selected_scan_param().to_owned();
        let (scan_tx, scan_rx) = mpsc::channel();
        thread::Builder::new()
            .name("simulation-lab-scan".into())
            .spawn(move || {
                let _ =
                    scan_tx.send(run_parameter_scan(&config, &param, &values).map_err(ui_err_text));
            })
            .map_err(|err| format!("failed to spawn scan worker: {err}"))?;

        self.progress = 0.0;
        self.status = "Scanning".into();
        self.running = true;
        self.result_rx = None;
        self.scan_rx = Some(scan_rx);
        self.progress_rx = None;
        self.cancel_tx = None;
        Ok(())
    }

    pub fn cancel(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
            self.status = "Cancellation requested".into();
        }
    }

    pub fn can_cancel(&self) -> bool {
        self.cancel_tx.is_some()
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.progress_rx {
            while let Ok(value) = rx.try_recv() {
                self.progress = value.clamp(0.0, 1.0);
            }
        }

        if let Some(rx) = self.result_rx.take() {
            match rx.try_recv() {
                Ok(Ok(result)) => {
                    self.progress = 1.0;
                    self.running = false;
                    self.status = if result.metrics.cancelled {
                        "Cancelled".into()
                    } else {
                        "Completed".into()
                    };
                    self.result = Some(result);
                    self.cancel_tx = None;
                    self.progress_rx = None;
                }
                Ok(Err(err)) => {
                    self.running = false;
                    self.status = format!("Failed: {err}");
                    self.cancel_tx = None;
                    self.progress_rx = None;
                }
                Err(TryRecvError::Empty) => {
                    self.result_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.running = false;
                    self.status = "Simulation worker disconnected".into();
                    self.cancel_tx = None;
                    self.progress_rx = None;
                }
            }
        }

        if let Some(rx) = self.scan_rx.take() {
            match rx.try_recv() {
                Ok(Ok(points)) => {
                    self.progress = 1.0;
                    self.running = false;
                    self.status = "Scan completed".into();
                    self.scan_results = points;
                }
                Ok(Err(err)) => {
                    self.running = false;
                    self.status = format!("Scan failed: {err}");
                }
                Err(TryRecvError::Empty) => {
                    self.scan_rx = Some(rx);
                    if self.running {
                        self.progress = (self.progress + 0.03).min(0.95);
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.running = false;
                    self.status = "Scan worker disconnected".into();
                }
            }
        }
    }

    pub fn export_csv_text(&self) -> String {
        self.result
            .as_ref()
            .map(SimulationRunResult::to_csv)
            .unwrap_or_default()
    }

    pub fn export_json_text(&self) -> String {
        self.result
            .as_ref()
            .and_then(|result| result.to_json().ok())
            .unwrap_or_default()
    }
}

impl Default for SimulationLabState {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_finite(text: &str, name: &str) -> Result<f64, String> {
    let value: f64 = text
        .trim()
        .parse()
        .map_err(|_| format!("{name} must be a finite number"))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(format!("{name} must be finite"))
    }
}

fn parse_finite_positive(text: &str, name: &str) -> Result<f64, String> {
    let value = parse_finite(text, name)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(format!("{name} must be positive"))
    }
}

fn parse_scan_values(text: &str) -> Result<Vec<f64>, String> {
    let values: Result<Vec<_>, _> = text
        .split([',', ';', ' ', '\n', '\t'])
        .filter(|part| !part.trim().is_empty())
        .map(|part| parse_finite(part, "scan value"))
        .collect();
    let values = values?;
    if values.is_empty() {
        Err("scan values must not be empty".into())
    } else if values.len() > 32 {
        Err("scan values are capped at 32 points".into())
    } else {
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_config_parses_ui_fields() {
        let mut state = SimulationLabState::new();
        state.duration_text = "0.25".into();
        state.dt_us_text = "100".into();
        state.speed_ref_text = "80".into();
        state.load_torque_text = "0.2".into();

        state.sync_config_from_text().unwrap();

        assert_eq!(state.config.dt_ns, 100_000);
        assert_eq!(state.config.duration_s, 0.25);
        assert_eq!(state.config.speed_ref, 80.0);
        assert_eq!(state.config.load_torque, 0.2);
    }

    #[test]
    fn scan_values_reject_empty_and_non_finite_inputs() {
        assert!(parse_scan_values("").is_err());
        assert!(parse_scan_values("1,NaN").is_err());
        assert_eq!(parse_scan_values("1, 2; 3").unwrap(), vec![1.0, 2.0, 3.0]);
    }
}
