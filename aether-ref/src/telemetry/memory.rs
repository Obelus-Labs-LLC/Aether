use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::types::link::LinkId;

/// A single experience record for a link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceRecord {
    pub link_id: LinkId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub latency_ms: Option<f64>,
    pub availability_changed: Option<bool>,
    pub event: Option<String>,
}

/// Bounded, inspectable, resettable, exportable link experience memory.
///
/// Per spec section 6.2: Memory MUST be bounded (fixed max size),
/// inspectable, resettable, and exportable.
#[derive(Debug, Clone)]
pub struct ExperienceMemory {
    max_entries_per_link: usize,
    entries: BTreeMap<LinkId, VecDeque<ExperienceRecord>>,
}

/// Exportable snapshot of all experience memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceMemoryExport {
    pub max_entries_per_link: usize,
    pub entries: BTreeMap<LinkId, Vec<ExperienceRecord>>,
}

impl ExperienceMemory {
    pub fn new(max_entries_per_link: usize) -> Self {
        Self {
            max_entries_per_link,
            entries: BTreeMap::new(),
        }
    }

    /// Record an experience for a link. Oldest entries are evicted when full.
    pub fn record(&mut self, record: ExperienceRecord) {
        let queue = self
            .entries
            .entry(record.link_id.clone())
            .or_insert_with(|| VecDeque::with_capacity(self.max_entries_per_link));

        if queue.len() >= self.max_entries_per_link {
            queue.pop_front();
        }
        queue.push_back(record);
    }

    /// Inspect all records for a given link.
    pub fn inspect(&self, link_id: &LinkId) -> Option<&VecDeque<ExperienceRecord>> {
        self.entries.get(link_id)
    }

    /// Reset all memory.
    pub fn reset(&mut self) {
        self.entries.clear();
    }

    /// Reset memory for a specific link.
    pub fn reset_link(&mut self, link_id: &LinkId) {
        self.entries.remove(link_id);
    }

    /// Export all memory for audit/migration.
    pub fn export(&self) -> ExperienceMemoryExport {
        ExperienceMemoryExport {
            max_entries_per_link: self.max_entries_per_link,
            entries: self
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
                .collect(),
        }
    }

    /// Import previously exported memory.
    pub fn import(&mut self, data: ExperienceMemoryExport) {
        self.max_entries_per_link = data.max_entries_per_link;
        self.entries = data
            .entries
            .into_iter()
            .map(|(k, v)| {
                let mut q = VecDeque::with_capacity(self.max_entries_per_link);
                for record in v.into_iter().take(self.max_entries_per_link) {
                    q.push_back(record);
                }
                (k, q)
            })
            .collect();
    }

    /// Average latency for a link from experience records (if available).
    pub fn avg_latency(&self, link_id: &LinkId) -> Option<f64> {
        let records = self.entries.get(link_id)?;
        let latencies: Vec<f64> = records.iter().filter_map(|r| r.latency_ms).collect();
        if latencies.is_empty() {
            return None;
        }
        Some(latencies.iter().sum::<f64>() / latencies.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(link: &str, latency: f64) -> ExperienceRecord {
        ExperienceRecord {
            link_id: LinkId::from(link),
            timestamp: chrono::Utc::now(),
            latency_ms: Some(latency),
            availability_changed: None,
            event: None,
        }
    }

    #[test]
    fn bounded_eviction() {
        let mut mem = ExperienceMemory::new(3);
        for i in 0..5 {
            mem.record(make_record("link_a", i as f64 * 10.0));
        }
        let records = mem.inspect(&LinkId::from("link_a")).unwrap();
        assert_eq!(records.len(), 3);
        // Should have the last 3 entries (20, 30, 40)
        assert_eq!(records[0].latency_ms, Some(20.0));
    }

    #[test]
    fn reset_clears_all() {
        let mut mem = ExperienceMemory::new(10);
        mem.record(make_record("link_a", 10.0));
        mem.record(make_record("link_b", 20.0));
        mem.reset();
        assert!(mem.inspect(&LinkId::from("link_a")).is_none());
        assert!(mem.inspect(&LinkId::from("link_b")).is_none());
    }

    #[test]
    fn export_import_roundtrip() {
        let mut mem = ExperienceMemory::new(10);
        mem.record(make_record("link_a", 10.0));
        mem.record(make_record("link_a", 20.0));

        let exported = mem.export();
        let mut mem2 = ExperienceMemory::new(10);
        mem2.import(exported);

        assert_eq!(mem2.inspect(&LinkId::from("link_a")).unwrap().len(), 2);
    }

    #[test]
    fn avg_latency() {
        let mut mem = ExperienceMemory::new(10);
        mem.record(make_record("link_a", 10.0));
        mem.record(make_record("link_a", 30.0));
        assert_eq!(mem.avg_latency(&LinkId::from("link_a")), Some(20.0));
    }
}
