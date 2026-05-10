use super::{DisplayMode, LogDirection, LogEntry};
use std::collections::VecDeque;

/// 日志管理器，负责应用日志的存储和导出
///
/// 维护日志条目的增量计数器（tx_count, rx_count, info_count），
/// 避免每帧遍历全部日志计算统计。
pub struct LogManager {
    pub log_entries: VecDeque<LogEntry>,
    pub tx_count: usize,
    pub rx_count: usize,
    pub info_count: usize,
}

impl LogManager {
    pub const MAX_LOG: usize = 5000;

    pub fn new() -> Self {
        Self {
            log_entries: VecDeque::new(),
            tx_count: 0,
            rx_count: 0,
            info_count: 0,
        }
    }

    pub fn add_log(&mut self, direction: LogDirection, msg: &str) {
        // 递增新条目的计数器
        match direction {
            LogDirection::Tx => self.tx_count += 1,
            LogDirection::Rx => self.rx_count += 1,
            LogDirection::Info => self.info_count += 1,
        }
        let entry = LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction,
            data: msg.as_bytes().to_vec(),
            display_mode: DisplayMode::Ascii,
            channel: "System".into(),
        };
        self.log_entries.push_back(entry);
        if self.log_entries.len() > Self::MAX_LOG {
            let removed = self.log_entries.pop_front();
            if let Some(r) = removed {
                match r.direction {
                    LogDirection::Tx => self.tx_count = self.tx_count.saturating_sub(1),
                    LogDirection::Rx => self.rx_count = self.rx_count.saturating_sub(1),
                    LogDirection::Info => self.info_count = self.info_count.saturating_sub(1),
                }
            }
        }
    }

    pub fn add_log_with_display_mode(
        &mut self,
        direction: LogDirection,
        msg: &str,
        display_mode: DisplayMode,
        channel: &str,
    ) {
        // 递增新条目的计数器
        match direction {
            LogDirection::Tx => self.tx_count += 1,
            LogDirection::Rx => self.rx_count += 1,
            LogDirection::Info => self.info_count += 1,
        }
        let entry = LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction,
            data: msg.as_bytes().to_vec(),
            display_mode,
            channel: channel.into(),
        };
        self.log_entries.push_back(entry);
        if self.log_entries.len() > Self::MAX_LOG {
            let removed = self.log_entries.pop_front();
            if let Some(r) = removed {
                match r.direction {
                    LogDirection::Tx => self.tx_count = self.tx_count.saturating_sub(1),
                    LogDirection::Rx => self.rx_count = self.rx_count.saturating_sub(1),
                    LogDirection::Info => self.info_count = self.info_count.saturating_sub(1),
                }
            }
        }
    }

    pub fn add_info_log(&mut self, msg: &str) {
        self.add_log(LogDirection::Info, msg);
    }

    pub fn len(&self) -> usize {
        self.log_entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log_entries.is_empty()
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        (self.tx_count, self.rx_count, self.info_count)
    }

    pub fn export_csv(&self) -> Result<String, String> {
        let mut csv = String::from("Timestamp,Direction,Message\n");
        for entry in &self.log_entries {
            let msg = String::from_utf8_lossy(&entry.data);
            csv.push_str(&format!(
                "{},{:?},\"{}\"\n",
                entry.timestamp,
                entry.direction,
                msg.replace('"', "\"\"")
            ));
        }
        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let lm = LogManager::new();
        assert!(lm.is_empty());
        assert_eq!(lm.len(), 0);
    }

    #[test]
    fn test_add_info_log() {
        let mut lm = LogManager::new();
        lm.add_info_log("test");
        assert_eq!(lm.len(), 1);
        assert!(!lm.is_empty());
    }

    #[test]
    fn test_export_csv_with_entries() {
        let mut lm = LogManager::new();
        lm.add_info_log("hello");
        let csv = lm.export_csv().unwrap();
        assert!(csv.contains("hello"));
    }

    #[test]
    fn test_max_log_limit() {
        let mut lm = LogManager::new();
        for i in 0..5001 {
            lm.add_info_log(&format!("msg {}", i));
        }
        assert_eq!(lm.len(), 5000);
    }

    #[test]
    fn test_tx_rx_info_counts() {
        let mut lm = LogManager::new();
        lm.add_log(LogDirection::Tx, "tx1");
        lm.add_log(LogDirection::Tx, "tx2");
        lm.add_log(LogDirection::Rx, "rx1");
        lm.add_info_log("info1");
        let (tx, rx, info) = lm.counts();
        assert_eq!(tx, 2);
        assert_eq!(rx, 1);
        assert_eq!(info, 1);
    }

    #[test]
    fn test_counts_decrement_on_eviction() {
        let mut lm = LogManager::new();
        for _ in 0..5000 {
            lm.add_log(LogDirection::Tx, "tx");
        }
        let (tx_before, _, _) = lm.counts();
        assert_eq!(tx_before, 5000);
        lm.add_log(LogDirection::Rx, "rx");
        let (tx_after, rx_after, _) = lm.counts();
        assert_eq!(tx_after, 4999);
        assert_eq!(rx_after, 1);
    }

    #[test]
    fn test_add_log_with_display_mode() {
        let mut lm = LogManager::new();
        lm.add_log_with_display_mode(LogDirection::Tx, "test", DisplayMode::Hex, "CAN");
        assert_eq!(lm.len(), 1);
        let entry = &lm.log_entries[0];
        assert_eq!(entry.display_mode, DisplayMode::Hex);
        assert_eq!(entry.channel, "CAN");
    }

    #[test]
    fn test_export_csv_empty() {
        let lm = LogManager::new();
        let csv = lm.export_csv().unwrap();
        assert!(csv.contains("Timestamp,Direction,Message"));
    }

    #[test]
    fn test_export_csv_multiple() {
        let mut lm = LogManager::new();
        lm.add_info_log("msg1");
        lm.add_info_log("msg2");
        let csv = lm.export_csv().unwrap();
        assert!(csv.contains("msg1"));
        assert!(csv.contains("msg2"));
    }

    #[test]
    fn test_export_csv_special_chars() {
        let mut lm = LogManager::new();
        lm.add_info_log("msg with \"quotes\"");
        let csv = lm.export_csv().unwrap();
        assert!(csv.contains("\"quotes\""));
    }

    #[test]
    fn test_saturating_sub_on_eviction() {
        let mut lm = LogManager::new();
        lm.add_info_log("info");
        for _ in 0..5000 {
            lm.add_log(LogDirection::Tx, "tx");
        }
        let (_, _, info) = lm.counts();
        assert_eq!(info, 0);
    }
}
