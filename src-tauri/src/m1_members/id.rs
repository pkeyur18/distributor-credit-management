// Rule-35 (corrected): random, six digits, 100001-999999 inclusive —
// 100000 is never assigned. IDs are never released, which this allocator
// gets for free: a deactivated member's row is never removed (Rule-42), so
// "currently unallocated" is simply "not present in `members`".
use rand::Rng;
use rusqlite::Connection;

use crate::error::AppError;

const MIN_ID: i64 = 100_001;
const MAX_ID: i64 = 999_999;
const MAX_ATTEMPTS: u32 = 1_000;

pub fn allocate_member_id(conn: &Connection) -> Result<i64, AppError> {
    let mut rng = rand::rng();
    for _ in 0..MAX_ATTEMPTS {
        let candidate = rng.random_range(MIN_ID..=MAX_ID);
        let taken: bool = conn
            .query_row("SELECT 1 FROM members WHERE id = ?1", [candidate], |_| {
                Ok(true)
            })
            .optional_bool()?;
        if !taken {
            return Ok(candidate);
        }
    }
    Err(AppError::Conflict {
        message:
            "Could not allocate a member ID — the pool is exhausted or the database is unreachable."
                .into(),
    })
}

// A tiny local extension rather than pulling in `OptionalExtension` for one
// call site with a different name than its `.optional()` — this reads as
// "is it taken", not "here's an Option I now unwrap".
trait OptionalBool {
    fn optional_bool(self) -> Result<bool, rusqlite::Error>;
}

impl OptionalBool for Result<bool, rusqlite::Error> {
    fn optional_bool(self) -> Result<bool, rusqlite::Error> {
        match self {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_seeded_in_memory;

    #[test]
    fn allocates_within_the_corrected_range() {
        let conn = open_seeded_in_memory().unwrap();
        for _ in 0..50 {
            let id = allocate_member_id(&conn).unwrap();
            assert!((MIN_ID..=MAX_ID).contains(&id));
            assert_ne!(id, 100_000, "100000 must never be assigned");
        }
    }

    #[test]
    fn never_reallocates_an_id_already_in_members() {
        let conn = open_seeded_in_memory().unwrap();
        let first = allocate_member_id(&conn).unwrap();
        conn.execute(
            "INSERT INTO members (id, name, phone, address, level, is_active, joining_date, consent_given, consent_date, created_at)
             VALUES (?1, 'X', '0000000000', 'addr', 1, 1, '2026-01-01', 1, '2026-01-01', '2026-01-01')",
            [first],
        )
        .unwrap();

        for _ in 0..200 {
            assert_ne!(allocate_member_id(&conn).unwrap(), first);
        }
    }

    #[test]
    fn allocation_is_not_sequential() {
        let conn = open_seeded_in_memory().unwrap();
        let a = allocate_member_id(&conn).unwrap();
        let b = allocate_member_id(&conn).unwrap();
        // Not a strict proof of randomness, but a sequential allocator would
        // deterministically produce a == b or b == a + 1 here every run.
        assert!(b != a + 1, "allocator must not be sequential");
    }
}
