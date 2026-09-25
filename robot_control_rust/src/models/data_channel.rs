use serde::{Deserialize, Serialize};

// 可视化类型

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
            Self::Line => "line",
            Self::Scatter => "scatter",
            Self::Bar => "bar",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
            Self::Table => "table",
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

// 数据来源

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

// 数据通道

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

// 时间序列数据缓冲区

const MAX_DATA_POINTS: usize = 2000;

#[derive(Debug, Clone)]
pub struct TimeSeriesBuffer {
    pub data: Vec<f64>,
    pub max_points: usize,
    pub dropped_points: u64,
    // Incremental statistics — maintained O(1) per push, recalculated from
    // scratch only on drain (amortized O(1)).
    cached_sum: f64,
    cached_sum_sq: f64,
    cached_min: f64,
    cached_max: f64,
}

impl Default for TimeSeriesBuffer {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(256),
            max_points: MAX_DATA_POINTS,
            dropped_points: 0,
            cached_sum: 0.0,
            cached_sum_sq: 0.0,
            cached_min: f64::INFINITY,
            cached_max: f64::NEG_INFINITY,
        }
    }
}

impl TimeSeriesBuffer {
    pub fn set_max_points(&mut self, max_points: usize) -> usize {
        self.max_points = max_points.max(32);
        if self.data.len() > self.max_points {
            let overflow = self.data.len() - self.max_points;
            self.data.drain(..overflow);
            self.dropped_points += overflow as u64;
            self.recalc_stats();
            return overflow;
        }
        0
    }

    pub fn push(&mut self, value: f64) {
        let _ = self.push_with_overflow(value);
    }

    pub fn push_with_overflow(&mut self, value: f64) -> usize {
        self.data.push(value);
        // O(1) incremental update
        self.cached_sum += value;
        self.cached_sum_sq += value * value;
        if value < self.cached_min {
            self.cached_min = value;
        }
        if value > self.cached_max {
            self.cached_max = value;
        }

        if self.data.len() > self.max_points {
            let overflow = self.data.len() - self.max_points;
            self.data.drain(..overflow);
            self.dropped_points += overflow as u64;
            // Drain invalidates incremental stats — recalculate from remaining data.
            self.recalc_stats();
            return overflow;
        }
        0
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.cached_sum = 0.0;
        self.cached_sum_sq = 0.0;
        self.cached_min = f64::INFINITY;
        self.cached_max = f64::NEG_INFINITY;
    }

    /// Recalculate incremental stats from scratch after a drain.
    fn recalc_stats(&mut self) {
        if self.data.is_empty() {
            self.cached_sum = 0.0;
            self.cached_sum_sq = 0.0;
            self.cached_min = f64::INFINITY;
            self.cached_max = f64::NEG_INFINITY;
            return;
        }
        let mut sum = 0.0f64;
        let mut sum_sq = 0.0f64;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &v in &self.data {
            sum += v;
            sum_sq += v * v;
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        self.cached_sum = sum;
        self.cached_sum_sq = sum_sq;
        self.cached_min = min;
        self.cached_max = max;
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

    /// Compute statistics using cached incremental values — O(1).
    pub fn statistics(&self) -> DataStatistics {
        if self.data.is_empty() {
            return DataStatistics::default();
        }
        let n = self.data.len() as f64;
        let mean = self.cached_sum / n;
        let variance = (self.cached_sum_sq / n) - mean * mean;
        // Clamp to handle floating-point rounding that could make variance < 0
        let std_dev = variance.max(0.0).sqrt();
        let last = *self.data.last().unwrap_or(&0.0);

        DataStatistics {
            min: self.cached_min,
            max: self.cached_max,
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

// 测试

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
            assert!(vt.icon().is_ascii());
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
            ..Default::default()
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
            ..Default::default()
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
    #[test]
    fn set_max_points_clamps_minimum_to_32() {
        let mut buf = TimeSeriesBuffer::default();
        let overflow = buf.set_max_points(5);
        assert_eq!(buf.max_points, 32);
        assert_eq!(overflow, 0);
    }

    #[test]
    fn set_max_points_drains_overflow() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..100 {
            buf.push(i as f64);
        }
        let overflow = buf.set_max_points(50);
        assert_eq!(overflow, 50);
        assert_eq!(buf.data.len(), 50);
        assert_eq!(buf.dropped_points, 50);
        // remaining data should be the last 50 values
        assert_eq!(buf.data[0], 50.0);
        assert_eq!(buf.data[49], 99.0);
    }

    #[test]
    fn set_max_points_no_drain_when_smaller() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..10 {
            buf.push(i as f64);
        }
        let overflow = buf.set_max_points(100);
        assert_eq!(overflow, 0);
        assert_eq!(buf.data.len(), 10);
    }
    #[test]
    fn as_plot_points_caps_at_200() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..500 {
            buf.push(i as f64);
        }
        let pts = buf.as_plot_points();
        assert_eq!(pts.len(), 200);
        // Should be the last 200 values (300..499)
        assert_eq!(pts[0], [0.0, 300.0]);
        assert_eq!(pts[199], [199.0, 499.0]);
    }

    #[test]
    fn as_plot_points_fewer_than_200() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..5 {
            buf.push(i as f64 * 10.0);
        }
        let pts = buf.as_plot_points();
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[0], [0.0, 0.0]);
        assert_eq!(pts[4], [4.0, 40.0]);
    }
    #[test]
    fn statistics_accurate_after_drain() {
        let mut buf = TimeSeriesBuffer {
            max_points: 5,
            ..Default::default()
        };
        // Push 1..=10, so after drain we keep [6,7,8,9,10]
        for i in 1..=10 {
            buf.push(i as f64);
        }
        let stats = buf.statistics();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 6.0);
        assert_eq!(stats.max, 10.0);
        assert_eq!(stats.last, 10.0);
        assert!((stats.mean - 8.0).abs() < 1e-10);
    }

    #[test]
    fn statistics_std_dev_nonnegative() {
        let mut buf = TimeSeriesBuffer::default();
        // Add values that could cause floating point issues
        for i in 0..100 {
            buf.push((i as f64) * 0.000001);
        }
        let stats = buf.statistics();
        assert!(stats.std_dev >= 0.0);
    }

    #[test]
    fn clear_resets_incremental_stats() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(100.0);
        buf.push(-50.0);
        let before = buf.statistics();
        assert_eq!(before.count, 2);

        buf.clear();
        let after = buf.statistics();
        assert_eq!(after.count, 0);
        assert_eq!(after.min, 0.0); // default
        assert_eq!(after.max, 0.0); // default
    }
    #[test]
    fn histogram_with_two_distinct_values() {
        let mut buf = TimeSeriesBuffer::default();
        for _ in 0..50 {
            buf.push(0.0);
        }
        for _ in 0..50 {
            buf.push(100.0);
        }
        let hist = buf.histogram(10);
        assert_eq!(hist.len(), 10);
        let total: usize = hist.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn histogram_with_one_bin() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..20 {
            buf.push(i as f64);
        }
        let hist = buf.histogram(1);
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].1, 20);
    }
    #[test]
    fn last_n_more_than_available() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        let slice = buf.last_n(100);
        assert_eq!(slice, &[1.0, 2.0]);
    }

    #[test]
    fn last_n_zero() {
        let mut buf = TimeSeriesBuffer::default();
        buf.push(1.0);
        let slice = buf.last_n(0);
        assert!(slice.is_empty());
    }
    #[test]
    fn push_batch_drain_multiple() {
        let mut buf = TimeSeriesBuffer {
            max_points: 3,
            ..Default::default()
        };
        // Push 10 items at once via repeated push_with_overflow
        let mut total_dropped = 0usize;
        for i in 0..10 {
            total_dropped += buf.push_with_overflow(i as f64);
        }
        assert_eq!(buf.data.len(), 3);
        assert_eq!(total_dropped, 7);
        assert_eq!(buf.dropped_points, 7);
        assert_eq!(buf.data, vec![7.0, 8.0, 9.0]);
    }
    #[test]
    fn viz_type_all_icons_distinct() {
        let types = VizType::all();
        let icons: Vec<&str> = types.iter().map(|v| v.icon()).collect();
        let unique: std::collections::HashSet<&str> = icons.iter().copied().collect();
        assert_eq!(icons.len(), unique.len());
    }

    #[test]
    fn viz_type_all_display_distinct() {
        let types = VizType::all();
        let displays: Vec<String> = types.iter().map(|v| format!("{}", v)).collect();
        let unique: std::collections::HashSet<String> = displays.iter().cloned().collect();
        assert_eq!(displays.len(), unique.len());
    }
    #[test]
    fn data_channel_roundtrip_serde() {
        let ch = DataChannel {
            name: "Speed".into(),
            source: DataSource::RobotState(RobotStateField::Velocity),
            viz_type: VizType::Scatter,
            color: [100, 200, 50],
            enabled: true,
            min_val: 0.0,
            max_val: 1000.0,
            unit: "rpm".into(),
        };
        let json = serde_json::to_string(&ch).unwrap();
        let restored: DataChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Speed");
        assert_eq!(restored.viz_type, VizType::Scatter);
        assert_eq!(restored.color, [100, 200, 50]);
        assert!(restored.enabled);
        assert!((restored.min_val - 0.0).abs() < f64::EPSILON);
        assert!((restored.max_val - 1000.0).abs() < f64::EPSILON);
        assert_eq!(restored.unit, "rpm");
    }

    #[test]
    fn data_channel_packet_field_serde() {
        let src = DataSource::PacketField {
            template_name: "Motor".into(),
            field_name: "RPM".into(),
        };
        let json = serde_json::to_string(&src).unwrap();
        let restored: DataSource = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, src);
    }

    #[test]
    fn robot_state_field_display_roundtrip() {
        for field in RobotStateField::all() {
            let display = format!("{}", field);
            assert!(!display.is_empty());
        }
    }
    #[test]
    fn default_channels_cover_all_robot_state_fields() {
        let chs = DataChannel::default_channels();
        let fields: Vec<String> = chs
            .iter()
            .filter_map(|c| match &c.source {
                DataSource::RobotState(f) => Some(format!("{}", f)),
                _ => None,
            })
            .collect();
        assert_eq!(fields.len(), 6);
        assert!(fields.iter().any(|f| f == "Position"));
        assert!(fields.iter().any(|f| f == "Velocity"));
        assert!(fields.iter().any(|f| f == "Current"));
        assert!(fields.iter().any(|f| f == "Temperature"));
        assert!(fields.iter().any(|f| f == "Error"));
        assert!(fields.iter().any(|f| f.contains("PID")));
    }
    #[test]
    fn time_series_buffer_default_capacity() {
        let buf = TimeSeriesBuffer::default();
        assert_eq!(buf.max_points, 2000);
        assert_eq!(buf.dropped_points, 0);
    }

    #[test]
    fn time_series_buffer_statistics_after_many_pushes() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..1000 {
            buf.push(i as f64);
        }
        let stats = buf.statistics();
        assert_eq!(stats.count, 1000);
        assert_eq!(stats.min, 0.0);
        assert_eq!(stats.max, 999.0);
        assert!(stats.std_dev > 0.0);
    }

    #[test]
    fn data_channel_new_defaults() {
        let ch = DataChannel::new(
            "Test",
            DataSource::RobotState(RobotStateField::Position),
            VizType::Line,
            [0, 0, 0],
        );
        assert!(ch.enabled);
        assert!(ch.unit.is_empty());
        assert_eq!(ch.min_val, f64::NEG_INFINITY);
        assert_eq!(ch.max_val, f64::INFINITY);
    }

    #[test]
    fn time_series_buffer_as_plot_points_exact_200() {
        let mut buf = TimeSeriesBuffer::default();
        for i in 0..200 {
            buf.push(i as f64);
        }
        let pts = buf.as_plot_points();
        assert_eq!(pts.len(), 200);
    }
}
