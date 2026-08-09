-- Migration 0001 — full 10-entity schema.
-- Source: documents/refinement/04-technical-architecture.md §4.4, with three
-- corrections from PI/05-decisions-and-gaps.md applied as published (D-14
-- below is a real column removal; D-11/D-12/D-13 are Rust-side enum/lookup
-- corrections with no DDL shape change — see src-tauri/src/db/mod.rs).

CREATE TABLE members (
    id                    INTEGER PRIMARY KEY,
    name                  TEXT NOT NULL,
    phone                 TEXT NOT NULL UNIQUE,
    email                 TEXT NULL,
    address               TEXT NOT NULL,
    introducer_member_id  INTEGER NULL REFERENCES members(id),
    level                 INTEGER NOT NULL,
    is_active             INTEGER NOT NULL DEFAULT 1,
    joining_date          TEXT NOT NULL,
    consent_given         INTEGER NOT NULL,
    consent_date          TEXT NOT NULL,
    created_at            TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_members_phone ON members(phone);
CREATE INDEX idx_members_introducer ON members(introducer_member_id);
CREATE INDEX idx_members_name ON members(name);

CREATE TABLE business_volume_entries (
    id            INTEGER PRIMARY KEY,
    member_id     INTEGER NOT NULL REFERENCES members(id),
    amount        INTEGER NOT NULL CHECK (amount > 0),
    entry_date    TEXT NOT NULL,
    period_month  TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NULL
);
CREATE INDEX idx_bve_member_period ON business_volume_entries(member_id, period_month);

-- D-11: sort_order is display-only (Settings-screen row order). The lookup
-- engine (M3, Sprint 6) scans by `threshold DESC` and must never read this
-- column to decide match order — see Rule-3 vs. the corrected data model.
CREATE TABLE slab_table (
    id          INTEGER PRIMARY KEY,
    threshold   INTEGER NOT NULL,
    percentage  INTEGER NOT NULL,
    sort_order  INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_slab_threshold ON slab_table(threshold);

CREATE TABLE periods (
    id            INTEGER PRIMARY KEY,
    period_month  TEXT NOT NULL UNIQUE,
    status        TEXT NOT NULL CHECK (status IN ('open','awaiting_close','closed')),
    ended_at      TEXT NULL,
    closed_at     TEXT NULL
);
CREATE INDEX idx_periods_status ON periods(status);

CREATE TABLE member_period_totals (
    member_id               INTEGER NOT NULL REFERENCES members(id),
    period_id               INTEGER NOT NULL REFERENCES periods(id),
    business_volume         INTEGER NOT NULL,
    total_business_volume   INTEGER NOT NULL,
    slab_pct                INTEGER NOT NULL,
    differential             INTEGER NOT NULL,
    royalty                  INTEGER NOT NULL,
    own_reward               INTEGER NOT NULL,
    rewards                  INTEGER NOT NULL,
    PRIMARY KEY (member_id, period_id)
);

CREATE TABLE monthly_snapshots (
    id                       INTEGER PRIMARY KEY,
    member_id                INTEGER NOT NULL REFERENCES members(id),
    period_id                INTEGER NOT NULL REFERENCES periods(id),
    version                  INTEGER NOT NULL,
    business_volume          INTEGER NOT NULL,
    total_business_volume    INTEGER NOT NULL,
    slab_pct                 INTEGER NOT NULL,
    differential              INTEGER NOT NULL,
    royalty                   INTEGER NOT NULL,
    own_reward                INTEGER NOT NULL,
    rewards                   INTEGER NOT NULL,
    is_active_status          INTEGER NOT NULL,
    created_at                TEXT NOT NULL,
    UNIQUE (member_id, period_id, version)
);
CREATE INDEX idx_snapshots_period_version ON monthly_snapshots(period_id, version DESC);

-- ADR-012: generalized whole-console backup table (already reflected in the
-- published DDL — period_id nullable, kind/schedule_kind added).
CREATE TABLE backups (
    id                       INTEGER PRIMARY KEY,
    period_id                INTEGER NULL REFERENCES periods(id),
    kind                      TEXT NOT NULL CHECK (kind IN ('period_close','scheduled','manual','pre_restore_safety')),
    schedule_kind             TEXT NULL CHECK (schedule_kind IN ('daily','weekly','monthly')),
    version                  INTEGER NOT NULL,
    internal_retained_path    TEXT NOT NULL,
    external_medium_path      TEXT NULL,
    checksum                  TEXT NOT NULL,
    is_original               INTEGER NOT NULL,
    created_at                TEXT NOT NULL
);
CREATE INDEX idx_backups_period_version ON backups(period_id, version DESC);
CREATE INDEX idx_backups_kind_created ON backups(kind, created_at DESC);

CREATE TABLE settings (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
);

-- D-14: no session_timeout_minutes column — settings is the only source of
-- truth for the session timeout, never duplicated here.
CREATE TABLE auth (
    id                        INTEGER PRIMARY KEY CHECK (id = 1),
    pin_hash                  TEXT NULL,
    password_hash             TEXT NULL,
    failed_attempts           INTEGER NOT NULL DEFAULT 0,
    locked_until              TEXT NULL,
    recovery_codes            TEXT NOT NULL
);

-- D-12/D-13: entity_type/cause have no CHECK enum here (none did in the
-- published DDL) — the Rust-side AuditEntityType/AuditCause enums are the
-- actual enforcement point and carry the corrected value sets.
CREATE TABLE audit_log (
    id           INTEGER PRIMARY KEY,
    entity_type  TEXT NOT NULL,
    entity_id    INTEGER NOT NULL,
    field        TEXT NOT NULL,
    old_value    TEXT NULL,
    new_value    TEXT NULL,
    changed_at   TEXT NOT NULL,
    cause        TEXT NOT NULL
);
CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id);
