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
}
