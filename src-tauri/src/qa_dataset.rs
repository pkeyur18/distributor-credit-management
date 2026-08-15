// US-QA.5 — synthetic dataset generator (NFR-1/NFR-2/C7). Builds a
// hierarchy and a year of Business Volume entries at any scale by calling
// the real engine (`m1_members`/`m2_entries`) against a real connection, so
// generated data can never drift from the actual validation/calculation
// rules (Rule-16a positivity, consent, phone uniqueness all come free this
// way).
//
// Library module, not `src/bin/generate_dataset.rs` itself — US-QA.6's
// performance harness (`tests/performance.rs`) needs this same generator,
// and a `tests/*.rs` integration test can only link against the library
// crate, never a sibling `bin` target's private functions. `generate_dataset`
// is now a thin CLI wrapper around `generate_dataset_into` below; this file
// carries the actual logic and its own unit tests (moved verbatim).
//
// The current month goes through `m2_entries::record_entry` exactly as the
// live UI would. The 11 months before it cannot (Rule-36, S12): a real
// system never holds live entries anywhere but the current month — every
// earlier month is closed, its figures already moved to `monthly_snapshots`
// (Rule-38). Those months are seeded with a direct `business_volume_entries`
// insert rather than the real `add_closed_month_entry` command (API-45,
// Rule-39) — that command's per-call `write_correction_snapshot` walks the
// full ancestor chain on every single entry, which is fine for one
// correction-panel save but far too slow for seeding a year of bulk test
// data at scale (NFR-1/2). The bulk insert here is rolled up once, with the
// real `m3_calc::recalculate_chain` and snapshotted with the real
// `m5_close::write_period_close_snapshots`/`zero_period_totals` — the same
// functions the actual close flow calls, just without the backup-file
// ceremony, which has nothing to verify for throwaway performance-test data.
use crate::m1_members::{self, AddMemberInput, AddMemberOutcome, CreateRootMemberInput};
use crate::m2_entries::{self, RecordEntryInput};
use crate::{m3_calc, m5_close};
use chrono::Datelike;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// T-QA.5-2: a vocabulary-clean corpus (checked by hand against
// `scripts/vocabulary-grep.mjs`'s `EXCLUDED_WORDS`) — ordinary names and
// places share no words with that list, so this was never going to be
// mechanically checked by the existing tool, which only scans
// `src/**/*.{ts,tsx}` (see `settings.tsx`'s own note on the same point).
const FIRST_NAMES: &[&str] = &[
    "Aarav", "Vivaan", "Aditya", "Vihaan", "Arjun", "Sai", "Reyansh", "Krishna", "Ishaan", "Rohan",
    "Ananya", "Diya", "Saanvi", "Aadhya", "Kavya", "Myra", "Anika", "Riya", "Ishita", "Priya",
    "Meera", "Rahul", "Amit", "Suresh", "Kiran", "Manish", "Neha", "Pooja", "Sanjay", "Deepak",
];
const LAST_NAMES: &[&str] = &[
    "Sharma", "Verma", "Patel", "Gupta", "Reddy", "Nair", "Iyer", "Menon", "Rao", "Joshi", "Mehta",
    "Shah", "Kulkarni", "Desai", "Chauhan", "Bhatt", "Kapoor", "Malhotra", "Singh", "Yadav",
];
const STREETS: &[&str] = &[
    "Lake View Road",
    "Church Street",
    "Station Road",
    "Garden Lane",
    "Hill Crest Avenue",
    "River Side Drive",
    "Temple Street",
    "Market Road",
    "Park Avenue",
    "College Road",
];
const CITIES: &[&str] = &[
    "Pune",
    "Nashik",
    "Nagpur",
    "Surat",
    "Indore",
    "Jaipur",
    "Bhopal",
    "Kochi",
    "Coimbatore",
    "Mysuru",
];

fn synthetic_name(rng: &mut StdRng) -> String {
    format!(
        "{} {}",
        FIRST_NAMES[rng.random_range(0..FIRST_NAMES.len())],
        LAST_NAMES[rng.random_range(0..LAST_NAMES.len())]
    )
}

fn synthetic_address(rng: &mut StdRng) -> String {
    format!(
        "{} {}, {}",
        rng.random_range(1..999),
        STREETS[rng.random_range(0..STREETS.len())],
        CITIES[rng.random_range(0..CITIES.len())]
    )
}

fn synthetic_phone(counter: u64) -> String {
    // 10 digits, always starting 9 — stays clear of any real-looking or
    // already-reserved test range, and deterministic in `counter` alone.
    format!("9{counter:09}")
}

// T-QA.5-3: ~1,000 entries/month at the 500-member scale (client's actual
// range, explicitly variable — C7), 200,000/year at the 25,000-member
// design ceiling. The two named anchors don't share a single ratio, so
// this is a plain lookup rather than a fitted formula.
pub fn entries_per_month(member_count: usize) -> usize {
    match member_count {
        n if n <= 500 => 1_000,
        n if n <= 5_000 => 5_000,
        _ => 16_667,
    }
}

fn add_child(
    conn: &rusqlite::Connection,
    rng: &mut StdRng,
    next_phone: &mut impl FnMut() -> String,
    introducer_id: i64,
) -> i64 {
    let outcome = m1_members::add_member(
        conn,
        AddMemberInput {
            name: synthetic_name(rng),
            phone: next_phone(),
            address: synthetic_address(rng),
            email: None,
            consent_given: true,
            introducer_member_id: introducer_id,
        },
    )
    .expect("add_member");
    let AddMemberOutcome::Created { member, .. } = outcome else {
        panic!("a freshly generated phone number must never collide");
    };
    member.id
}

/// T-QA.5-1: configurable depth *and* branching. `max_depth` (1 = root
/// only) caps how deep the tree grows — once reached, remaining members
/// become extra children of the deepest-reached nodes instead (wider, not
/// deeper), so `target_count` is still always met. `None` leaves depth
/// unconstrained, growing only as deep as `branching` naturally requires.
pub fn build_hierarchy(
    conn: &rusqlite::Connection,
    rng: &mut StdRng,
    target_count: usize,
    branching: usize,
    max_depth: Option<usize>,
) -> Vec<i64> {
    let mut phone_counter: u64 = 0;
    let mut next_phone = move || {
        phone_counter += 1;
        synthetic_phone(phone_counter)
    };

    let root = m1_members::create_root_member(
        conn,
        CreateRootMemberInput {
            name: synthetic_name(rng),
            phone: next_phone(),
            address: synthetic_address(rng),
            email: None,
            consent_given: true,
        },
    )
    .expect("create_root_member");

    let mut all_ids = vec![root.id];
    let mut frontier = std::collections::VecDeque::from([(root.id, 1usize)]);
    // Nodes one level *above* the cap — every child added to one of these
    // lands exactly at `max_depth`, never deeper. The fallback below grows
    // this level wider, never the tree taller.
    let mut leaf_parents: Vec<i64> = Vec::new();

    while all_ids.len() < target_count {
        let Some((introducer_id, depth)) = frontier.pop_front() else {
            break;
        };
        if max_depth.is_some_and(|cap| depth >= cap) {
            continue; // terminal — this node never gets children, at any point
        }
        let child_depth = depth + 1;
        let mut added_any_child = false;
        for _ in 0..branching {
            if all_ids.len() >= target_count {
                break;
            }
            let child_id = add_child(conn, rng, &mut next_phone, introducer_id);
            all_ids.push(child_id);
            frontier.push_back((child_id, child_depth));
            added_any_child = true;
        }
        if added_any_child && max_depth == Some(child_depth) {
            leaf_parents.push(introducer_id);
        }
    }

    // The depth cap stopped generation short of `target_count` — attach the
    // rest as more children of the parents one level above the cap, so
    // every new member still lands at exactly `max_depth`, never deeper.
    if all_ids.len() < target_count {
        assert!(
            !leaf_parents.is_empty(),
            "max_depth must allow at least one level below the root to reach target_count > 1"
        );
        let mut cursor = 0;
        while all_ids.len() < target_count {
            let introducer_id = leaf_parents[cursor % leaf_parents.len()];
            let child_id = add_child(conn, rng, &mut next_phone, introducer_id);
            all_ids.push(child_id);
            cursor += 1;
        }
    }

    all_ids
}

// Rule-36 (S12): the real system never holds live entries in more than the
// current month at once — everything earlier is closed, its figures living
// in `monthly_snapshots`, not `member_period_totals`. `record_entry` refuses
// a month it can't write to, exactly as it must. So the 11 past months here
// are generated, chain-recalculated (the real engine, `m3_calc::recalculate_chain`),
// snapshotted (`m5_close::write_period_close_snapshots`) and zeroed
// (`m5_close::zero_period_totals`) to end in exactly the state a real close
// would leave them — everything but the backup-file ceremony, which has
// nothing to verify for throwaway performance-test data. Only the current
// month is recorded through the real `record_entry` path.
pub fn generate_entries(conn: &rusqlite::Connection, rng: &mut StdRng, member_ids: &[i64]) {
    let today = chrono::Local::now().date_naive();
    let per_month = entries_per_month(member_ids.len());

    for months_ago in (1..12).rev() {
        generate_closed_month(conn, rng, member_ids, today, months_ago, per_month);
    }
    generate_current_month(conn, rng, member_ids, today, per_month);
}

fn generate_closed_month(
    conn: &rusqlite::Connection,
    rng: &mut StdRng,
    member_ids: &[i64],
    today: chrono::NaiveDate,
    months_ago: u32,
    per_month: usize,
) {
    let month_start = today
        .checked_sub_months(chrono::Months::new(months_ago))
        .expect("month arithmetic in range")
        .with_day(1)
        .expect("day 1 always valid");
    let period_month = month_start.format("%Y-%m").to_string();
    let last_day = month_start
        .checked_add_months(chrono::Months::new(1))
        .and_then(|next| next.pred_opt())
        .expect("last day of month");

    conn.execute(
        "INSERT INTO periods (period_month, status, ended_at, closed_at) VALUES (?1, 'closed', ?2, ?2)",
        rusqlite::params![period_month, last_day.to_string()],
    )
    .expect("insert historical period row");
    let period_id = conn.last_insert_rowid();

    for _ in 0..per_month {
        let member_id = member_ids[rng.random_range(0..member_ids.len())];
        let day = rng.random_range(1..=last_day.day());
        let entry_date = month_start.with_day(day).expect("valid day in month");
        // Rule-16a: strictly > 0. Two decimal places, ×100 fixed-point
        // (ADR-004) — a plain random cents value already satisfies both.
        let amount = rng.random_range(100..=500_000i64);

        conn.execute(
            "INSERT INTO business_volume_entries (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![
                member_id,
                amount,
                entry_date.format("%Y-%m-%d").to_string(),
                period_month,
            ],
        )
        .expect("insert historical entry");
        m3_calc::recalculate_chain(conn, member_id, period_id).expect("recalculate_chain");
    }

    m5_close::write_period_close_snapshots(conn, period_id, &last_day.to_string())
        .expect("write historical snapshots");
    m5_close::zero_period_totals(conn, period_id).expect("zero historical totals");
}

fn generate_current_month(
    conn: &rusqlite::Connection,
    rng: &mut StdRng,
    member_ids: &[i64],
    today: chrono::NaiveDate,
    per_month: usize,
) {
    let month_start = today.with_day(1).expect("day 1 always valid");
    conn.execute(
        "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
        [month_start.format("%Y-%m").to_string()],
    )
    .expect("insert current period row");

    for _ in 0..per_month {
        let member_id = member_ids[rng.random_range(0..member_ids.len())];
        let day = rng.random_range(1..=today.day());
        let entry_date = month_start.with_day(day).expect("valid day in month");
        let amount = rng.random_range(100..=500_000i64);

        m2_entries::record_entry(
            conn,
            RecordEntryInput {
                member_id,
                amount,
                entry_date: entry_date.format("%Y-%m-%d").to_string(),
            },
        )
        .expect("record_entry");
    }
}

/// T-QA.5-4: deterministic seeding — the one entry point both
/// `generate_dataset`'s CLI and `tests/performance.rs` call. Builds the
/// hierarchy, then a year of entries (11 closed months + the current one),
/// against `conn` (already open, migrated and seeded). Returns the member
/// ids so a caller can pick targets for search/recalculation timing.
pub fn generate_dataset_into(
    conn: &rusqlite::Connection,
    seed: u64,
    target_count: usize,
    branching: usize,
    max_depth: Option<usize>,
) -> Vec<i64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let member_ids = build_hierarchy(conn, &mut rng, target_count, branching, max_depth);
    generate_entries(conn, &mut rng, &member_ids);
    member_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    // Runs against an in-memory connection deliberately, not a real
    // encrypted file: SQLCipher embeds a random per-file salt on every
    // write, so two files with byte-identical *content* never have
    // identical bytes — a raw-file checksum would fail this test for a
    // reason that has nothing to do with the generator. Logical content
    // (member names, entry amounts) is what T-QA.5-4 actually means by
    // deterministic.
    fn generate(seed: u64, target_count: usize, branching: usize) -> rusqlite::Connection {
        let conn = db::open_seeded_in_memory().unwrap();
        generate_dataset_into(&conn, seed, target_count, branching, None);
        conn
    }

    // `allocate_member_id` (US-M1.1) deliberately draws from `rand::rng()`,
    // the process-global OS-seeded RNG — Rule-35 requires genuinely
    // unpredictable ids, independent of any seed this generator controls.
    // So member id *values*, and therefore `ORDER BY id`, can never repeat
    // across runs — only the generated *content* (names, tree shape, entry
    // amounts) does. Sorted comparison is the correct determinism check;
    // an id-ordered one would fail for a reason that has nothing to do
    // with whether the generator is actually deterministic.
    fn sorted_member_names(conn: &rusqlite::Connection) -> Vec<String> {
        let mut stmt = conn.prepare("SELECT name FROM members").unwrap();
        let mut names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        names.sort();
        names
    }

    fn entry_amount_sum(conn: &rusqlite::Connection) -> i64 {
        conn.query_row("SELECT SUM(amount) FROM business_volume_entries", [], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn the_same_seed_produces_identical_members_and_entries() {
        let a = generate(7, 20, 3);
        let b = generate(7, 20, 3);

        assert_eq!(sorted_member_names(&a), sorted_member_names(&b));
        assert_eq!(entry_amount_sum(&a), entry_amount_sum(&b));
    }

    #[test]
    fn a_different_seed_produces_a_different_dataset() {
        let a = generate(1, 20, 3);
        let b = generate(2, 20, 3);

        assert_ne!(sorted_member_names(&a), sorted_member_names(&b));
    }

    #[test]
    fn max_depth_caps_the_tree_but_still_reaches_the_target_count() {
        let conn = db::open_seeded_in_memory().unwrap();
        let mut rng = StdRng::seed_from_u64(11);
        // branching 2, depth 3 alone could reach at most 1+2+4 = 7 members —
        // the remainder must land as extra breadth at the deepest level.
        let ids = build_hierarchy(&conn, &mut rng, 30, 2, Some(3));

        assert_eq!(ids.len(), 30);
        let max_level: i64 = conn
            .query_row("SELECT MAX(level) FROM members", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            max_level, 3,
            "no member may sit deeper than the configured max depth"
        );
    }

    #[test]
    fn every_generated_entry_amount_is_strictly_positive() {
        let conn = generate(3, 10, 3);
        let min: i64 = conn
            .query_row("SELECT MIN(amount) FROM business_volume_entries", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(min > 0, "Rule-16a: every generated amount must be > 0");
    }

    #[test]
    fn the_hierarchy_reaches_exactly_the_target_member_count() {
        let conn = generate(9, 37, 4);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 37);
    }
}
