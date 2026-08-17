//! T-QA.1-2: the six golden scenarios (02-business-rules.md §5.1-§5.6) as
//! data, not test code — a seventh scenario is a new array entry, never a
//! new function. Every figure here is transcribed directly from the source
//! document in its own units (not the DB's ×100 fixed-point encoding);
//! S6's calculation engine (T-M3.1-8) is what will run these trees through
//! real code and compare against `differential`/`royalty`/`own_reward`.
//!
//! Scenario 3's six children (B-G) are given only as pre-rolled TBV 1,250
//! each — the source document doesn't decompose D's own three children
//! (p1-p3) into individual figures, only stating they're "already folded
//! into D's 1,250". Modelling D (and its five siblings) as leaves with
//! `own_bv` equal to that TBV is behaviourally identical for A's rollup.

#[derive(Clone, Copy)]
pub struct MemberFixture {
    // Read by S6's engine test (T-M3.1-8) once it exists; unused by this
    // sprint's structural self-checks, which only need the BV numbers.
    #[allow(dead_code)]
    pub name: &'static str,
    pub own_bv: i64,
    pub children: &'static [MemberFixture],
}

pub struct GoldenScenario {
    pub name: &'static str,
    pub tree: MemberFixture,
    pub differential: i64,
    pub royalty: i64,
    pub own_reward: i64,
    pub total: i64,
}

const fn leaf(name: &'static str, own_bv: i64) -> MemberFixture {
    MemberFixture {
        name,
        own_bv,
        children: &[],
    }
}

// Top-level `const` items (rather than array literals built inline inside
// `golden_scenarios()`) so the compiler's rvalue static promotion applies
// cleanly — a function call inside a non-const fn's array literal doesn't
// promote to `'static` on its own.
const S1_CHILDREN: &[MemberFixture] = &[leaf("A", 300), leaf("B", 50), leaf("C", 1_000)];
const S2_CHILDREN: &[MemberFixture] = &[leaf("A", 300), leaf("B", 50), leaf("C", 3_000)];
const S3_CHILDREN: &[MemberFixture] = &[
    leaf("B", 1_250),
    leaf("C", 1_250),
    leaf("D", 1_250),
    leaf("E", 1_250),
    leaf("F", 1_250),
    leaf("G", 1_250),
];
const S4_CHILDREN: &[MemberFixture] = &[
    leaf("A", 10_000),
    leaf("B", 20_000),
    leaf("C", 30_000),
    leaf("D", 40_000),
];
const S5_CHILDREN: &[MemberFixture] = &[
    leaf("A", 10_000),
    leaf("B", 10_000),
    leaf("C", 10_000),
    leaf("D", 10_000),
    leaf("E", 2_000),
    leaf("F", 3_000),
    leaf("G", 4_000),
];
const S6_CHILDREN: &[MemberFixture] = &[leaf("B", 100), leaf("C", 100), leaf("D", 100)];

pub fn golden_scenarios() -> [GoldenScenario; 6] {
    [
        // 5.1 — basic differential.
        GoldenScenario {
            name: "Scenario 1 — basic differential",
            tree: MemberFixture {
                name: "D",
                own_bv: 500,
                children: S1_CHILDREN,
            },
            differential: 35,
            royalty: 0,
            own_reward: 30,
            total: 65,
        },
        // 5.2 — differential collapses to zero on an equal slab.
        GoldenScenario {
            name: "Scenario 2 — differential collapses on an equal slab",
            tree: MemberFixture {
                name: "D",
                own_bv: 500,
                children: S2_CHILDREN,
            },
            differential: 22,
            royalty: 0,
            own_reward: 40,
            total: 62,
        },
        // 5.3 — multi-depth rollup. A's own BV (500) is derived, not stated
        // directly, per the source document's own note.
        GoldenScenario {
            name: "Scenario 3 — multi-depth rollup",
            tree: MemberFixture {
                name: "A",
                own_bv: 500,
                children: S3_CHILDREN,
            },
            differential: 450,
            royalty: 0,
            own_reward: 60,
            total: 510,
        },
        // 5.4 — pure royalty. P's own BV is 0, a confirmed write-up
        // simplification, not a rule exception (own BV is always counted).
        GoldenScenario {
            name: "Scenario 4 — pure royalty",
            tree: MemberFixture {
                name: "P",
                own_bv: 0,
                children: S4_CHILDREN,
            },
            differential: 0,
            royalty: 1_000,
            own_reward: 0,
            total: 1_000,
        },
        // 5.5 — differential and royalty together.
        GoldenScenario {
            name: "Scenario 5 — differential and royalty together",
            tree: MemberFixture {
                name: "P",
                own_bv: 0,
                children: S5_CHILDREN,
            },
            differential: 580,
            royalty: 400,
            own_reward: 0,
            total: 980,
        },
        // 5.6 — own Business Volume reward (Rule-46, CR-4). The client's
        // own worked example.
        GoldenScenario {
            name: "Scenario 6 — own Business Volume reward",
            tree: MemberFixture {
                name: "A",
                own_bv: 100,
                children: S6_CHILDREN,
            },
            differential: 6,
            royalty: 0,
            own_reward: 4,
            total: 10,
        },
    ]
}

/// T-QA.1-3: reports which of the three terms diverged when a total moves,
/// rather than just "the total is wrong" — the failure mode that actually
/// matters once the real engine (S6) computes these.
pub struct DivergenceReport {
    pub differential: Option<(i64, i64)>,
    pub royalty: Option<(i64, i64)>,
    pub own_reward: Option<(i64, i64)>,
}

impl DivergenceReport {
    pub fn is_empty(&self) -> bool {
        self.differential.is_none() && self.royalty.is_none() && self.own_reward.is_none()
    }
}

pub fn diverging_terms(
    expected: &GoldenScenario,
    actual_differential: i64,
    actual_royalty: i64,
    actual_own_reward: i64,
) -> DivergenceReport {
    DivergenceReport {
        differential: (expected.differential != actual_differential)
            .then_some((expected.differential, actual_differential)),
        royalty: (expected.royalty != actual_royalty).then_some((expected.royalty, actual_royalty)),
        own_reward: (expected.own_reward != actual_own_reward)
            .then_some((expected.own_reward, actual_own_reward)),
    }
}
