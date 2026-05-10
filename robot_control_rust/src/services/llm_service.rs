// LLM tuning service for PID parameter recommendations.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

/// Current PID parameters.
#[derive(Debug, Clone, Serialize)]
pub struct PidParams {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
}

/// Suggested parameters returned by the LLM provider.
#[derive(Debug, Clone, Deserialize)]
pub struct SuggestedParams {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    #[serde(default)]
    pub reasoning: String,
}

/// LLM client wrapper.
#[derive(Debug, Clone)]
pub struct LlmService {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    agent: ureq::Agent,
}

impl LlmService {
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        Self::new_with_timeouts(
            api_url,
            api_key,
            model,
            Duration::from_secs(5),
            Duration::from_secs(15),
            Duration::from_secs(15),
        )
    }

    fn new_with_timeouts(
        api_url: String,
        api_key: String,
        model: String,
        connect_timeout: Duration,
        read_timeout: Duration,
        write_timeout: Duration,
    ) -> Self {
        let max_timeout = connect_timeout.max(read_timeout).max(write_timeout);
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(max_timeout))
            .build();
        let agent: ureq::Agent = config.into();
        Self {
            api_url,
            api_key,
            model,
            agent,
        }
    }

    /// Build the prompt from current controller state and error history.
    fn build_prompt(current: &PidParams, errors: &[f64]) -> String {
        let n = errors.len();
        if n == 0 {
            return "No error data available.".to_string();
        }
        let mean_err = errors.iter().map(|e| e.abs()).sum::<f64>() / n as f64;
        let max_err = errors.iter().map(|e| e.abs()).fold(0.0_f64, f64::max);
        let last_10: Vec<String> = errors
            .iter()
            .rev()
            .take(10)
            .map(|e| format!("{:.4}", e))
            .collect();

        // Oscillation indicator: sign changes in consecutive errors.
        let mut sign_changes = 0;
        for i in 1..errors.len() {
            if errors[i] * errors[i - 1] < 0.0 {
                sign_changes += 1;
            }
        }
        let oscillation_rate = sign_changes as f64 / n as f64;

        // Steady-state error estimated from the final 20% samples.
        let tail_start = (n as f64 * 0.8) as usize;
        let steady_state_err = if tail_start < n {
            errors[tail_start..].iter().map(|e| e.abs()).sum::<f64>() / (n - tail_start) as f64
        } else {
            mean_err
        };

        format!(
            r#"You are a PID tuning expert. Analyze the following control system data and suggest optimized PID parameters.

Current PID Parameters:
- Kp = {kp:.6}
- Ki = {ki:.6}
- Kd = {kd:.6}
- Setpoint = {sp:.4}

Error Analysis ({n} data points):
- Mean absolute error: {mean_err:.6}
- Max absolute error: {max_err:.6}
- Oscillation rate: {osc:.4} (sign changes/total)
- Steady-state error: {sse:.6}
- Recent errors (last 10): [{last10}]

Please provide optimized PID parameters. Respond ONLY with a JSON object:
{{"kp": <number>, "ki": <number>, "kd": <number>, "reasoning": "<brief explanation>"}}"#,
            kp = current.kp,
            ki = current.ki,
            kd = current.kd,
            sp = current.setpoint,
            n = n,
            mean_err = mean_err,
            max_err = max_err,
            osc = oscillation_rate,
            sse = steady_state_err,
            last10 = last_10.join(", "),
        )
    }

    /// Request PID suggestions from LLM API.
    pub fn suggest_pid_params(
        &self,
        current: &PidParams,
        errors: &[f64],
    ) -> Result<SuggestedParams> {
        info!(
            "LLM suggest_params: model={}, url={}",
            self.model, self.api_url
        );

        if self.api_key.is_empty() {
            let e = anyhow!("API key is empty");
            error!("LLM request failed: {}", e);
            return Err(e);
        }
        if self.api_url.is_empty() {
            let e = anyhow!("API URL is empty");
            error!("LLM request failed: {}", e);
            return Err(e);
        }

        let prompt = Self::build_prompt(current, errors);

        // Build an OpenAI-compatible request body.
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a PID control tuning assistant. Always respond with valid JSON only."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 500
        });

        let mut resp = self
            .agent
            .post(&self.api_url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(&body)
            .context("HTTP error")?;

        info!("LLM response: status={}", resp.status());

        let status = resp.status().as_u16();
        let body_str = resp.body_mut().read_to_string().context("Read error")?;

        if status != 200 {
            let e = anyhow!("API returned status {}: {}", status, body_str);
            error!("LLM request failed: {}", e);
            return Err(e);
        }

        // Parse OpenAI-compatible response body.
        let resp_json: serde_json::Value =
            serde_json::from_str(&body_str).context("JSON parse error")?;

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .context("No content in response")?;

        // Extract JSON payload from content.
        let json_str = extract_json(content);
        let suggested: SuggestedParams = serde_json::from_str(&json_str)
            .context(format!("Failed to parse suggestion from: {}", json_str))?;

        // Basic sanity check.
        if suggested.kp < 0.0 || suggested.ki < 0.0 || suggested.kd < 0.0 {
            let e = anyhow!("LLM suggested negative parameters");
            error!("LLM request failed: {}", e);
            return Err(e);
        }

        Ok(suggested)
    }
}

/// Extract a JSON object from plain text or markdown snippets.
fn extract_json(text: &str) -> String {
    // Find the outermost JSON object if present.
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_llm_service_new() {
        let svc = LlmService::new(
            "https://api.openai.com/v1".into(),
            "test-key".into(),
            "gpt-4".into(),
        );
        assert_eq!(svc.model, "gpt-4");
        assert_eq!(svc.api_key, "test-key");
    }

    #[test]
    fn test_build_prompt() {
        let params = PidParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 100.0,
        };
        let errors: Vec<f64> = (0..50).map(|i| (50 - i) as f64 * 0.1).collect();
        let prompt = LlmService::build_prompt(&params, &errors);
        assert!(prompt.contains("Kp = 1.000000"));
        assert!(prompt.contains("Ki = 0.100000"));
        assert!(prompt.contains("50 data points"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_build_prompt_with_oscillation() {
        let params = PidParams {
            kp: 2.0,
            ki: 0.5,
            kd: 0.1,
            setpoint: 0.0,
        };
        let errors: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let prompt = LlmService::build_prompt(&params, &errors);
        assert!(prompt.contains("Oscillation rate"));
        assert!(prompt.contains("20 data points"));
    }

    #[test]
    fn test_extract_json_direct() {
        let text = r#"{"kp": 1.5, "ki": 0.2, "kd": 0.05, "reasoning": "test"}"#;
        let json = extract_json(text);
        let parsed: SuggestedParams = serde_json::from_str(&json).unwrap();
        assert!((parsed.kp - 1.5).abs() < 0.001);
        assert!((parsed.ki - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_extract_json_with_markdown() {
        let text = "Here are the parameters:\n```json\n{\"kp\": 2.0, \"ki\": 0.3, \"kd\": 0.08}\n```\nDone.";
        let json = extract_json(text);
        let parsed: SuggestedParams = serde_json::from_str(&json).unwrap();
        assert!((parsed.kp - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_extract_json_no_json() {
        let text = "No JSON here";
        let json = extract_json(text);
        assert_eq!(json, text);
    }

    #[test]
    fn test_suggested_params_default_reasoning() {
        let json = r#"{"kp": 1.0, "ki": 0.1, "kd": 0.01}"#;
        let parsed: SuggestedParams = serde_json::from_str(json).unwrap();
        assert!(parsed.reasoning.is_empty());
    }

    #[test]
    fn test_suggest_empty_key() {
        let svc = LlmService::new("https://example.com".into(), "".into(), "m".into());
        let params = PidParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
        };
        let errors = vec![1.0; 20];
        let result = svc.suggest_pid_params(&params, &errors);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("key"));
    }

    #[test]
    fn test_suggest_empty_url() {
        let svc = LlmService::new("".into(), "key".into(), "m".into());
        let params = PidParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
        };
        let errors = vec![1.0; 20];
        let result = svc.suggest_pid_params(&params, &errors);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("URL"));
    }

    #[test]
    fn test_pid_params_serialize() {
        let params = PidParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 50.0,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"kp\":1.0"));
        assert!(json.contains("\"setpoint\":50.0"));
    }

    #[test]
    fn test_suggest_timeout_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut sink = [0u8; 1024];
                let _ = stream.read(&mut sink);
                std::thread::sleep(Duration::from_millis(300));
            }
        });

        let svc = LlmService::new_with_timeouts(
            format!("http://{}", addr),
            "test-key".into(),
            "test-model".into(),
            Duration::from_millis(50),
            Duration::from_millis(50),
            Duration::from_millis(50),
        );

        let params = PidParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
        };
        let errors = vec![1.0; 20];
        let result = svc.suggest_pid_params(&params, &errors);
        assert!(result.is_err());
    }
}
