//! T-QA.1-1/1-2/1-3: the golden-scenario fixture harness (fixture-data
//! self-checks). T-M3.1-8 below runs the same six trees through the real
//! engine (`bvconsole_lib::m3_calc::engine`, US-M3.1, S6) and reconciles
//! against the client's hand-worked totals.

mod fixtures;

use bvconsole_lib::m3_calc::engine::{compute_node, ChildFigures};
use fixtures::{diverging_terms, golden_scenarios, MemberFixture};

// §4.3 default slab table, real units (matching the fixtures' own units —
// see fixtures/mod.rs's doc comment).
const SLABS: &[(i64, i64)] = &[
    (100, 2),
    (400, 4),
    (1_200, 6),
    (3_000, 8),
    (5_000, 10),
    (7_000, 12),
    (10_000, 14),
];
const ROYALTY_MIN_CHILDREN: i64 = 3;
const ROYALTY_RATE_PERCENT: f64 = 1.0;

/// Rule-5's post-order walk over a fixture tree, through the real engine.
fn evaluate(tree: &MemberFixture) -> bvconsole_lib::m3_calc::engine::NodeFigures {
    let children: Vec<ChildFigures> = tree
        .children
        .iter()
        .map(|child| {
            let figures = evaluate(child);
            ChildFigures {
                total_business_volume: figures.total_business_volume,
                slab_pct: figures.slab_pct,
            }
        })
        .collect();
    compute_node(
        tree.own_bv,
        &children,
        SLABS,
        ROYALTY_MIN_CHILDREN,
        ROYALTY_RATE_PERCENT,
    )
}

#[test]
fn there_are_exactly_six_golden_scenarios() {
    assert_eq!(golden_scenarios().len(), 6);
}

#[test]
fn each_scenario_s_three_terms_sum_to_its_documented_total() {
    for scenario in golden_scenarios() {
        assert_eq!(
            scenario.differential + scenario.royalty + scenario.own_reward,
            scenario.total,
            "{} — differential + royalty + own_reward must equal total",
            scenario.name
        );
    }
}

#[test]
fn headline_totals_match_the_client_s_hand_worked_figures() {
    let totals: Vec<i64> = golden_scenarios().iter().map(|s| s.total).collect();
    assert_eq!(totals, vec![65, 62, 510, 1_000, 980, 10]);
}

#[test]
fn scenario_trees_roll_up_to_the_documented_total_business_volume() {
    // Cross-check against each scenario's stated TotalBusinessVolume, so a
    // transcription slip in a child's own_bv is caught here rather than
    // only once the engine exists.
    fn tbv(m: &fixtures::MemberFixture) -> i64 {
        m.own_bv + m.children.iter().map(tbv).sum::<i64>()
    }
    let expected = [1_850, 3_850, 8_000, 100_000, 49_000, 400];
    for (scenario, expected_tbv) in golden_scenarios().iter().zip(expected) {
        assert_eq!(tbv(&scenario.tree), expected_tbv, "{}", scenario.name);
    }
}

#[test]
fn diverging_terms_reports_no_divergence_against_a_matching_scenario() {
    let scenario = &golden_scenarios()[0];
    let report = diverging_terms(
        scenario,
        scenario.differential,
        scenario.royalty,
        scenario.own_reward,
    );
    assert!(report.is_empty());
}

#[test]
fn diverging_terms_names_exactly_which_term_moved() {
    let scenario = &golden_scenarios()[0];
    let report = diverging_terms(
        scenario,
        scenario.differential + 1,
        scenario.royalty,
        scenario.own_reward,
    );
    assert_eq!(
        report.differential,
        Some((scenario.differential, scenario.differential + 1))
    );
    assert_eq!(report.royalty, None);
    assert_eq!(report.own_reward, None);
}

// --- T-M3.1-8: the same six trees, run through the real engine. ---

#[test]
fn all_six_golden_scenarios_reproduce_exactly_through_the_real_engine() {
    for scenario in golden_scenarios() {
        let figures = evaluate(&scenario.tree);
        let report = diverging_terms(
            &scenario,
            figures.differential,
            figures.royalty,
            figures.own_reward,
        );
        assert!(
            report.is_empty(),
            "{}: differential={:?} royalty={:?} own_reward={:?}",
            scenario.name,
            report.differential,
            report.royalty,
            report.own_reward
        );
        assert_eq!(
            figures.differential + figures.royalty + figures.own_reward,
            scenario.total
        );
    }
}

#[test]
fn boundary_children_land_in_the_higher_slab_exactly_as_rule_3_requires() {
    // Scenario 2's C at 3,000 -> 8%; Scenario 4's A at 10,000 -> 14% — the
    // two boundary cases Rule-3's own test names, reproduced against the
    // actual golden fixtures rather than hand-picked numbers.
    let scenario_2_c = golden_scenarios()[1].tree.children[2];
    assert_eq!(scenario_2_c.own_bv, 3_000);
    assert_eq!(evaluate(&scenario_2_c).slab_pct, 8);

    let scenario_4_a = golden_scenarios()[3].tree.children[0];
    assert_eq!(scenario_4_a.own_bv, 10_000);
    assert_eq!(evaluate(&scenario_4_a).slab_pct, 14);
}

const fn leaf_bv(name: &'static str, own_bv: i64) -> MemberFixture {
    MemberFixture {
        name,
        own_bv,
        children: &[],
    }
}

const TWO_TOP_SLAB: &[MemberFixture] = &[leaf_bv("X", 10_000), leaf_bv("Y", 10_000)];
const THREE_TOP_SLAB: &[MemberFixture] = &[
    leaf_bv("X", 10_000),
    leaf_bv("Y", 10_000),
    leaf_bv("Z", 10_000),
];
const PARENT_OF_TWO: MemberFixture = MemberFixture {
    name: "parent2",
    own_bv: 0,
    children: TWO_TOP_SLAB,
};
const PARENT_OF_THREE: MemberFixture = MemberFixture {
    name: "parent3",
    own_bv: 0,
    children: THREE_TOP_SLAB,
};

#[test]
fn royalty_qualifies_only_once_the_configured_min_children_boundary_is_reached() {
    assert_eq!(
        evaluate(&PARENT_OF_TWO).royalty,
        0,
        "2 top-slab children, one short of the default minimum of 3"
    );
    assert_eq!(
        evaluate(&PARENT_OF_THREE).royalty,
        300,
        "3 top-slab children reaches the boundary: 1% of 30,000"
    );
}

// Rule-25/§5.7's worked illustration: A, B, C at 10,000 each under P (and
// identically under Q and R), all three under T.
const ABC: &[MemberFixture] = &[
    leaf_bv("A", 10_000),
    leaf_bv("B", 10_000),
    leaf_bv("C", 10_000),
];
const STACK_P: MemberFixture = MemberFixture {
    name: "P",
    own_bv: 0,
    children: ABC,
};
const STACK_Q: MemberFixture = MemberFixture {
    name: "Q",
    own_bv: 0,
    children: ABC,
};
const STACK_R: MemberFixture = MemberFixture {
    name: "R",
    own_bv: 0,
    children: ABC,
};
const STACK_T: MemberFixture = MemberFixture {
    name: "T",
    own_bv: 0,
    children: &[STACK_P, STACK_Q, STACK_R],
};

#[test]
fn royalty_stacking_illustration_matches_rule_25s_worked_example() {
    let p = evaluate(&STACK_P);
    let q = evaluate(&STACK_Q);
    let r = evaluate(&STACK_R);
    let t = evaluate(&STACK_T);
    assert_eq!(p.royalty, 300, "P: 1% of A+B+C's 30,000");
    assert_eq!(q.royalty, 300);
    assert_eq!(r.royalty, 300);
    assert_eq!(
        t.royalty, 900,
        "T: 1% of P+Q+R's 90,000, all three on the top slab"
    );
    assert_eq!(
        p.royalty + q.royalty + r.royalty + t.royalty,
        1_800,
        "A's original 10,000 attracts royalty twice in the same chain — via P and via T"
    );
}

#[test]
fn rewards_never_feed_back_into_total_business_volume_recomputing_is_idempotent() {
    // Rule-13's own test: re-running recalculation must leave TBV unchanged.
    for scenario in golden_scenarios() {
        let first = evaluate(&scenario.tree);
        let second = evaluate(&scenario.tree);
        assert_eq!(first.total_business_volume, second.total_business_volume);
        assert_eq!(first.rewards, second.rewards);
    }
}

fn leak_chain(levels: i64, own_bv: i64) -> MemberFixture {
    let mut children: &'static [MemberFixture] = &[];
    for _ in 0..levels {
        let node = MemberFixture {
            name: "n",
            own_bv,
            children,
        };
        children = Box::leak(vec![node].into_boxed_slice());
    }
    children[0]
}

#[test]
fn no_rounding_drift_across_a_long_chain() {
    // ADR-004: each term rounds once, at the point it's finalised, never
    // on a running sum. A 15-level chain of a deliberately non-round own
    // Business Volume figure exercises that repeatedly, then checks the
    // root against an independent, single-application reference
    // calculation rather than the engine's own rounding helper.
    const LEVELS: i64 = 15;
    const OWN_BV: i64 = 137;
    let tree = leak_chain(LEVELS, OWN_BV);
    let figures = evaluate(&tree);

    assert_eq!(
        figures.total_business_volume,
        LEVELS * OWN_BV,
        "Rule-6: TBV rollup is plain integer addition, exact by construction"
    );

    let expected_slab_pct = SLABS
        .iter()
        .filter(|(threshold, _)| *threshold <= LEVELS * OWN_BV)
        .max_by_key(|(threshold, _)| *threshold)
        .map(|(_, pct)| *pct)
        .unwrap_or(0);
    let expected_own_reward = (expected_slab_pct * OWN_BV + 50) / 100;

    assert_eq!(figures.slab_pct, expected_slab_pct);
    assert_eq!(figures.own_reward, expected_own_reward);
}
