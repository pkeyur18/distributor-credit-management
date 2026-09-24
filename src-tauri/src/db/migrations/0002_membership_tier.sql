-- Migration 0002 — membership levels (CR-7, Rule-47).
-- Rank only (0 = none, 1..=4 = Gold/Platinum/Diamond/Ace): names are
-- drafts that live in code, never in the database. Existing snapshots stay
-- at 0 — that is what those closed months showed.
ALTER TABLE member_period_totals ADD COLUMN membership_tier INTEGER NOT NULL DEFAULT 0;
ALTER TABLE monthly_snapshots    ADD COLUMN membership_tier INTEGER NOT NULL DEFAULT 0;

-- Levels 2-4's settings, on an already-seeded database only. db/mod.rs
-- runs migrations *before* seed::run, and seed_settings skips entirely
-- when settings is non-empty — inserting unconditionally here would
-- suppress every default setting on a brand-new install (seed.rs owns
-- those). Rates copy the installation's current royalty rate, so an
-- upgrade moves no Rewards figure until the client edits one.
INSERT INTO settings (key, value)
SELECT k, v FROM (
    SELECT 'royalty_tier_2_qualifying_count' AS k, '3' AS v
    UNION ALL SELECT 'royalty_tier_3_qualifying_count', '3'
    UNION ALL SELECT 'royalty_tier_4_qualifying_count', '3'
    UNION ALL SELECT 'royalty_tier_2_rate_percent',
                     (SELECT value FROM settings WHERE key = 'royalty_rate_percent')
    UNION ALL SELECT 'royalty_tier_3_rate_percent',
                     (SELECT value FROM settings WHERE key = 'royalty_rate_percent')
    UNION ALL SELECT 'royalty_tier_4_rate_percent',
                     (SELECT value FROM settings WHERE key = 'royalty_rate_percent')
)
WHERE EXISTS (SELECT 1 FROM settings);
