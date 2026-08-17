use crate::models::*;

pub struct VisualizationStore {
    pub data_channels: Vec<DataChannel>,
    pub channel_buffers: Vec<TimeSeriesBuffer>,
    pub channel_overflow_events: u64,
}

impl VisualizationStore {
    pub fn new() -> Self {
        Self {
            data_channels: Vec::new(),
            channel_buffers: Vec::new(),
            channel_overflow_events: 0,
        }
    }

    pub fn report_channel_overflow(&mut self) {
        self.channel_overflow_events += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let vs = VisualizationStore::new();
        assert!(vs.data_channels.is_empty());
        assert!(vs.channel_buffers.is_empty());
        assert_eq!(vs.channel_overflow_events, 0);
    }

    #[test]
    fn test_overflow_tracking() {
        let mut vs = VisualizationStore::new();
        vs.report_channel_overflow();
        vs.report_channel_overflow();
        assert_eq!(vs.channel_overflow_events, 2);
    }

    #[test]
    fn test_overflow_tracking_saturates() {
        let mut vs = VisualizationStore::new();
        // Simulate many overflows — should not panic
        for _ in 0..1000 {
            vs.report_channel_overflow();
        }
        assert_eq!(vs.channel_overflow_events, 1000);
    }

    #[test]
    fn test_channel_buffer_count_matches_channels() {
        let mut vs = VisualizationStore::new();
        vs.data_channels.push(crate::models::DataChannel::new(
            "Ch1",
            crate::models::DataSource::RobotState(
                crate::models::data_channel::RobotStateField::Position,
            ),
            crate::models::VizType::Line,
            [255, 0, 0],
        ));
        vs.channel_buffers
            .push(crate::models::TimeSeriesBuffer::default());
        assert_eq!(vs.data_channels.len(), vs.channel_buffers.len());
    }

    #[test]
    fn test_channel_buffer_max_points() {
        let mut buf = crate::models::TimeSeriesBuffer::default();
        buf.max_points = 3;
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0); // should evict oldest
        assert_eq!(buf.data.len(), 3);
        assert_eq!(buf.data[0], 2.0); // 1.0 evicted
        assert_eq!(buf.data[2], 4.0);
    }
}
