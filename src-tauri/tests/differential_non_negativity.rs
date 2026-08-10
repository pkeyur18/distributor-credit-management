//! T-QA.1-4: property-based test for Rule-9's structural guarantee — no
//! differential term is ever negative, *given a monotonic slab table*.
//! Rule-41 explicitly removes that guarantee at the input layer (a client
//! can save a non-monotonic table and accept the residual risk); this test
//! documents and relies on the monotonic assumption, and must not be read
//! as proof the system rejects a non-monotonic one (it doesn't, by design).
//!
//! The slab-lookup/differential formulas below are a small local
//! reimplementation for this property test only — the real calculation
//! engine is US-M3.1 (S6, `src-tauri/src/m3_calc`). Once it exists,
//! T-M3.1-8 re-points this property at the real function instead of this
//! copy; the two must not be allowed to permanently diverge.

use proptest::prelude::*;

fn slab_pct(thresholds_pct: &[(i64, i64)], tbv: i64) -> i64 {
    thresholds_pct
        .iter()
        .rev()
        .find(|(threshold, _)| tbv >= *threshold)
        .map(|(_, pct)| *pct)
        .unwrap_or(0)
}

fn differential(thresholds_pct: &[(i64, i64)], parent_tbv: i64, child_tbvs: &[i64]) -> i64 {
    let parent_pct = slab_pct(thresholds_pct, parent_tbv);
    child_tbvs
        .iter()
        .map(|&child_tbv| (parent_pct - slab_pct(thresholds_pct, child_tbv)) * child_tbv)
        .sum()
}

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
        // Every child's own contribution rolls up into the parent's, so
        // parent TBV is always at least as large as any single child's —
        // this is what makes Rule-9's guarantee hold under monotonicity.
        let parent_tbv = own_bv + child_bvs.iter().sum::<i64>();
        let total = differential(&table, parent_tbv, &child_bvs);
        prop_assert!(total >= 0, "differential total went negative: {total}");

        for &child_tbv in &child_bvs {
            let parent_pct = slab_pct(&table, parent_tbv);
            let child_pct = slab_pct(&table, child_tbv);
            prop_assert!(
                parent_pct >= child_pct,
                "parent slab % ({parent_pct}) fell below a child's ({child_pct}) despite a monotonic table"
            );
        }
    }
}
