// US-M3.1 — the pure calculation core (04-technical-architecture.md §3.2,
// §5.1). No I/O: every figure a node needs about its children is passed in
// already computed, so this is unit-testable against the six golden
// scenarios (02-business-rules.md §5) without touching a database. The
// caller (`super::recalculate_chain`, or a test walking a fixture tree)
// owns loading the chain and persisting the result.

/// A direct child's already-known figures for the period — either read
/// from `member_period_totals` (the DB caller) or from a fixture tree
/// (the golden-scenario tests).
pub struct ChildFigures {
    pub total_business_volume: i64,
    pub slab_pct: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodeFigures {
    pub total_business_volume: i64,
    pub slab_pct: i64,
    pub differential: i64,
    pub royalty: i64,
    pub own_reward: i64,
    pub rewards: i64,
}

/// Rule-3: the highest threshold at or below `tbv`; 0% below the lowest
/// threshold. Selection is by threshold, not by percentage — under
/// Rule-41's accepted risk (unvalidated monotonicity) a misconfigured
/// table could have the higher threshold carry a lower percentage, and
/// Rule-3 governs that case explicitly.
pub fn slab_lookup(tbv: i64, slabs: &[(i64, i64)]) -> i64 {
    slabs
        .iter()
        .filter(|(threshold, _)| *threshold <= tbv)
        .max_by_key(|(threshold, _)| *threshold)
        .map(|(_, pct)| *pct)
        .unwrap_or(0)
}

/// Rule-10's "top slab": the highest-percentage row in the table, whatever
/// its threshold — not the highest percentage among a member's children.
fn top_slab_percentage(slabs: &[(i64, i64)]) -> i64 {
    slabs.iter().map(|(_, pct)| *pct).max().unwrap_or(0)
}

/// ADR-004: round-half-up, applied once, at the point a term is finalised.
/// `n` is a percentage-times-×100-money product; dividing by 100 gives the
/// term back in ×100 fixed point. Symmetric on negative input so a
/// misconfigured slab table (Rule-41) can't panic here — Rule-9 guarantees
/// this never happens in normal operation.
pub(crate) fn round_half_up_div100(n: i64) -> i64 {
    if n >= 0 {
        (n + 50) / 100
    } else {
        -((-n + 50) / 100)
    }
}

/// Same rounding rule as `round_half_up_div100`, for the one term (royalty)
/// whose rate can carry a fractional percent (T-M7.4-3) and so can't stay on
/// pure-integer ×100 math the way slab-driven terms do.
pub(crate) fn round_half_up_f64(n: f64) -> i64 {
    if n >= 0.0 {
        (n + 0.5).floor() as i64
    } else {
        -((-n + 0.5).floor() as i64)
    }
}

/// One post-order step (Rule-5): given a member's own Business Volume and
/// its direct children's current figures, compute that member's TBV
/// (Rule-6), slab (Rule-7), differential (Rule-8), royalty (Rule-10,
/// Rule-25), own-Business-Volume reward (Rule-46), and Rewards (Rule-12).
pub fn compute_node(
    own_business_volume: i64,
    children: &[ChildFigures],
    slabs: &[(i64, i64)],
    royalty_min_children: i64,
    royalty_rate_percent: f64,
) -> NodeFigures {
    let total_business_volume = own_business_volume
        + children
            .iter()
            .map(|c| c.total_business_volume)
            .sum::<i64>();
    let slab_pct = slab_lookup(total_business_volume, slabs);

    // Rule-8: every direct child re-scanned, base is the child's TBV.
    let differential: i64 = children
        .iter()
        .map(|c| round_half_up_div100((slab_pct - c.slab_pct) * c.total_business_volume))
        .sum();

    // Rule-10: only children on the table's top slab, both to count and to pay.
    let top_slab_pct = top_slab_percentage(slabs);
    let qualifying: Vec<&ChildFigures> = children
        .iter()
        .filter(|c| c.slab_pct == top_slab_pct)
        .collect();
    let royalty: i64 = if qualifying.len() as i64 >= royalty_min_children {
        qualifying
            .iter()
            .map(|c| {
                round_half_up_f64(royalty_rate_percent * c.total_business_volume as f64 / 100.0)
            })
            .sum()
    } else {
        0
    };

    // Rule-46 (CR-4): own Business Volume, at the member's own slab.
    let own_reward = round_half_up_div100(slab_pct * own_business_volume);

    // Rule-12/13: a third additive term, never fed back into BV/TBV.
    let rewards = differential + royalty + own_reward;

    NodeFigures {
        total_business_volume,
        slab_pct,
        differential,
        royalty,
        own_reward,
        rewards,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Regression proof for existing deployments: every already-saved
        /// whole-percent royalty rate (1%, 5%, ...) must still produce the
        /// exact same royalty figure it did before `round_half_up_f64`
        /// replaced `round_half_up_div100` on this term. Volume range goes
        /// to 100 billion ×100-fixed-point units (i.e. ~₹1 trillion of real
        /// money) — several orders of magnitude past any realistic Total
        /// Business Volume and still far under f64's 2^53 exact-integer
        /// ceiling, so this is not a narrow spot-check.
        #[test]
        fn f64_royalty_math_matches_the_old_i64_path_for_whole_percent_rates(
            pct in 0i64..100,
            volume in 0i64..100_000_000_000i64,
        ) {
            let old = round_half_up_div100(pct * volume);
            let new = round_half_up_f64(pct as f64 * volume as f64 / 100.0);
            prop_assert_eq!(old, new, "diverged for pct={} volume={}", pct, volume);
        }

        /// The reported bug's actual scenario (1.50%, 1.25%, ...): checks
        /// the f64 path against an independent, float-free ground truth
        /// computed in i128 straight from the definition "rate_hundredths /
        /// 100 percent of volume" — not against the code under test.
        #[test]
        fn f64_royalty_math_matches_exact_integer_math_for_two_decimal_rates(
            rate_hundredths in 0i64..10_000,
            volume in 0i64..100_000_000_000i64,
        ) {
            let rate_percent = rate_hundredths as f64 / 100.0;
            let got = round_half_up_f64(rate_percent * volume as f64 / 100.0);
            let exact = (rate_hundredths as i128 * volume as i128 + 5_000) / 10_000;
            prop_assert_eq!(got, exact as i64, "rate={} ({:.2}%) volume={}", rate_hundredths, rate_percent, volume);
        }
    }

    /// Deterministic boundary/tie cases the random property tests above
    /// aren't guaranteed to land on exactly: an exact .5 tie, a rate whose
    /// decimal isn't exactly representable in binary (0.33%), a max-realistic
    /// volume, and zero at both ends.
    #[test]
    fn round_half_up_f64_boundary_cases() {
        // 1% of 150 = 1.5 exactly -> ties round up, matching round_half_up_div100.
        assert_eq!(round_half_up_f64(1.0 * 150.0 / 100.0), 2);
        assert_eq!(round_half_up_div100(150), 2);

        // 0.33% (not exactly representable in binary) of 10,000 = 33.00 exactly.
        assert_eq!(round_half_up_f64(0.33 * 10_000.0 / 100.0), 33);

        // 1.25% (exactly representable: 1.25 = 1 + 1/4) of 10,000 = 125.
        assert_eq!(round_half_up_f64(1.25 * 10_000.0 / 100.0), 125);

        // 99.99% of 100 billion ×100-fixed-point units (~₹1 trillion real
        // money) — far past any realistic Total Business Volume.
        let exact = (9_999i128 * 100_000_000_000i128 + 5_000) / 10_000;
        assert_eq!(
            round_half_up_f64(99.99 * 100_000_000_000.0 / 100.0),
            exact as i64
        );

        // Zero at both ends.
        assert_eq!(round_half_up_f64(0.0 * 0.0 / 100.0), 0);
        assert_eq!(round_half_up_f64(5.0 * 0.0 / 100.0), 0);
    }

    // §4.3 default slab table, real units (not ×100 — see the module doc
    // comment; slab_lookup is unit-agnostic as long as thresholds and tbv
    // use the same scale).
    const SLABS: &[(i64, i64)] = &[
        (100, 2),
        (400, 4),
        (1_200, 6),
        (3_000, 8),
        (5_000, 10),
        (7_000, 12),
        (10_000, 14),
    ];

    fn leaf(total_business_volume: i64, slab_pct: i64) -> ChildFigures {
        ChildFigures {
            total_business_volume,
            slab_pct,
        }
    }

    #[test]
    fn below_the_lowest_threshold_is_zero_percent() {
        assert_eq!(slab_lookup(99, SLABS), 0);
    }

    #[test]
    fn a_value_exactly_at_a_threshold_lands_in_the_higher_slab() {
        // Rule-3's own named tests: Scenario 2's C at 3,000 -> 8%;
        // Scenario 4's A at 10,000 -> 14%.
        assert_eq!(slab_lookup(3_000, SLABS), 8);
        assert_eq!(slab_lookup(10_000, SLABS), 14);
    }

    #[test]
    fn selection_is_by_threshold_not_by_percentage() {
        // Rule-41's accepted risk: a non-monotonic table where the higher
        // threshold carries the lower percentage. Rule-3 says threshold
        // wins regardless.
        let misconfigured = &[(100, 20), (200, 5)];
        assert_eq!(slab_lookup(250, misconfigured), 5);
    }

    #[test]
    fn round_half_up_rounds_ties_up_and_is_symmetric_on_negative_input() {
        assert_eq!(round_half_up_div100(149), 1); // 1.49 -> 1
        assert_eq!(round_half_up_div100(150), 2); // 1.50 -> 2
        assert_eq!(round_half_up_div100(-150), -2);
        assert_eq!(round_half_up_div100(0), 0);
    }

    #[test]
    fn differential_sums_every_direct_child_at_the_parents_slab() {
        // Rule-8's own example: D at 6%, children A (2%, 300), B (0%, 50),
        // C (4%, 1000) -> 35, reproducing Scenario 1's differential term.
        let children = [leaf(300, 2), leaf(50, 0), leaf(1_000, 4)];
        let figures = compute_node(500, &children, SLABS, 3, 1.0);
        assert_eq!(figures.total_business_volume, 1_850);
        assert_eq!(figures.slab_pct, 6);
        assert_eq!(figures.differential, 35);
    }

    #[test]
    fn royalty_is_zero_below_the_configured_min_children() {
        let top = top_slab_percentage(SLABS);
        let children = [leaf(10_000, top), leaf(10_000, top)];
        let figures = compute_node(0, &children, SLABS, 3, 1.0);
        assert_eq!(figures.royalty, 0, "only 2 qualifying, min is 3");
    }

    #[test]
    fn royalty_pays_once_the_min_children_boundary_is_reached() {
        let top = top_slab_percentage(SLABS);
        let children = [leaf(10_000, top), leaf(10_000, top), leaf(10_000, top)];
        let figures = compute_node(0, &children, SLABS, 3, 1.0);
        assert_eq!(figures.royalty, 300, "3 qualifying at 1% of 10,000 each");
    }

    #[test]
    fn royalty_rate_supports_a_fractional_percent() {
        // T-M7.4-3: 1.25% of 10,000 = 125, exactly, per qualifying leg.
        let top = top_slab_percentage(SLABS);
        let children = [leaf(10_000, top), leaf(10_000, top), leaf(10_000, top)];
        let figures = compute_node(0, &children, SLABS, 3, 1.25);
        assert_eq!(figures.royalty, 375, "3 qualifying at 1.25% of 10,000 each");
    }

    #[test]
    fn only_top_slab_children_qualify_for_royalty_even_if_they_are_the_richest_present() {
        // Rule-10: "top slab" is the table's own highest-percentage row,
        // not the highest percentage actually present among the children.
        let children = [leaf(9_000, 12), leaf(9_000, 12), leaf(9_000, 12)];
        let figures = compute_node(0, &children, SLABS, 3, 1.0);
        assert_eq!(
            figures.royalty, 0,
            "12% isn't the table's top slab (14% is), so none of these qualify"
        );
    }

    #[test]
    fn royalty_and_differential_never_both_pay_on_the_same_leg() {
        // AC-6/Rule-11: a top-slab child is automatically excluded from the
        // parent's differential term (parent and child share the same
        // slab %, so the per-child term is exactly zero) while still
        // counting toward — and being paid by — royalty. No exclusion
        // logic exists for this; it falls out of the formulas themselves.
        let top = top_slab_percentage(SLABS);
        let children = [leaf(10_000, top), leaf(10_000, top), leaf(10_000, top)];
        let figures = compute_node(0, &children, SLABS, 3, 1.0);
        assert_eq!(
            figures.differential, 0,
            "every child is on the parent's own top slab, so each differential term is zero"
        );
        assert_eq!(
            figures.royalty, 300,
            "the same three children are paid through royalty instead"
        );
    }

    #[test]
    fn own_reward_pays_at_the_members_own_slab_on_their_own_business_volume_only() {
        // Rule-46: A's own BV = 100 at A's own slab (4%) -> 4.
        let children = [leaf(100, 2), leaf(100, 2), leaf(100, 2)];
        let figures = compute_node(100, &children, SLABS, 3, 1.0);
        assert_eq!(figures.slab_pct, 4);
        assert_eq!(figures.own_reward, 4);
    }

    #[test]
    fn rewards_is_the_sum_of_all_three_terms_and_never_negative_in_normal_operation() {
        let children = [leaf(300, 2), leaf(50, 0), leaf(1_000, 4)];
        let figures = compute_node(500, &children, SLABS, 3, 1.0);
        assert_eq!(
            figures.rewards,
            figures.differential + figures.royalty + figures.own_reward
        );
        assert!(figures.differential >= 0);
        assert!(figures.royalty >= 0);
        assert!(figures.own_reward >= 0);
    }

    #[test]
    fn rewards_never_feeds_back_into_total_business_volume() {
        // Rule-13: recomputing the same node twice with the same inputs
        // must be idempotent — TBV depends only on own_bv + children's
        // TBV, never on the previous rewards figure.
        let children = [leaf(300, 2), leaf(50, 0), leaf(1_000, 4)];
        let first = compute_node(500, &children, SLABS, 3, 1.0);
        let second = compute_node(500, &children, SLABS, 3, 1.0);
        assert_eq!(first.total_business_volume, second.total_business_volume);
        assert_eq!(first.rewards, second.rewards);
    }
}
