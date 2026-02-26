use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// 可视化类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VizType {
    Line,
    Scatter,
    Bar,
    Gauge,
    Histogram,
    Table,
}

impl VizType {
    pub fn all() -> &'static [VizType] {
        &[
            Self::Line,
            Self::Scatter,
            Self::Bar,
            Self::Gauge,
            Self::Histogram,
            Self::Table,
        ]
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Line => "\u{1F4C8}",      // 📈
            Self::Scatter => "\u{2B50}",    // ⭐
            Self::Bar => "\u{1F4CA}",       // 📊
            Self::Gauge => "\u{1F3AF}",     // 🎯
            Self::Histogram => "\u{1F4CB}", // 📋
            Self::Table => "\u{1F4DD}",     // 📝
        }
    }
}

impl std::fmt::Display for VizType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Line => write!(f, "Line"),
            Self::Scatter => write!(f, "Scatter"),
            Self::Bar => write!(f, "Bar"),
            Self::Gauge => write!(f, "Gauge"),
            Self::Histogram => write!(f, "Histogram"),
            Self::Table => write!(f, "Table"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 数据来源
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// 内置机器人状态字段
    RobotState(RobotStateField),
    /// 从解析的数据包字段获取
    PacketField {
        template_name: String,
        field_name: String,
    },
    /// 自定义表达式（字节偏移提取）
    RawOffset {
        offset: usize,
        field_type: crate::models::packet::FieldType,
        endianness: crate::models::packet::Endianness,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RobotStateField {
    Position,
    Velocity,
    Current,
    Temperature,
    Error,
    PidOutput,
}

impl RobotStateField {
    pub fn all() -> &'static [RobotStateField] {
        &[
            Self::Position,
            Self::Velocity,
            Self::Current,
            Self::Temperature,
            Self::Error,
            Self::PidOutput,
        ]
    }
}

impl std::fmt::Display for RobotStateField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Position => write!(f, "Position"),
            Self::Velocity => write!(f, "Velocity"),
            Self::Current => write!(f, "Current"),
            Self::Temperature => write!(f, "Temperature"),
            Self::Error => write!(f, "Error"),
            Self::PidOutput => write!(f, "PID Output"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 数据通道
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChannel {
    pub name: String,
    pub source: DataSource,
    pub viz_type: VizType,
    pub color: [u8; 3],
    pub enabled: bool,
    pub min_val: f64,
    pub max_val: f64,
    pub unit: String,
}

impl DataChannel {
    pub fn new(name: &str, source: DataSource, viz_type: VizType, color: [u8; 3]) -> Self {
        Self {
            name: name.into(),
            source,
            viz_type,
            color,
            enabled: true,
            min_val: f64::NEG_INFINITY,
            max_val: f64::INFINITY,
            unit: String::new(),
        }
    }

    /// Create default channels for robot state visualization
    pub fn default_channels() -> Vec<DataChannel> {
        vec![
            DataChannel::new(
                "Position",
                DataSource::RobotState(RobotStateField::Position),
                VizType::Line,
                [65, 155, 255],
            ),
            DataChannel::new(
                "Velocity",
                DataSource::RobotState(RobotStateField::Velocity),
                VizType::Line,
                [255, 165, 0],
            ),
            DataChannel::new(
                "Current",
                DataSource::RobotState(RobotStateField::Current),
                VizType::Line,
                [255, 100, 100],
            ),
            DataChannel::new(
                "Temperature",
                DataSource::RobotState(RobotStateField::Temperature),
                VizType::Gauge,
                [255, 100, 255],
            ),
            DataChannel::new(
                "Error",
                DataSource::RobotState(RobotStateField::Error),
                VizType::Line,
                [255, 50, 50],
            ),
            DataChannel::new(
                "PID Output",
                DataSource::RobotState(RobotStateField::PidOutput),
                VizType::Line,
                [100, 255, 100],
            ),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════
// 时间序列数据缓冲区
// ═══════════════════════════════════════════════════════════════

const MAX_DATA_POINTS: usize = 2000;

#[derive(Debug, Clone)]
pub struct TimeSeriesBuffer {
    pub data: Vec<f64>,
    pub max_points: usize,
    pub dropped_points: u64,
}

impl Default for TimeSeriesBuffer {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(256),
            max_points: MAX_DATA_POINTS,
            dropped_points: 0,
        }
    }
}

impl TimeSeriesBuffer {
    pub fn push(&mut self, value: f64) {
        let _ = self.push_with_overflow(value);
    }

    pub fn push_with_overflow(&mut self, value: f64) -> usize {
        self.data.push(value);
        if self.data.len() > self.max_points {
            let overflow = self.data.len() - self.max_points;
            self.data.drain(..overflow);
            self.dropped_points += overflow as u64;
            return overflow;
        }
        0
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn last_n(&self, n: usize) -> &[f64] {
        let start = self.data.len().saturating_sub(n);
        &self.data[start..]
    }

    pub fn as_plot_points(&self) -> Vec<[f64; 2]> {
        let start = self.data.len().saturating_sub(200);
        self.data[start..]
            .iter()
            .enumerate()
            .map(|(i, &v)| [i as f64, v])
            .collect()
    }

    pub fn statistics(&self) -> DataStatistics {
        if self.data.is_empty() {
            return DataStatistics::default();
        }
        let n = self.data.len() as f64;
        let sum: f64 = self.data.iter().sum();
        let mean = sum / n;
        let min = self.data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let variance = self.data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        let last = *self.data.last().unwrap_or(&0.0);

        DataStatistics {
            min,
            max,
            mean,
            std_dev,
            last,
            count: self.data.len(),
        }
    }

    /// Generate histogram bins
    pub fn histogram(&self, num_bins: usize) -> Vec<(f64, usize)> {
        if self.data.is_empty() {
            return vec![];
        }
        let stats = self.statistics();
        let range = stats.max - stats.min;
        if range < 1e-12 {
            return vec![(stats.min, self.data.len())];
        }

        let bin_width = range / num_bins as f64;
        let mut bins = vec![0usize; num_bins];
        for &v in &self.data {
            let idx = ((v - stats.min) / bin_width) as usize;
            let idx = idx.min(num_bins - 1);
            bins[idx] += 1;
        }
        bins.into_iter()
            .enumerate()
            .map(|(i, count)| (stats.min + (i as f64 + 0.5) * bin_width, count))
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataStatistics {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub last: f64,
    pub count: usize,
}

// ═══════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viz_type_all() {
        assert_eq!(VizType::all().len(), 6);
    }

    #[test]
    fn test_viz_type_display() {
        assert_eq!(format!("{}", VizType::Line), "Line");
        assert_eq!(format!("{}", VizType::Gauge), "Gauge");
    }

    #[test]
    fn test_viz_type_icons() {
        for vt in VizType::all() {
            assert!(!vt.icon().is_empty());
        }
    }

    #[test]
    fn test_robot_state_field_all() {
        assert_eq!(RobotStateField::all().len(), 6);
    }

    #[test]
    fn test_data_channel_new() {
        let ch = DataChannel::new(
            "Test",
            DataSource::RobotState(RobotStateField::Position),
            VizType::Line,
            [255, 0, 0],
        );
        assert_eq!(ch.name, "Test");
        assert!(ch.enabled);
    }

    #[test]
    fn test_default_channels() {
        let chs = DataChannel::default_channels();
        assert_eq!(chs.len(), 6);
        assert!(chs.iter().all(|c| c.enabled));
    }

    #[test]
    fn test_time_series_buffer_push() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..10 {
            buf.push(i as f64);
        }
        assert_eq!(buf.data.len(), 10);
    }

    #[test]
    fn test_time_series_buffer_overflow() {
        let mut buf = TimeSeriesBuffer {
            data: Vec::new(),
            max_points: 5,
            dropped_points: 0,
        };
        for i in 0..10 {
            buf.push(i as f64);
        }
        assert_eq!(buf.data.len(), 5);
        assert_eq!(buf.data[0], 5.0);
        assert_eq!(buf.dropped_points, 5);
    }

    #[test]
    fn test_time_series_push_with_overflow_returns_dropped() {
        let mut buf = TimeSeriesBuffer {
            data: Vec::new(),
            max_points: 3,
            dropped_points: 0,
        };
        assert_eq!(buf.push_with_overflow(1.0), 0);
        assert_eq!(buf.push_with_overflow(2.0), 0);
        assert_eq!(buf.push_with_overflow(3.0), 0);
        assert_eq!(buf.push_with_overflow(4.0), 1);
        assert_eq!(buf.data, vec![2.0, 3.0, 4.0]);
        assert_eq!(buf.dropped_points, 1);
    }

    #[test]
    fn test_time_series_clear() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        buf.clear();
        assert!(buf.data.is_empty());
    }

    #[test]
    fn test_time_series_last_n() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..10 {
            buf.push(i as f64);
        }
        let last3 = buf.last_n(3);
        assert_eq!(last3, &[7.0, 8.0, 9.0]);
    }

    #[test]
    fn test_time_series_statistics() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0);
        buf.push(5.0);
        let stats = buf.statistics();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert!((stats.mean - 3.0).abs() < 1e-10);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.last, 5.0);
    }

    #[test]
    fn test_statistics_empty() {
        let buf = TimeSeriesBuffer::default();
        let stats = buf.statistics();
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_histogram() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..100 {
            buf.push(i as f64);
        }
        let hist = buf.histogram(10);
        assert_eq!(hist.len(), 10);
        let total: usize = hist.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_histogram_empty() {
        let buf = TimeSeriesBuffer::default();
        assert!(buf.histogram(10).is_empty());
    }

    #[test]
    fn test_histogram_single_value() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(5.0);
        buf.push(5.0);
        buf.push(5.0);
        let hist = buf.histogram(10);
        assert_eq!(hist.len(), 1); // all same value → single bin
        assert_eq!(hist[0].1, 3);
    }

    #[test]
    fn test_as_plot_points() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        let pts = buf.as_plot_points();
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], [0.0, 1.0]);
        assert_eq!(pts[2], [2.0, 3.0]);
    }

    #[test]
    fn test_data_source_packet_field() {
        let src = DataSource::PacketField {
            template_name: "Motor".into(),
            field_name: "Speed".into(),
        };
        assert_eq!(
            src,
            DataSource::PacketField {
                template_name: "Motor".into(),
                field_name: "Speed".into(),
            }
        );
    }
}
