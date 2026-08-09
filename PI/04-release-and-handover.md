# Release, Deployment & Handover

Covers everything from "the code is written" to "the client is running it unaided three months later." None of this existed in the source document set — `04-technical-architecture.md` §10 states signing is required and stops there; `01-product-and-scope.md` §12 lists five handover deliverables and stops there.

---

## 1. Versioning

Semantic versioning, single source of truth, propagated to `Cargo.toml`, `package.json` and `tauri.conf.json` from one place (`T-REL.1-1`).

| Part | Bumped when |
|---|---|
| **Major** | A change the client must be told about before installing — a schema migration that cannot roll back, a rule change altering computed figures |
| **Minor** | New capability, no behaviour change to existing figures |
| **Patch** | Defect fix |

⚠️ **There is no auto-update mechanism, and there must not be one** (ADR-011). It would require exactly the network capability the offline requirement forbids. Every upgrade is a new installer, delivered and run manually, with the maintainer notifying the client that one is available.

---

## 2. The pre-release gate

**This replaces CI (D-7).** No pipeline exists; DoD §6.3 explicitly warns against treating that absence as satisfaction, so the checks CI would have run are scripted into one gate that must pass before any build is delivered.

Run order — the script exits non-zero on the first failure:

| # | Step | Fails the gate when |
|---|---|---|
| 1 | `clippy` | Any new warning |
| 2 | ESLint | Any new warning |
| 3 | `cargo audit` | ⚠️ **Any known advisory.** A hard gate, not advisory — a compromised dependency here sits inside the same process as the encryption key |
| 4 | `npm audit` | Same |
| 5 | Rust unit + integration suite | Any failure |
| 6 | ⚠️ **Golden-scenario re-verification, as a separately named step** | Any of 65 / 62 / 510 / 1,000 / 980 / 10 has moved. Named separately so a moved total cannot be lost in a wall of passing output |
| 7 | Contract suite (40 commands, closed unauthenticated set of 7) | Any failure, or the set has grown |
| 8 | E2E suite (Windows) | Any failure |
| 9 | Vocabulary grep | ⚠️ Any excluded term in any literal string, filename, or fixture |
| 10 | Windows release build | Build failure |
| 11 | macOS release build | Build failure |
| 12 | macOS manual verification checklist | Any step fails |

⚠️ **The residual risk is discipline (PR-1): nothing forces the gate to run.** It is a script, not a pipeline. Running it is a release-checklist step, and the checklist is the only enforcement.

---

## 3. Build

| Target | Bundle | Host | Notes |
|---|---|---|---|
| **Windows** | `.msi` / `.exe`, native Tauri bundler | Dev-Win | ~10–20MB, no bundled browser runtime |
| **macOS** | `.dmg` / `.app`, native Tauri bundler | Dev-Mac | ⚠️ **Cannot be cross-compiled from Windows.** Dev-Mac is a requirement, not a convenience |

Release profile strips debug symbols. The application icon and window mark are a **plain visual mark only** — no company name, no commercial branding anywhere (BC-4).

---

## 4. Code signing

**Decision D-6.** Windows signed with a self-signed certificate plus a one-time trust install; macOS unsigned; paid certificates deferred.

### Why not a paid certificate

Windows Authenticode from a trusted CA costs money — realistically $200–600/year — and since June 2023 the CA/Browser Forum requires the private key on FIPS 140-2 Level 2 hardware, so even an OV certificate needs a token or cloud HSM. macOS notarization requires a paid Apple Developer Program membership. Neither buys anything this deployment needs, because there is exactly one machine and the installer is hand-delivered.

### Windows runbook (`T-REL.3-2`, `T-REL.3-3`)

1. Generate a self-signed code-signing certificate; store the private key securely with the maintainer.
2. Sign the `.msi`/`.exe` with it as part of the build.
3. **On the client's machine, once:** install the certificate's public half into **Trusted Root Certification Authorities** and **Trusted Publishers**.
4. Verify: the installer's UAC dialog names the publisher instead of showing "Unknown publisher".

⚠️ **Deliver on physical media, not by download.** A file copied from a USB drive carries no Mark-of-the-Web, so SmartScreen never engages. A downloaded file would, and SmartScreen reputation is independent of signature validity — a paid OV certificate would not reliably avoid it either.

⚠️ **Ceiling (PR-3):** the certificate is trusted **only on machines where it has been installed**. This is not a general distribution mechanism and does not need to be — but the cross-device restore target (AC-38) is a second machine, and it needs the same one-time install. Named here so it is not discovered at the client's desk.

### macOS runbook (`T-REL.4-2`, `T-REL.4-3`)

Ships unsigned and un-notarized — a recorded deviation from `04-technical-architecture.md` §10. The client gets a documented one-time Gatekeeper first-open step: right-click → Open → Open, or System Settings → Privacy & Security → Open Anyway.

⚠️ Written **for someone who will not read documentation** (P-1) — screenshots, not prose.

---

## 5. Installer verification

On genuinely clean machines with no development toolchain (S15).

| # | Check | Platform |
|---|---|---|
| 1 | Installer runs; the publisher is named, not "unknown" | Clean-Win |
| 2 | Installer runs; the Gatekeeper step works as documented | Clean-Mac |
| 3 | First-run setup wizard completes — credential set, recovery codes revealed once behind the mandatory confirmation gate | Both |
| 4 | Root member created; a member added; a Business Volume figure recorded; the figures appear correct on screen | Both |
| 5 | All three extracts generate and open in a spreadsheet application | Both |
| 6 | ⚠️ **Cross-device restore (AC-38)** — install on a second machine, restore from a backup file, confirm it reaches **exactly** the state the original held, with **no separate setup step and the same login credential working unchanged** | Clean-Win → Clean-Mac |
| 7 | The macOS manual verification checklist, in full | Clean-Mac |
| 8 | Uninstall leaves no orphaned encrypted data in an unexpected location | Both |

---

## 6. Release checklist

Every step, every release. Not a suggestion.

- [ ] Version bumped in the single source of truth
- [ ] **Pre-release gate passed end to end** (§2) — all twelve steps
- [ ] Golden totals confirmed unmoved, read from the gate output
- [ ] Windows installer built and signed
- [ ] macOS installer built
- [ ] Installer verification complete on both clean machines (§5)
- [ ] Cross-device restore verified
- [ ] Performance targets confirmed at the 25,000-member ceiling
- [ ] Any open S1 or S2 defect closed — no exceptions
- [ ] Deviations D-6, D-7, D-8 re-confirmed as still accepted
- [ ] Installer written to physical media for delivery
- [ ] Maintainer runbook updated if the build process changed

---

## 7. Handover pack

Mapped one-to-one against the five deliverables promised in `01-product-and-scope.md` §12. Nothing more is promised, and nothing promised is missing.

| Promised | Delivered as | Verified by |
|---|---|---|
| A private desktop application — no internet, nothing browser-based | Signed Windows installer on physical media; macOS installer with its Gatekeeper step | §5 checks 1–4 |
| Three ready-to-use extracts, opening directly in a spreadsheet | Monthly data, yearly average, low-contribution — plus closed-month snapshot re-download | §5 check 5, AC-30 |
| Permanent, safe backups, taken before anything is ever cleared | The close's backup gate, plus the whole-console schedule and on-demand backup | AC-22, AC-25, AC-37 |
| A recovery method, in case a login is ever forgotten | One-time recovery codes, issued at setup | AC-35, US-M8.4 |
| A change log, so any figure can always be explained | The audit log, filterable by member | NFR-5, US-M9.1 |

Plus, not promised but required to operate it:

- The **maintainer runbook** — how to build, sign and deliver an upgrade installer, given there is no auto-update.
- The **certificate trust-install steps**, for any future machine.
- The **known-good toolchain record** (TR-1), so a rebuild years later does not fail on a moved dependency.

⚠️ **No user manual.** The client will not read one (P-1) — every recovery path is required to be self-evident from the screen instead. Written documentation here would be a substitute for design work that has already been done.

---

## 8. Training

One session, in this order, because each item builds on the last.

1. **Daily recording** — search by name, member number or phone; one field; save. ⚠️ Timed against SC-5's fifteen-second target with the client doing it themselves.
2. **Reading a member** — how Member Detail explains a figure completely, without leaving the screen (SC-3). The own-Business-Volume line first, then the per-leg differential rows, then royalty, then the total.
3. **The monthly close** — the wizard, and ⚠️ **explicitly: a failed backup aborts the close entirely and nothing is lost.** The client must not fear the gate; they must understand it is protecting them.
4. **The outstanding-month behaviour** — that an ended-but-unclosed month keeps accepting entries dated within it, and the current month unlocks once it closes. ⚠️ This is CR-2's whole point and the client asked for it; they should recognise the behaviour as theirs.
5. ⚠️ **The external-medium backup discipline.** TR-4 leaves this unenforced by design — the internal retained copy is the actual gate, and the external copy is prompted but never blocking. **It is the single point of failure the software does not defend against, so it is taught as a client process, plainly.**
6. **Corrections** — that a closed month can be corrected, that it writes a new version, and that the original is never touched.
7. **Settings self-service** — the client changes one themselves, unaided, once (SC-6). ⚠️ Including the on-screen disclaimer explaining that the slab table is not checked for consistency, because they declined that safeguard.
8. **Restore** — that it always names what it replaces, requires deliberate confirmation, and takes a safety backup of the current state first.
9. ⚠️ **Recovery-code custody.** Where the codes live, and the plain statement that **losing both the credential and the codes is permanently unrecoverable** — no vendor backdoor, no email flow, no way back. This is the direct, accepted cost of "nobody but the client can ever get in", and it must be said at handover, not buried in settings.

---

## 9. Hypercare

The support window after go-live. Not open-ended, and not zero.

| Phase | Duration | What happens |
|---|---|---|
| **First close** | Whichever month ends first | ⚠️ **Support the client's first live monthly close end to end.** It is the first time the irreversible action runs against real data, and NFR-12 means nothing will detect it silently failing |
| **First month** | 1 month | Spot-check figures against hand calculations (SC-2's second half). Respond to questions within a working day |
| **Measurement window** | 3 months | ⚠️ SC-1's stated period, after which the client confirms they no longer perform any reward calculation by hand. SC-4 checked at the end: **every month since go-live holds a permanent record and a retained backup** |
| **After** | — | No standing operational role (P-3). The maintainer builds and supports on request |

---

## 10. Post-handover risks the client owns

Stated so they are transferred deliberately rather than assumed.

| Risk | Why the software does not solve it |
|---|---|
| **Never taking the external-medium backup copy** (TR-4) | The internal copy is the real gate; forcing the close to block on an external write would contradict a documented decision (RQ-6). Taught at handover, not enforced |
| **A close silently failing to write its record** | Monitoring was **explicitly declined** by the client (NFR-12). Nothing detects it |
| **Losing the credential and all recovery codes** | Permanently unrecoverable by design (ADR-008). The direct cost of no vendor backdoor |
| **A misconfigured slab table producing a negative reward** | Monotonicity validation was offered and **declined** (Rule-41, ADR-009). An on-screen disclaimer stands in for a code guard |
| **A stolen or accessed unlocked machine** | Out of the threat model. Defended only by the inactivity timeout — physical security is the client's own responsibility |
