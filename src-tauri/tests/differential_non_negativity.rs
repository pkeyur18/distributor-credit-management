//! T-QA.1-4 / T-M3.1-8: property-based test for Rule-9's structural
//! guarantee — no differential term is ever negative, *given a monotonic
//! slab table*. Rule-41 explicitly removes that guarantee at the input
//! layer (a client can save a non-monotonic table and accept the residual
//! risk); this test documents and relies on the monotonic assumption, and
//! must not be read as proof the system rejects a non-monotonic one (it
//! doesn't, by design).
//!
//! Runs against the real engine (`bvconsole_lib::m3_calc::engine`, US-M3.1,
//! S6) — this used to be a local reimplementation pending the engine's
//! existence; that placeholder is gone now that T-M3.1-8 has a real
//! function to point at.

use bvconsole_lib::m3_calc::engine::{compute_node, slab_lookup, ChildFigures};
use proptest::prelude::*;

/// A monotonic slab table: thresholds strictly increasing, percentages
/// non-decreasing alongside them — Rule-41's assumption, made explicit.
fn monotonic_slab_table() -> impl Strategy<Value = Vec<(i64, i64)>> {
    (1usize..6).prop_flat_map(|n| {
        let deltas = prop::collection::vec(1i64..10_000, n);
        let pct_deltas = prop::collection::vec(0i64..20, n);
        (deltas, pct_deltas).prop_map(|(deltas, pct_deltas)| {
            let mut threshold = 0i64;
            let mut pct = 0i64;
            deltas
                .into_iter()
                .zip(pct_deltas)
                .map(|(d, p)| {
                    threshold += d;
                    pct += p;
                    (threshold, pct)
                })
                .collect()
        })
    })
}

proptest! {
    #[test]
    fn differential_is_never_negative_under_a_monotonic_slab_table(
        table in monotonic_slab_table(),
        own_bv in 0i64..50_000,
        child_bvs in prop::collection::vec(0i64..50_000, 0..8),
    ) {
        let children: Vec<ChildFigures> = child_bvs
            .iter()
            .map(|&tbv| ChildFigures {
                total_business_volume: tbv,
                slab_pct: slab_lookup(tbv, &table),
            })
            .collect();

        // Every child's own contribution rolls up into the parent's, so
        // parent TBV is always at least as large as any single child's —
        // this is what makes Rule-9's guarantee hold under monotonicity.
        let figures = compute_node(own_bv, &children, &table, 3, 1.0);
        prop_assert!(figures.differential >= 0, "differential total went negative: {}", figures.differential);

        for child in &children {
            prop_assert!(
                figures.slab_pct >= child.slab_pct,
                "parent slab % ({}) fell below a child's ({}) despite a monotonic table",
                figures.slab_pct,
                child.slab_pct
            );
        }
    }
}
