use serde::{Deserialize, Serialize};

/// 简单前馈神经网络 - 用于 PID 自动调参
/// 架构: Input(6) → Hidden(16, ReLU) → Hidden(8, ReLU) → Output(3, Sigmoid)
/// 输入: 最近的误差序列特征(均值、标准差、振荡频率、超调量、稳态误差、上升时间)
/// 输出: Kp, Ki, Kd 调整建议 (归一化到 0-1，然后缩放)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    pub layers: Vec<Layer>,
    pub learning_rate: f64,
    pub training_epochs: usize,
    pub loss_history: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub weights: Vec<Vec<f64>>, // [output_size][input_size]
    pub biases: Vec<f64>,       // [output_size]
    pub activation: Activation,
    // 运行时缓存 (用于反向传播)
    #[serde(skip)]
    pub last_input: Vec<f64>,
    #[serde(skip)]
    pub last_output: Vec<f64>,
    #[serde(skip)]
    pub last_pre_activation: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activation {
    ReLU,
    Sigmoid,
    Tanh,
    Linear,
}

impl NeuralNetwork {
    pub fn new(layer_sizes: &[usize], activations: &[Activation]) -> Self {
        assert!(layer_sizes.len() >= 2);
        assert_eq!(layer_sizes.len() - 1, activations.len());

        let mut layers = Vec::new();
        for i in 0..activations.len() {
            let input_size = layer_sizes[i];
            let output_size = layer_sizes[i + 1];
            layers.push(Layer::new(input_size, output_size, activations[i]));
        }

        Self {
            layers,
            learning_rate: 0.01,
            training_epochs: 0,
            loss_history: Vec::new(),
        }
    }

    /// 创建 PID 调参专用网络
    pub fn pid_tuner() -> Self {
        Self::new(
            &[6, 16, 8, 3],
            &[Activation::ReLU, Activation::ReLU, Activation::Sigmoid],
        )
    }

    /// 前向传播
    pub fn forward(&mut self, input: &[f64]) -> Vec<f64> {
        let mut current = input.to_vec();
        for layer in &mut self.layers {
            current = layer.forward(&current);
        }
        current
    }

    /// 训练一步 (反向传播 + SGD)
    pub fn train_step(&mut self, input: &[f64], target: &[f64]) -> f64 {
        // 前向传播
        let output = self.forward(input);

        // 计算损失 (MSE)
        let loss: f64 = output
            .iter()
            .zip(target.iter())
            .map(|(o, t)| (o - t).powi(2))
            .sum::<f64>()
            / output.len() as f64;

        // 计算输出层梯度
        let mut delta: Vec<f64> = output
            .iter()
            .zip(target.iter())
            .map(|(o, t)| (o - t) * 2.0 / output.len() as f64)
            .collect();

        // 反向传播
        let lr = self.learning_rate;
        for i in (0..self.layers.len()).rev() {
            let layer = &self.layers[i];
            let activation_grad: Vec<f64> = layer
                .last_pre_activation
                .iter()
                .zip(delta.iter())
                .map(|(z, d)| d * layer.activation.derivative(*z))
                .collect();

            let input_for_layer = layer.last_input.clone();

            // 计算传递到上一层的梯度
            let mut new_delta = vec![0.0; input_for_layer.len()];
            for (j, ag) in activation_grad.iter().enumerate() {
                for (k, nd) in new_delta.iter_mut().enumerate() {
                    *nd += ag * self.layers[i].weights[j][k];
                }
            }

            // 更新权重和偏置
            for (j, ag) in activation_grad.iter().enumerate() {
                for (k, inp) in input_for_layer.iter().enumerate() {
                    self.layers[i].weights[j][k] -= lr * ag * inp;
                }
                self.layers[i].biases[j] -= lr * ag;
            }

            delta = new_delta;
        }

        self.training_epochs += 1;
        self.loss_history.push(loss);
        if self.loss_history.len() > 1000 {
            self.loss_history.drain(..self.loss_history.len() - 1000);
        }

        loss
    }

    /// 从误差历史中提取特征
    pub fn extract_features(errors: &[f64]) -> Vec<f64> {
        if errors.is_empty() {
            return vec![0.0; 6];
        }

        let n = errors.len() as f64;
        let mean = errors.iter().sum::<f64>() / n;
        let std_dev = (errors.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / n).sqrt();

        // 振荡：符号变化次数
        let sign_changes = errors
            .windows(2)
            .filter(|w| w[0].signum() != w[1].signum())
            .count() as f64
            / n.max(1.0);

        // 超调量
        let overshoot = errors
            .iter()
            .filter(|&&e| e < 0.0)
            .map(|e| e.abs())
            .fold(0.0f64, f64::max);

        // 稳态误差 (最后 10% 数据的均值)
        let tail_start = (errors.len() as f64 * 0.9) as usize;
        let steady_state = if tail_start < errors.len() {
            errors[tail_start..].iter().map(|e| e.abs()).sum::<f64>()
                / (errors.len() - tail_start) as f64
        } else {
            mean.abs()
        };

        // 上升时间指标 (误差首次降到初始值10%以下的索引比例)
        let initial = errors.first().map(|e| e.abs()).unwrap_or(0.0);
        let rise_time = if initial > 0.0 {
            errors
                .iter()
                .position(|e| e.abs() < initial * 0.1)
                .map(|i| i as f64 / n)
                .unwrap_or(1.0)
        } else {
            0.0
        };

        // 归一化特征
        vec![
            (mean / 100.0).clamp(-1.0, 1.0),
            (std_dev / 100.0).clamp(0.0, 1.0),
            sign_changes.clamp(0.0, 1.0),
            (overshoot / 100.0).clamp(0.0, 1.0),
            (steady_state / 100.0).clamp(0.0, 1.0),
            rise_time.clamp(0.0, 1.0),
        ]
    }
}

impl Layer {
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        // Xavier/He 初始化
        let scale = match activation {
            Activation::ReLU => (2.0 / input_size as f64).sqrt(),
            _ => (1.0 / input_size as f64).sqrt(),
        };

        let weights: Vec<Vec<f64>> = (0..output_size)
            .map(|i| {
                (0..input_size)
                    .map(|j| {
                        // 简单的确定性伪随机初始化
                        let seed = (i * 1337 + j * 7919 + 42) as f64;
                        ((seed.sin() * 10000.0).fract() * 2.0 - 1.0) * scale
                    })
                    .collect()
            })
            .collect();

        let biases = vec![0.0; output_size];

        Self {
            weights,
            biases,
            activation,
            last_input: Vec::new(),
            last_output: Vec::new(),
            last_pre_activation: Vec::new(),
        }
    }

    pub fn forward(&mut self, input: &[f64]) -> Vec<f64> {
        self.last_input = input.to_vec();
        let mut output = Vec::with_capacity(self.biases.len());

        for (weights_row, bias) in self.weights.iter().zip(self.biases.iter()) {
            let z: f64 = weights_row
                .iter()
                .zip(input.iter())
                .map(|(w, x)| w * x)
                .sum::<f64>()
                + bias;
            output.push(z);
        }

        self.last_pre_activation = output.clone();
        self.last_output = output.iter().map(|&z| self.activation.apply(z)).collect();
        self.last_output.clone()
    }
}

impl Activation {
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            Self::ReLU => x.max(0.0),
            Self::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Self::Tanh => x.tanh(),
            Self::Linear => x,
        }
    }

    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            Self::ReLU => {
                if x > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Sigmoid => {
                let s = 1.0 / (1.0 + (-x).exp());
                s * (1.0 - s)
            }
            Self::Tanh => 1.0 - x.tanh().powi(2),
            Self::Linear => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activation_relu() {
        assert_eq!(Activation::ReLU.apply(5.0), 5.0);
        assert_eq!(Activation::ReLU.apply(-3.0), 0.0);
        assert_eq!(Activation::ReLU.apply(0.0), 0.0);
    }

    #[test]
    fn test_activation_sigmoid() {
        let s = Activation::Sigmoid.apply(0.0);
        assert!(
            (s - 0.5).abs() < 1e-10,
            "sigmoid(0) should be 0.5, got {}",
            s
        );
        assert!(Activation::Sigmoid.apply(100.0) > 0.99);
        assert!(Activation::Sigmoid.apply(-100.0) < 0.01);
    }

    #[test]
    fn test_activation_tanh() {
        let t = Activation::Tanh.apply(0.0);
        assert!((t).abs() < 1e-10, "tanh(0) should be 0, got {}", t);
        assert!(Activation::Tanh.apply(10.0) > 0.99);
        assert!(Activation::Tanh.apply(-10.0) < -0.99);
    }

    #[test]
    fn test_activation_linear() {
        assert_eq!(Activation::Linear.apply(42.0), 42.0);
        assert_eq!(Activation::Linear.apply(-1.5), -1.5);
    }

    #[test]
    fn test_activation_derivatives() {
        assert_eq!(Activation::ReLU.derivative(1.0), 1.0);
        assert_eq!(Activation::ReLU.derivative(-1.0), 0.0);
        assert_eq!(Activation::Linear.derivative(999.0), 1.0);

        let sig_d = Activation::Sigmoid.derivative(0.0);
        assert!(
            (sig_d - 0.25).abs() < 1e-10,
            "sigmoid'(0) should be 0.25, got {}",
            sig_d
        );

        let tanh_d = Activation::Tanh.derivative(0.0);
        assert!(
            (tanh_d - 1.0).abs() < 1e-10,
            "tanh'(0) should be 1.0, got {}",
            tanh_d
        );
    }

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new(4, 3, Activation::ReLU);
        assert_eq!(layer.weights.len(), 3); // 3 output neurons
        assert_eq!(layer.weights[0].len(), 4); // 4 input features
        assert_eq!(layer.biases.len(), 3);
    }

    #[test]
    fn test_layer_forward_shape() {
        let mut layer = Layer::new(4, 3, Activation::ReLU);
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let output = layer.forward(&input);
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn test_nn_pid_tuner_shape() {
        let mut nn = NeuralNetwork::pid_tuner();
        // 6 → 16 → 8 → 3
        assert_eq!(nn.layers.len(), 3);
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let output = nn.forward(&input);
        assert_eq!(output.len(), 3);
        // 输出应该在 0-1 之间 (sigmoid)
        for &v in &output {
            assert!((0.0..=1.0).contains(&v), "Output {} should be in [0,1]", v);
        }
    }

    #[test]
    fn test_nn_training_reduces_loss() {
        let mut nn = NeuralNetwork::pid_tuner();
        let input = vec![0.5, 0.3, 0.1, 0.2, 0.1, 0.4];
        let target = vec![0.3, 0.2, 0.1];

        let initial_loss = nn.train_step(&input, &target);
        // 多次训练后 loss 应该下降
        let mut last_loss = initial_loss;
        for _ in 0..100 {
            last_loss = nn.train_step(&input, &target);
        }
        assert!(
            last_loss < initial_loss,
            "Loss should decrease after training: initial={}, final={}",
            initial_loss,
            last_loss
        );
    }

    #[test]
    fn test_nn_loss_history_capped() {
        let mut nn = NeuralNetwork::pid_tuner();
        let input = vec![0.1; 6];
        let target = vec![0.5; 3];
        for _ in 0..1500 {
            nn.train_step(&input, &target);
        }
        assert!(
            nn.loss_history.len() <= 1000,
            "Loss history should be capped at 1000"
        );
    }

    #[test]
    fn test_extract_features_empty() {
        let features = NeuralNetwork::extract_features(&[]);
        assert_eq!(features.len(), 6);
        assert!(features.iter().all(|&f| f == 0.0));
    }

    #[test]
    fn test_extract_features_constant() {
        let errors = vec![5.0; 100];
        let features = NeuralNetwork::extract_features(&errors);
        assert_eq!(features.len(), 6);
        // 标准差应接近0
        assert!(features[1].abs() < 0.01, "Std dev of constant should be ~0");
        // 没有符号变化
        assert_eq!(features[2], 0.0, "No sign changes in constant data");
    }

    #[test]
    fn test_extract_features_oscillating() {
        let errors: Vec<f64> = (0..100)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let features = NeuralNetwork::extract_features(&errors);
        assert_eq!(features.len(), 6);
        // 高振荡
        assert!(
            features[2] > 0.5,
            "Oscillating data should have high sign change ratio"
        );
    }

    #[test]
    fn test_extract_features_bounded() {
        let errors = vec![1000.0, -500.0, 200.0, 0.0, -100.0];
        let features = NeuralNetwork::extract_features(&errors);
        for (i, &f) in features.iter().enumerate() {
            assert!(
                (-1.0..=1.0).contains(&f),
                "Feature[{}]={} should be in [-1,1]",
                i,
                f
            );
        }
    }

    #[test]
    fn test_nn_deterministic_init() {
        // 使用相同结构创建两次，应该得到相同权重
        let nn1 = NeuralNetwork::pid_tuner();
        let nn2 = NeuralNetwork::pid_tuner();
        for (l1, l2) in nn1.layers.iter().zip(nn2.layers.iter()) {
            for (w1, w2) in l1.weights.iter().zip(l2.weights.iter()) {
                for (&v1, &v2) in w1.iter().zip(w2.iter()) {
                    assert_eq!(v1, v2, "Deterministic init should produce same weights");
                }
            }
        }
    }

    #[test]
    fn test_nn_forward_deterministic() {
        let mut nn1 = NeuralNetwork::pid_tuner();
        let mut nn2 = NeuralNetwork::pid_tuner();
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let out1 = nn1.forward(&input);
        let out2 = nn2.forward(&input);
        for (a, b) in out1.iter().zip(out2.iter()) {
            assert!((a - b).abs() < 1e-15, "Forward should be deterministic");
        }
    }
}
