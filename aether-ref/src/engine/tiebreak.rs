use crate::types::link::{Availability, LinkId, TelemetrySnapshot};

/// Select the best link from candidates using the spec-defined tie-breaking order:
/// 1. Explicit prefer order (first in list wins)
/// 2. Lowest latency (known values only; missing sorts last)
/// 3. Lexicographic link ID
pub fn select_link(
    prefer: &[LinkId],
    available_links: &[LinkId],
    telemetry: &TelemetrySnapshot,
) -> Option<LinkId> {
    if available_links.is_empty() {
        return None;
    }

    // Filter to only links that are up or degraded
    let up_links: Vec<&LinkId> = available_links
        .iter()
        .filter(|id| {
            telemetry
                .links
                .get(*id)
                .map(|s| s.availability != Availability::Down)
                .unwrap_or(true) // unknown availability = don't exclude
        })
        .collect();

    if up_links.is_empty() {
        return None;
    }

    // 1. Explicit prefer order: return the first preferred link that is available
    for preferred in prefer {
        if up_links.contains(&preferred) {
            return Some(preferred.clone());
        }
    }

    // 2. Lowest latency (known values only, missing sorts last)
    let mut candidates: Vec<(&LinkId, Option<f64>)> = up_links
        .iter()
        .map(|id| {
            let latency = telemetry.links.get(*id).and_then(|s| s.latency_ms);
            (*id, latency)
        })
        .collect();

    candidates.sort_by(|a, b| {
        match (a.1, b.1) {
            (Some(la), Some(lb)) => la.partial_cmp(&lb).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,    // known < unknown
            (None, Some(_)) => std::cmp::Ordering::Greater, // unknown > known
            (None, None) => a.0.cmp(b.0),                   // 3. lexicographic
        }
    });

    // If multiple have same latency, lexicographic link ID breaks the tie
    if candidates.len() > 1 {
        let best_latency = candidates[0].1;
        let tied: Vec<&LinkId> = candidates
            .iter()
            .filter(|(_, lat)| *lat == best_latency)
            .map(|(id, _)| *id)
            .collect();

        if tied.len() > 1 {
            let mut sorted = tied;
            sorted.sort();
            return Some(sorted[0].clone());
        }
    }

    Some(candidates[0].0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::link::{Availability, LinkState};
    use std::collections::BTreeMap;

    fn make_snapshot(entries: Vec<(&str, Option<f64>, Availability)>) -> TelemetrySnapshot {
        let mut links = BTreeMap::new();
        for (id, latency, avail) in entries {
            links.insert(
                LinkId(id.to_string()),
                LinkState {
                    link_id: LinkId(id.to_string()),
                    latency_ms: latency,
                    jitter_ms: None,
                    availability: avail,
                    capacity_mbps: None,
                    timestamp: chrono::Utc::now(),
                    source_id: "test".to_string(),
                },
            );
        }
        TelemetrySnapshot { links }
    }

    #[test]
    fn prefer_list_wins() {
        let snap = make_snapshot(vec![
            ("link_a", Some(100.0), Availability::Up),
            ("link_b", Some(10.0), Availability::Up),
        ]);
        let available = vec![LinkId::from("link_a"), LinkId::from("link_b")];
        let prefer = vec![LinkId::from("link_a")];

        let result = select_link(&prefer, &available, &snap);
        assert_eq!(result, Some(LinkId::from("link_a")));
    }

    #[test]
    fn lowest_latency_wins_without_prefer() {
        let snap = make_snapshot(vec![
            ("link_a", Some(100.0), Availability::Up),
            ("link_b", Some(10.0), Availability::Up),
        ]);
        let available = vec![LinkId::from("link_a"), LinkId::from("link_b")];

        let result = select_link(&[], &available, &snap);
        assert_eq!(result, Some(LinkId::from("link_b")));
    }

    #[test]
    fn lexicographic_tiebreak() {
        let snap = make_snapshot(vec![
            ("link_b", Some(50.0), Availability::Up),
            ("link_a", Some(50.0), Availability::Up),
        ]);
        let available = vec![LinkId::from("link_b"), LinkId::from("link_a")];

        let result = select_link(&[], &available, &snap);
        assert_eq!(result, Some(LinkId::from("link_a")));
    }

    #[test]
    fn missing_latency_sorts_last() {
        let snap = make_snapshot(vec![
            ("link_a", None, Availability::Up),
            ("link_b", Some(50.0), Availability::Up),
        ]);
        let available = vec![LinkId::from("link_a"), LinkId::from("link_b")];

        let result = select_link(&[], &available, &snap);
        assert_eq!(result, Some(LinkId::from("link_b")));
    }

    #[test]
    fn down_links_excluded() {
        let snap = make_snapshot(vec![
            ("link_a", Some(10.0), Availability::Down),
            ("link_b", Some(50.0), Availability::Up),
        ]);
        let available = vec![LinkId::from("link_a"), LinkId::from("link_b")];

        let result = select_link(&[], &available, &snap);
        assert_eq!(result, Some(LinkId::from("link_b")));
    }

    #[test]
    fn no_available_links() {
        let snap = make_snapshot(vec![]);
        let result = select_link(&[], &[], &snap);
        assert_eq!(result, None);
    }

    #[test]
    fn all_links_down() {
        let snap = make_snapshot(vec![
            ("link_a", Some(10.0), Availability::Down),
            ("link_b", Some(20.0), Availability::Down),
        ]);
        let available = vec![LinkId::from("link_a"), LinkId::from("link_b")];
        let result = select_link(&[], &available, &snap);
        assert_eq!(result, None);
    }

    #[test]
    fn degraded_links_are_available() {
        let snap = make_snapshot(vec![
            ("link_a", Some(10.0), Availability::Degraded),
        ]);
        let available = vec![LinkId::from("link_a")];
        let result = select_link(&[], &available, &snap);
        assert_eq!(result, Some(LinkId::from("link_a")));
    }
}
