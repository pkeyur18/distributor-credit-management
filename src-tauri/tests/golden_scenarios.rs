//! T-QA.1-1/1-2/1-3: the golden-scenario fixture harness. No calculation
//! engine exists yet (US-M3.1, S6) — these tests validate the *fixture
//! data* itself (transcription-correct, internally consistent) and the
//! divergence-reporting helper T-M3.1-8 will use once the engine lands.

mod fixtures;

use fixtures::{diverging_terms, golden_scenarios};

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
