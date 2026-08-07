# Definition of Done

Project-specific — adapted to an offline, single-user, Tauri/React/Rust/SQLCipher desktop application with no server, no CI infrastructure yet, and no existing code.

## Per user story

A story is **Done** only when all of the following are true:

1. **Implementation complete** — matches the acceptance criteria in [09-implementation-backlog.md](09-implementation-backlog.md) exactly, including the Given/When/Then cases, not just the happy path.
2. **Code review complete** — given this is currently a solo-maintainer project (`architecture.md` TR-6 names this explicitly as a risk), review may be a self-review checklist pass at minimum, but must explicitly re-check the story against [03-business-rules.md](03-business-rules.md) and [07-error-edge-case-matrix.md](07-error-edge-case-matrix.md) for the rules it touches — not just against the acceptance criteria as written.
3. **Unit tests** — written and passing for any calculation, validation, or state-transition logic touched (per [08-testing-strategy.md](08-testing-strategy.md) §2).
4. **Integration tests** — written and passing for any change touching a transaction boundary (chain-upward recalculation, monthly close, closed-month correction versioning).
5. **API/contract tests** — written and passing for any new or modified IPC command, matching its contract in [04-api-specification.md](04-api-specification.md) exactly (request/response shape, validation, authorization, error responses).
6. **E2E test** — added or updated for any story that changes a user-facing workflow, run against the actual built UI, not a mock.
7. **Security validation** — for any story touching auth, encryption, or the audit log: confirm no plaintext credential is logged/stored, confirm the WebView capability allowlist is unchanged unless the story explicitly extends it (and if so, the extension is justified in the PR/commit description).
8. **Static analysis / linting** — Rust (`clippy`) and TypeScript (project ESLint config, once Sprint 0 establishes it) pass with no new warnings.
9. **Dependency vulnerability check** — any new Rust crate or npm package is checked against known advisories before being added (`cargo audit` / `npm audit` or equivalent) — relevant here because this is an offline, security-sensitive, single-database application where a compromised dependency has an unusually large blast radius (it would sit inside the same process as the encryption key).
10. **Database migration** — any schema change ships with a versioned migration, even though the system starts empty at launch (NFR-16) — once real client data exists post-launch, ad-hoc schema edits are not acceptable.
11. **Logging** — any new mutating command writes an `audit_log` entry per its contract's Audit requirement column in [04-api-specification.md](04-api-specification.md); read-only commands correctly do **not** write one.
12. **Documentation** — [12-implementation-context.md](12-implementation-context.md) is updated if the story changes an API contract, data model entity, or business rule interpretation documented there (it is the primary context file for future sessions — letting it drift from reality defeats its purpose).
13. **Acceptance criteria verified** — manually walked through in the built application, not just asserted by passing automated tests, for any story with a UI component (per the general instruction to test the golden path and edge cases in a real browser/app before claiming completion).
14. **Build passes** — the full Tauri build succeeds on both target platforms (Windows, macOS) before a story is considered shippable, not just "builds on the developer's machine."

## Per module (Epic-level)

A module (M1–M9) is **Done** only when, in addition to every story within it meeting the per-story bar above:
- Every Rule/FR/NFR row in [02-requirements-traceability-matrix.md](02-requirements-traceability-matrix.md) attributed to that module shows a passing test, not just "Fully traced."
- Any open item in [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) attributed to that module is resolved — not merely noted and deferred. **As of 6 August 2026 none remain**: every item raised by the readiness analysis is closed, so no module is gated by one. The three that were built in the prototype (settings warning, last-slab-row refusal, data-recovery screen) count as approved reference behaviour and must be ported, like any other approved prototype behaviour, before M7 / M5 / M8 are Done.
- The five worked scenarios (Scenario 1–5) still reproduce their golden totals through the real UI, not just in a unit test, once M2/M3/M4 are all Done together.

**Additional module gates from the 7 August 2026 change requests (CR-1/CR-2/CR-3):**
- **M2 is not Done** until the full entry-eligibility matrix passes as a test, not merely as an implementation: an entry into an ended-but-unclosed month is accepted, a current-month entry is refused naming the blocking month, a closed-month entry is directed to the correction path, and a recalculation triggered in one live period leaves every other live period byte-identical (Rule-36 as amended, TEST-R36).
- **M4 is not Done** until the full hierarchy window has been exercised against a network above 60 members: the size gate names the real count and Cancel opens nothing; the window draws every branch with exactly three fields per node; zoom, fit-width, in-window search and print all work; **the main console is measurably responsive while it draws**; and a figure recorded in the console afterwards leaves the open window unchanged (Rule-45, AC-44, AC-45). The responsiveness measurement is the client's binding constraint on CR-3 and is a gate, not a nicety.
- **M4 is not Done** until phone matching behaves identically in **every** search box in the console, asserted against the shared search function rather than screen by screen (Rule-44, TEST-R44).

## Project-level (pre-handover)

- All nine modules meet the module-level bar above.
- Full UAT pass: the client reconciles all five scenarios and confirms the on-screen figures match their own hand-worked numbers (per `client-requirements-validation.md`'s own stated success criterion).
- Performance targets verified at the 25,000-member design ceiling, not only at the client's actual 500–5,000-member scale.
- A full monthly-close cycle (backup → snapshot → zero → alert clears) has been exercised end-to-end at least once against realistic data volume.
- Handover deliverables match `project-confirmation-summary.html`'s "What you'll receive" section exactly: installable desktop app (no browser/internet dependency), three working exports, backups verified working before anything is cleared, a working recovery-code path, and an audit log that can explain any figure.
- CI pipeline (once Sprint 0 establishes one) is green — this repository currently has **no CI configuration at all**, so "CI passing" is only meaningful once Epic 0 creates it; do not silently skip this DoD item by treating its absence as satisfaction.

## Explicitly not required (do not gold-plate against this DoD)

- Cross-browser or mobile/tablet testing — out of scope by design (NFR-15).
- Load testing for concurrent users — structurally inapplicable, single-user system.
- Localization testing beyond English/Indian date format — no other locale is in scope (NFR-9).
- A monitoring/alerting test — the capability itself was explicitly declined by the client (NFR-12); do not build or test it "for completeness."
