# Back-Navigation Breadcrumbs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the prototype's back-link/breadcrumb navigation (dynamic "came from" history + Structure's ancestor trail) onto the Structure, Member Detail, and Volume Entry screens.

**Architecture:** One new backend command (`get_ancestor_chain`, API-42) supplies Structure's root-to-current ancestor trail. One new frontend context (`NavigationHistoryProvider`) observes every route transition generically and lets any screen register its own display label (`useRouteLabel`) and read what to show as "back to X" (`useBackTarget`) — no per-navigation-call-site wiring needed elsewhere in the app. A shared `Breadcrumb` component renders the result identically across the three screens.

**Tech Stack:** Rust/rusqlite/Tauri (backend), React 19 + react-router 8 + Vitest/React Testing Library (frontend), WebdriverIO + Tauri Driver (e2e).

**Spec:** `docs/superpowers/specs/2026-08-14-back-navigation-breadcrumbs-design.md`

## Global Constraints

- No business rule, calculation, or data-model change — this is UX-only (per the spec's framing and the user's explicit instruction).
- Every commit is authored solely by the user — no co-author trailer of any kind (`[[feedback_no_coauthor]]`).
- Work happens on `feature/back-navigation-breadcrumbs`, branched off `develop`. Never commit directly to `develop` or `main`; the user merges `develop` → `main` manually.
- `src-tauri/src/command_names.rs`'s `ALL_COMMAND_NAMES` is a closed, numbered list (API-01 to API-41 today) that three places must agree on exactly: the Rust command surface, `capabilities/default.json`'s allow-list, and `documents/implementation-readiness/04-api-specification.md`. Adding `get_ancestor_chain` makes it 42 — every one of those three, plus the doc files that state the count, must move together or `src-tauri/tests/contract.rs`'s guard tests fail.
- Frontend IPC commands are always exposed as a plain function from the matching `src/lib/ipc/m*-*.ts` module and re-exported through `src/lib/ipc/index.ts`'s `export * as` — never call `invoke`/`invokeCommand` directly from a screen.
- Every existing test (`npm run test`, `cargo test` from `src-tauri/`) must stay green throughout — this includes the two closed-set guard tests in `contract.rs` and the count assertion in `src/lib/ipc/wrappers.test.ts`.

---

### Task 1: Backend — `get_ancestor_chain` (API-42)

**Files:**
- Modify: `src-tauri/src/m4_search/mod.rs` (domain function + types + unit tests)
- Modify: `src-tauri/src/commands.rs:238` (thin command wrapper, inserted after `get_direct_children_chart`)
- Modify: `src-tauri/src/command_names.rs:21` (register in `ALL_COMMAND_NAMES`)
- Modify: `src-tauri/src/lib.rs:53` (register in `invoke_handler`)
- Modify: `src-tauri/capabilities/default.json:21` (add `allow-get-ancestor-chain`)
- Modify: `src-tauri/tests/contract.rs` (rename+bump the closed-surface guard test, add two new tests)
- Modify: `documents/implementation-readiness/04-api-specification.md:60,141`
- Modify: `documents/implementation-readiness/08-testing-strategy.md:55`
- Modify: `documents/implementation-readiness/12-implementation-context.md:37`
- Modify: `documents/refinement/00-master-index.md:73`
- Modify: `documents/refinement/04-technical-architecture.md:503,529,608` (heading, table row, command index)
- Modify: `documents/refinement/05-quality-and-acceptance.md:163`
- Modify: `documents/refinement/06-decision-log-and-open-items.md` (C2 table + new dated decision entry)

**Interfaces:**
- Produces: `pub fn get_ancestor_chain(conn: &Connection, member_id: i64) -> Result<AncestorChainResult, AppError>` in `m4_search` — root-first, the requested member last. `AncestorChainResult { pub chain: Vec<AncestorNode> }`, `AncestorNode { pub id: i64, pub name: String }` (both `#[derive(Debug, Serialize)]`, `#[serde(rename_all = "camelCase")]`).
- Produces: `#[tauri::command] pub fn get_ancestor_chain(session, db, member_id: i64) -> Result<m4_search::AncestorChainResult, AppError>` in `commands.rs`, callable as IPC command `"get_ancestor_chain"`.

- [ ] **Step 1: Write the failing Rust unit tests**

Add to the `#[cfg(test)] mod tests` block at the bottom of `src-tauri/src/m4_search/mod.rs` (it already has `seeded()` and `insert_member()` helpers — reuse them):

```rust
    #[test]
    fn get_ancestor_chain_is_root_first_and_includes_the_member_itself() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let child = insert_member(&conn, "Child", Some(root));
        let grandchild = insert_member(&conn, "Grandchild", Some(child));

        let result = get_ancestor_chain(&conn, grandchild).unwrap();
        let names: Vec<&str> = result.chain.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["Root", "Child", "Grandchild"]);
        assert_eq!(result.chain.last().unwrap().id, grandchild);
    }

    #[test]
    fn get_ancestor_chain_for_the_root_member_is_a_single_entry() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let result = get_ancestor_chain(&conn, root).unwrap();
        assert_eq!(result.chain.len(), 1);
        assert_eq!(result.chain[0].id, root);
    }

    #[test]
    fn get_ancestor_chain_refuses_an_unknown_member() {
        let conn = seeded();
        let err = get_ancestor_chain(&conn, 999_999).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    // Rule-32: exceeding the configured max depth only warns, it never
    // blocks onboarding — the chain walk must not assume any bound.
    #[test]
    fn get_ancestor_chain_handles_a_chain_deeper_than_the_advisory_max_depth() {
        let conn = seeded();
        let mut parent = insert_member(&conn, "L0", None);
        for i in 1..=30 {
            parent = insert_member(&conn, &format!("L{i}"), Some(parent));
        }
        let result = get_ancestor_chain(&conn, parent).unwrap();
        assert_eq!(result.chain.len(), 31);
        assert_eq!(result.chain[0].name, "L0");
        assert_eq!(result.chain.last().unwrap().name, "L30");
    }
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run (from `src-tauri/`): `cargo test get_ancestor_chain`
Expected: compile error, `get_ancestor_chain` not found in this scope.

- [ ] **Step 3: Implement the domain function**

Add to `src-tauri/src/m4_search/mod.rs`, directly after the `get_direct_children_chart` function (after line 443, before the `#[cfg(test)]` block):

```rust
// ---------------------------------------------------------------------
// API-42 — get_ancestor_chain (Structure screen's breadcrumb trail)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AncestorNode {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AncestorChainResult {
    /// Root-first, the requested member last — ancestorTrail()'s ordering
    /// in the prototype (ui-prototype-v2.html:626-630).
    pub chain: Vec<AncestorNode>,
}

fn introducer_of(conn: &Connection, member_id: i64) -> Result<Option<i64>, AppError> {
    conn.query_row(
        "SELECT introducer_member_id FROM members WHERE id = ?1",
        [member_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Member not found.".into(),
    })
}

/// Leaf(the requested member)-to-root walk — same upward-loop idiom as
/// `m3_calc::chain_to_root`, duplicated locally rather than shared across
/// the module boundary (this module already keeps its own small
/// single-row-lookup helpers, e.g. `member_exists`, rather than reaching
/// into `m3_calc`'s private ones).
fn ancestor_chain_ids(conn: &Connection, member_id: i64) -> Result<Vec<i64>, AppError> {
    let mut chain = vec![member_id];
    let mut current = member_id;
    while let Some(parent) = introducer_of(conn, current)? {
        chain.push(parent);
        current = parent;
    }
    Ok(chain)
}

/// API-42. Cost scales with chain *depth* (indexed primary-key point
/// lookups, in-process SQLite), not with total member count — see the
/// design spec §2 for the worst-case analysis.
pub fn get_ancestor_chain(conn: &Connection, member_id: i64) -> Result<AncestorChainResult, AppError> {
    let ids = ancestor_chain_ids(conn, member_id)?;
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT id, name FROM members WHERE id IN ({placeholders})");
    let mut stmt = conn.prepare(&sql)?;
    let mut by_id: std::collections::HashMap<i64, String> = stmt
        .query_map(rusqlite::params_from_iter(ids.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<_, _>>()?;
    let chain = ids
        .into_iter()
        .rev()
        .map(|id| AncestorNode {
            name: by_id
                .remove(&id)
                .expect("id came from the members table, so a row must exist"),
            id,
        })
        .collect();
    Ok(AncestorChainResult { chain })
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test get_ancestor_chain`
Expected: all 4 new tests PASS.

- [ ] **Step 5: Wire the command through the standard pipeline**

In `src-tauri/src/commands.rs`, immediately after the closing `}` of `get_direct_children_chart` (line 238):

```rust
/// API-42. Root-first ancestor path (member itself last) for the Structure
/// screen's breadcrumb trail.
#[tauri::command]
pub fn get_ancestor_chain(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    member_id: i64,
) -> Result<m4_search::AncestorChainResult, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m4_search::get_ancestor_chain(conn, member_id)
}
```

In `src-tauri/src/command_names.rs`, in `ALL_COMMAND_NAMES`, immediately after `"get_direct_children_chart",` (line 21):

```rust
    "get_ancestor_chain",
```

In `src-tauri/src/lib.rs`, in the `generate_handler!` list, immediately after `commands::get_direct_children_chart,` (line 53):

```rust
            commands::get_ancestor_chain,
```

In `src-tauri/capabilities/default.json`, in `"permissions"`, immediately after `"allow-get-direct-children-chart",` (line 21):

```json
    "allow-get-ancestor-chain",
```

- [ ] **Step 6: Update the contract-test guard suite**

In `src-tauri/tests/contract.rs`, rename and update the closed-surface test (currently at line 122-145):

```rust
#[test]
fn the_command_surface_holds_exactly_forty_two_commands() {
    assert_eq!(
        ALL_COMMAND_NAMES.len(),
        42,
        "API-01 to API-42, no gaps (C2)"
    );

    let capabilities = include_str!("../capabilities/default.json");
    let allow_count = capabilities.matches("\"allow-").count();
    assert_eq!(
        allow_count, 42,
        "the Tauri capability allowlist must have exactly 42 allow-* entries"
    );
    for name in ALL_COMMAND_NAMES {
        let slug = name.replace('_', "-");
        assert!(
            capabilities.contains(&format!("\"allow-{slug}\"")),
            "capabilities/default.json is missing allow-{slug} (for command {name})"
        );
    }
}
```

Immediately after `get_direct_children_chart_requires_a_session` (after line 396), add:

```rust
#[test]
fn get_ancestor_chain_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::get_ancestor_chain(app.state::<SessionState>(), app.state::<DbState>(), 1);
    assert!(matches!(result, Err(AppError::AuthRequired)));
}
```

Immediately after `get_member_detail_and_get_direct_children_chart_end_to_end_through_the_command_layer` (after line 430), add:

```rust
#[test]
fn get_ancestor_chain_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876599906"),
    )
    .unwrap();

    let result = commands::get_ancestor_chain(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root.id,
    )
    .unwrap();
    assert_eq!(result.chain.len(), 1);
    assert_eq!(result.chain[0].id, root.id);
}
```

- [ ] **Step 7: Update the spec-of-record and cross-reference docs**

These are mechanical count/row updates, following exactly the pattern the prior API-41 addition used (commit `857e9c1`). Each is a small, independent edit:

`documents/implementation-readiness/04-api-specification.md` — insert a new table row immediately after the API-11 row (line 60), inside the `## Module M4` table:

```
| API-42 | `get_ancestor_chain` | **[ADDED 14 Aug 2026]** Root-to-member ancestor path for the Structure screen's breadcrumb trail — every ancestor of the requested member, root first, the member itself last | Admin | Authenticated | member_id | `{chain: [{id, name}, ...]}` | member_id must exist | — | 200-equivalent | Not found → refused | Read-only | None | Not audited |
```

Then update the tally at line 141: change `- **41 commands total** —` to `- **42 commands total** —`, and append to the end of that sentence (before the closing period): `, and \`get_ancestor_chain\` (API-42) added 14 August 2026 for the Structure/Member Detail/Volume Entry back-navigation breadcrumbs`.

`documents/implementation-readiness/08-testing-strategy.md:55` — change `there are 41 commands` to `there are 42 commands`.

`documents/implementation-readiness/12-implementation-context.md:37` — change `41-command IPC surface` to `42-command IPC surface`.

`documents/refinement/00-master-index.md:73` — change `API-01 … API-41 | **41**` to `API-01 … API-42 | **42**`.

`documents/refinement/04-technical-architecture.md`:
- Line 503 (section heading area — both the `## 6.` heading and the paragraph right under it): `41 Tauri IPC commands` → `42 Tauri IPC commands`, and `**41 commands total** — API-01 to API-41, no gaps. See [06](06-decision-log-and-open-items.md) C2. \`reverse_entry\` was removed (dead, confirmed) from the original 26-command count; \`list_period_entries\` (API-41) was added 13 August 2026.` → same text with `**42 commands total** — API-01 to API-42` and appending `; \`get_ancestor_chain\` (API-42) was added 14 August 2026.`
- After the API-11 row (line 542), insert: `| API-42 | \`get_ancestor_chain\` | Root-to-member ancestor path for the Structure screen's breadcrumb trail, root first, member last | Auth | member_id must exist | Read-only | Not audited |`
- Line 608 (`### 6.1 Command index by ID`): change `` `API-10`–`API-11` M4 `` to `` `API-10`–`API-11`, `API-42` M4 ``.

`documents/refinement/05-quality-and-acceptance.md:163` — change `**API / contract — the 41-command IPC surface**` to `**API / contract — the 42-command IPC surface**`.

`documents/refinement/06-decision-log-and-open-items.md` — update the C2 table (the `| **Resolution** |`, `| **Authority** |`, `| **Build consequence** |` rows) from 41 to 42 and the date to 14 Aug 2026, matching the exact edit shape the 13 Aug 2026 amendment used. Then add a new dated section, mirroring the existing "### 13 August 2026 — Volume Entry period table and summary nodes" entry's format, immediately after it:

```markdown
### 14 August 2026 — Back-navigation breadcrumbs (Structure / Member Detail / Volume Entry)

| | |
|---|---|
| **Requested** | Structure, Member Detail, and Volume Entry need the back-link/breadcrumb navigation the client-approved prototype already ships: a dynamic "back to whatever screen you came from" link on all three, plus Structure's root-to-current ancestor trail and Member Detail's fixed Home crumb |
| **Which command** | Structure's ancestor trail needs a root-to-member path that no existing command returns (`get_direct_children_chart` only walks downward) |
| **Decided** | **API-42 `get_ancestor_chain`** added — the closed 41-command surface (C2) becomes 42. Returns the ancestor path root-first, the requested member last; the back-link labels themselves are computed client-side from navigation history, no backend involvement |
| **Rule** | No new business rule — a read-only structural lookup, same status as `get_direct_children_chart` (API-11) |
```

- [ ] **Step 8: Run the full backend test suite**

Run (from `src-tauri/`): `cargo test`
Expected: all tests PASS, including `the_command_surface_holds_exactly_forty_two_commands`, `the_unauthenticated_set_is_exactly_the_named_seven` (unaffected — `get_ancestor_chain` is authenticated), and the new `get_ancestor_chain_*` tests.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/m4_search/mod.rs src-tauri/src/commands.rs src-tauri/src/command_names.rs \
  src-tauri/src/lib.rs src-tauri/capabilities/default.json src-tauri/tests/contract.rs \
  documents/implementation-readiness/04-api-specification.md \
  documents/implementation-readiness/08-testing-strategy.md \
  documents/implementation-readiness/12-implementation-context.md \
  documents/refinement/00-master-index.md \
  documents/refinement/04-technical-architecture.md \
  documents/refinement/05-quality-and-acceptance.md \
  documents/refinement/06-decision-log-and-open-items.md
git commit -m "feat(M4): add get_ancestor_chain (API-42) for Structure's breadcrumb trail

Root-to-member ancestor path, needed so the Structure screen can render
the client-approved prototype's breadcrumb trail. Command surface goes
41->42; docs, contract tests, and the capability allowlist updated to
match."
```

---

### Task 2: Frontend IPC binding

**Files:**
- Modify: `src/lib/ipc/m4-search.ts`
- Modify: `src/lib/ipc/wrappers.test.ts`

**Interfaces:**
- Consumes: IPC command `"get_ancestor_chain"` (Task 1), payload `{ memberId: number }`, response `{ chain: AncestorNode[] }`.
- Produces: `export interface AncestorNode { id: number; name: string }`, `export function getAncestorChain(memberId: number): Promise<{ chain: AncestorNode[] }>` from `src/lib/ipc/m4-search.ts` — consumed by Task 6 (Structure screen).

- [ ] **Step 1: Write the failing test**

In `src/lib/ipc/wrappers.test.ts`, update the count test (it currently asserts 41):

```typescript
  it("expose exactly 42 command functions, API-01 to API-42 with no gaps", () => {
    const modules = [
      ipc.m1Members,
      ipc.m2Entries,
      ipc.m3Calc,
      ipc.m4Search,
      ipc.m5Close,
      ipc.m6Reports,
      ipc.m7Settings,
      ipc.m8Auth,
      ipc.m9Audit,
      ipc.preflight,
    ];
    const commandFns = modules.flatMap((mod) =>
      Object.values(mod).filter((value): value is (...args: never[]) => unknown => {
        return typeof value === "function";
      }),
    );
    expect(commandFns).toHaveLength(42);
  });
```

(Replace the `41` in both the test title and `toHaveLength(41)`.)

- [ ] **Step 2: Run the test to verify it fails**

Run: `npx vitest run src/lib/ipc/wrappers.test.ts`
Expected: FAIL — `commandFns` still has length 41, expected 42.

- [ ] **Step 3: Add the binding**

In `src/lib/ipc/m4-search.ts`, append:

```typescript
export interface AncestorNode {
  id: number;
  name: string;
}

export interface AncestorChainResult {
  /** Root-first, the requested member last. */
  chain: AncestorNode[];
}

// API-42 — the Structure screen's breadcrumb trail.
export function getAncestorChain(memberId: number): Promise<AncestorChainResult> {
  return invokeCommand("get_ancestor_chain", { memberId });
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npx vitest run src/lib/ipc/wrappers.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/ipc/m4-search.ts src/lib/ipc/wrappers.test.ts
git commit -m "feat(ipc): add getAncestorChain frontend binding for API-42"
```

---

### Task 3: Navigation-history context (`useRouteLabel` / `useBackTarget`)

**Files:**
- Create: `src/lib/navigation-history.tsx`
- Create: `src/lib/navigation-history.test.tsx`

**Interfaces:**
- Produces: `export function NavigationHistoryProvider({ children }: { children: ReactNode })`, `export function useRouteLabel(label: string | undefined): void`, `export function useBackTarget(): { label: string; hasHistory: boolean; go: () => void }` — consumed by Task 4 (provider mount) and Tasks 6-8 (screens).

- [ ] **Step 1: Write the failing test**

Create `src/lib/navigation-history.test.tsx`:

```tsx
// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route, useNavigate } from "react-router";

import { NavigationHistoryProvider, useRouteLabel, useBackTarget } from "./navigation-history";

function Home() {
  useRouteLabel("Home");
  const navigate = useNavigate();
  return (
    <div>
      <button onClick={() => navigate("/detail")}>Open detail</button>
      <button onClick={() => navigate("/detail-b")}>Open detail B</button>
    </div>
  );
}

function Detail() {
  useRouteLabel("Detail");
  const { label, go } = useBackTarget();
  return <button onClick={go}>Back to {label}</button>;
}

function DetailB() {
  useRouteLabel("Detail B");
  const { label, go } = useBackTarget();
  const navigate = useNavigate();
  return (
    <div>
      <button onClick={() => navigate("/detail-b-2", { replace: true })}>Rename</button>
      <button onClick={go}>Back to {label}</button>
    </div>
  );
}

function DetailB2() {
  useRouteLabel("Detail B renamed");
  const { label, go } = useBackTarget();
  return <button onClick={go}>Back to {label}</button>;
}

function Reports() {
  const { label } = useBackTarget();
  return <span>Back to {label}</span>;
}

function harness(initialPath: string) {
  return render(
    <MemoryRouter initialEntries={[initialPath]}>
      <NavigationHistoryProvider>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/detail" element={<Detail />} />
          <Route path="/detail-b" element={<DetailB />} />
          <Route path="/detail-b-2" element={<DetailB2 />} />
          <Route path="/reports" element={<Reports />} />
        </Routes>
      </NavigationHistoryProvider>
    </MemoryRouter>,
  );
}

describe("navigation-history", () => {
  it("labels the back link with the screen that pushed the navigation", async () => {
    harness("/");
    fireEvent.click(screen.getByText("Open detail"));
    expect(await screen.findByText("Back to Home")).toBeInTheDocument();
  });

  it("pops back to the previous screen when the back link is clicked", async () => {
    harness("/");
    fireEvent.click(screen.getByText("Open detail"));
    fireEvent.click(await screen.findByText("Back to Home"));
    expect(await screen.findByText("Open detail")).toBeInTheDocument();
  });

  it("falls back to the static route label when nothing has been pushed yet", () => {
    harness("/reports");
    expect(screen.getByText("Back to Home")).toBeInTheDocument();
  });

  it("does not disturb the back target on a replace navigation", async () => {
    harness("/");
    fireEvent.click(screen.getByText("Open detail B"));
    fireEvent.click(await screen.findByText("Rename"));
    // still on the replaced screen, back target is still Home (the PUSH
    // that got us to /detail-b in the first place), not /detail-b itself.
    expect(await screen.findByText("Back to Home")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npx vitest run src/lib/navigation-history.test.tsx`
Expected: FAIL — module `./navigation-history` does not exist.

- [ ] **Step 3: Implement the provider and hooks**

Create `src/lib/navigation-history.tsx`:

```tsx
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useLocation, useNavigate, useNavigationType } from "react-router";

interface StackEntry {
  path: string;
  label: string;
}

interface NavigationHistoryContextValue {
  registerLabel: (path: string, label: string) => void;
  stack: StackEntry[];
}

const NavigationHistoryContext = createContext<NavigationHistoryContextValue | null>(null);

// Mirrors the prototype's ROUTE_LABELS (ui-prototype-v2.html:1309) — the
// fallback for any screen that never calls useRouteLabel itself.
const STATIC_ROUTE_LABELS: Record<string, string> = {
  "/": "Home",
  "/structure": "Structure",
  "/entry": "Volume entry",
  "/entry/correct": "Volume entry",
  "/close": "Monthly close",
  "/reports": "Reports",
  "/audit": "Audit log",
  "/settings": "Settings",
};

function fallbackLabel(path: string): string {
  return STATIC_ROUTE_LABELS[path] ?? "Home";
}

/**
 * Observes every route transition and keeps a small stack of {path, label}
 * for whatever screen was left behind on each PUSH — this is what lets
 * `useBackTarget()` on the *next* screen say "Back to <real previous
 * screen>" without that screen having to pass anything explicitly. REPLACE
 * (Structure's ancestor re-root) leaves the stack untouched, matching the
 * prototype's own `{replace:true}` behaviour. POP (the back link itself,
 * or a real browser-back) consumes one level.
 */
export function NavigationHistoryProvider({ children }: { children: ReactNode }) {
  const location = useLocation();
  const navigationType = useNavigationType();
  const [stack, setStack] = useState<StackEntry[]>([]);
  const labelsRef = useRef<Map<string, string>>(new Map());
  const prevPathRef = useRef<string | null>(null);

  useEffect(() => {
    const path = location.pathname;
    const prevPath = prevPathRef.current;
    if (prevPath !== null && prevPath !== path) {
      if (navigationType === "PUSH") {
        const label = labelsRef.current.get(prevPath) ?? fallbackLabel(prevPath);
        setStack((s) => [...s, { path: prevPath, label }]);
      } else if (navigationType === "POP") {
        setStack((s) => s.slice(0, -1));
      }
      // REPLACE: stack stays as-is.
    }
    prevPathRef.current = path;
  }, [location.pathname, navigationType]);

  function registerLabel(path: string, label: string) {
    labelsRef.current.set(path, label);
  }

  return (
    <NavigationHistoryContext.Provider value={{ registerLabel, stack }}>
      {children}
    </NavigationHistoryContext.Provider>
  );
}

function useNavigationHistory(): NavigationHistoryContextValue {
  const ctx = useContext(NavigationHistoryContext);
  if (!ctx) {
    throw new Error("useNavigationHistory must be used within a NavigationHistoryProvider");
  }
  return ctx;
}

/** Screens call this with their own current display identity (a member's
 *  name, "Structure (<root name>)", a static string) so that whatever
 *  screen the user navigates to next can show an accurate "Back to X".
 *  `undefined` (still loading) leaves whatever was registered before. */
export function useRouteLabel(label: string | undefined): void {
  const { registerLabel } = useNavigationHistory();
  const location = useLocation();
  useEffect(() => {
    if (label) registerLabel(location.pathname, label);
  }, [label, location.pathname, registerLabel]);
}

/** `hasHistory` is false only when nothing has been pushed onto the stack
 *  this session (a fresh load / deep link) — Structure hides its back link
 *  in that case; Member Detail and Volume Entry show it anyway, since
 *  `label` already defaults to "Home", a real, always-valid destination. */
export function useBackTarget(): { label: string; hasHistory: boolean; go: () => void } {
  const { stack } = useNavigationHistory();
  const navigate = useNavigate();
  const top = stack[stack.length - 1];
  return {
    label: top?.label ?? "Home",
    hasHistory: !!top,
    go: () => (top ? navigate(-1) : navigate("/")),
  };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npx vitest run src/lib/navigation-history.test.tsx`
Expected: all 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/navigation-history.tsx src/lib/navigation-history.test.tsx
git commit -m "feat(nav): add NavigationHistoryProvider for dynamic back-link labels"
```

---

### Task 4: Mount the provider in the app shell

**Files:**
- Modify: `src/components/app-shell.tsx`

**Interfaces:**
- Consumes: `NavigationHistoryProvider` from `src/lib/navigation-history.tsx` (Task 3).

- [ ] **Step 1: Wrap the routed content**

In `src/components/app-shell.tsx`, import the provider and wrap `<Outlet/>` — the whole `AppShellLayout` function body, since `useLocation`/`useNavigationType` inside the provider need to sit above the routed screens but the provider itself needs `useLocation`, which requires being inside the `<RouterProvider>` tree (it already is, since `AppShell` is itself a route element). Add the import:

```tsx
import { NavigationHistoryProvider } from "@/lib/navigation-history";
```

Then wrap only the `<main>` block's `<Outlet/>` (leaving the sidebar/banner untouched, since neither reads breadcrumb state):

```tsx
        <main ref={mainRef} className="flex-1 overflow-y-auto px-8 pb-10 pt-5">
          <NavigationHistoryProvider>
            <Outlet />
          </NavigationHistoryProvider>
        </main>
```

- [ ] **Step 2: Run the existing app-shell-adjacent tests to confirm nothing broke**

Run: `npx vitest run src/components/outstanding-month-banner.test.tsx`
Expected: PASS (unrelated to this change, but exercises the same shell tree shape — a quick regression check since `app-shell.tsx` has no dedicated test file of its own).

- [ ] **Step 3: Commit**

```bash
git add src/components/app-shell.tsx
git commit -m "feat(nav): mount NavigationHistoryProvider around the routed outlet"
```

---

### Task 5: `Breadcrumb` component + `PageHeader` slot

**Files:**
- Create: `src/components/breadcrumb.tsx`
- Create: `src/components/breadcrumb.test.tsx`
- Modify: `src/components/page-header.tsx`

**Interfaces:**
- Produces: `export interface BreadcrumbCrumb { label: string; to?: string; replace?: boolean }`, `export function Breadcrumb({ backLabel, onBack, crumbs }: { backLabel?: string; onBack: () => void; crumbs: BreadcrumbCrumb[] })` — consumed by Tasks 6-8.
- Modifies: `PageHeader`'s props to accept an optional `breadcrumb?: ReactNode`, rendered above the title row.

- [ ] **Step 1: Write the failing test**

Create `src/components/breadcrumb.test.tsx`:

```tsx
import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";

import { Breadcrumb } from "./breadcrumb";

describe("Breadcrumb", () => {
  it("renders nothing when there is no back label and no crumbs", () => {
    const { container } = render(
      <MemoryRouter>
        <Breadcrumb backLabel={undefined} onBack={() => {}} crumbs={[]} />
      </MemoryRouter>,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("renders the back link and calls onBack when clicked", () => {
    const onBack = vi.fn();
    render(
      <MemoryRouter>
        <Breadcrumb backLabel="Home" onBack={onBack} crumbs={[]} />
      </MemoryRouter>,
    );
    screen.getByText("Back to Home").click();
    expect(onBack).toHaveBeenCalled();
  });

  it("renders every crumb, only the last one non-clickable", () => {
    render(
      <MemoryRouter>
        <Breadcrumb
          backLabel={undefined}
          onBack={() => {}}
          crumbs={[
            { label: "Root", to: "/structure/1" },
            { label: "Child", to: "/structure/2" },
            { label: "Grandchild" },
          ]}
        />
      </MemoryRouter>,
    );
    expect(screen.getAllByRole("link")).toHaveLength(2);
    expect(screen.getByText("Grandchild").closest("a")).toBeNull();
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npx vitest run src/components/breadcrumb.test.tsx`
Expected: FAIL — module `./breadcrumb` does not exist.

- [ ] **Step 3: Implement the component**

Create `src/components/breadcrumb.tsx`:

```tsx
import { ChevronLeft } from "lucide-react";
import { Link } from "react-router";

export interface BreadcrumbCrumb {
  label: string;
  /** Omit for the current, non-clickable crumb. */
  to?: string;
  /** Structure's ancestor-trail crumbs re-root via `replace` so the back
   *  target doesn't change as the user moves within the same trail. */
  replace?: boolean;
}

export function Breadcrumb({
  backLabel,
  onBack,
  crumbs,
}: {
  backLabel?: string;
  onBack: () => void;
  crumbs: BreadcrumbCrumb[];
}) {
  if (!backLabel && crumbs.length === 0) return null;

  return (
    <div className="mb-1.5 flex flex-wrap items-center gap-1.5 text-caption text-muted-text">
      {backLabel && (
        <>
          <button
            type="button"
            onClick={onBack}
            className="inline-flex items-center gap-0.5 hover:text-accent"
          >
            <ChevronLeft className="size-3.5" />
            Back to {backLabel}
          </button>
          {crumbs.length > 0 && <span className="opacity-50">/</span>}
        </>
      )}
      {crumbs.map((c, i) => (
        <span key={i} className="inline-flex items-center gap-1.5">
          {i > 0 && <span className="opacity-50">/</span>}
          {c.to ? (
            <Link to={c.to} replace={c.replace} className="hover:text-accent">
              {c.label}
            </Link>
          ) : (
            <span className="font-semibold text-ink">{c.label}</span>
          )}
        </span>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npx vitest run src/components/breadcrumb.test.tsx`
Expected: all 3 tests PASS.

- [ ] **Step 5: Add the `breadcrumb` slot to `PageHeader`**

In `src/components/page-header.tsx`, add the prop and render it above the title row:

```tsx
import type { ReactNode } from "react";

export function PageHeader({
  title,
  subtitle,
  actions,
  breadcrumb,
}: {
  title: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  breadcrumb?: ReactNode;
}) {
  return (
    <div className="sticky -top-5 z-10 -mx-8 -mt-5 mb-10 bg-background px-8 pb-2 pt-8">
      {breadcrumb}
      <div className="flex items-center justify-between">
        <h1 className="text-headline">{title}</h1>
        {actions}
      </div>
      {subtitle && <p className="text-caption mt-1">{subtitle}</p>}
    </div>
  );
}
```

- [ ] **Step 6: Commit**

```bash
git add src/components/breadcrumb.tsx src/components/breadcrumb.test.tsx src/components/page-header.tsx
git commit -m "feat(nav): add Breadcrumb component and PageHeader breadcrumb slot"
```

---

### Task 6: Structure screen — ancestor trail + back link

**Files:**
- Modify: `src/screens/structure.tsx`

**Interfaces:**
- Consumes: `getAncestorChain` (Task 2), `useRouteLabel`/`useBackTarget` (Task 3), `Breadcrumb`/`BreadcrumbCrumb` (Task 5).

- [ ] **Step 1: Add the imports**

In `src/screens/structure.tsx`, add:

```tsx
import { Breadcrumb, type BreadcrumbCrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";
import { getAncestorChain, type AncestorNode } from "@/lib/ipc/m4-search";
```

- [ ] **Step 2: Fetch the ancestor chain and register the route label**

Inside the `Structure()` component, alongside the other `useState`/`useEffect` declarations (after the `lockStatus`/`selectedMonth` block, before the "fetch children" effect):

```tsx
  const [ancestorChain, setAncestorChain] = useState<AncestorNode[]>([]);
  useEffect(() => {
    if (!rootNode) return;
    let cancelled = false;
    getAncestorChain(rootNode.memberId).then((result) => {
      if (!cancelled) setAncestorChain(result.chain);
    });
    return () => {
      cancelled = true;
    };
  }, [rootNode?.memberId]);

  const backTarget = useBackTarget();
  useRouteLabel(rootNode ? `Structure (${rootNode.name})` : undefined);
```

- [ ] **Step 3: Render the breadcrumb**

Replace the `<PageHeader title="Structure" subtitle="Open one branch at a time" />` line with:

```tsx
      <PageHeader
        title="Structure"
        subtitle="Open one branch at a time"
        breadcrumb={
          <Breadcrumb
            backLabel={backTarget.hasHistory ? backTarget.label : undefined}
            onBack={backTarget.go}
            crumbs={ancestorChain.map(
              (a, i): BreadcrumbCrumb =>
                i === ancestorChain.length - 1
                  ? { label: a.name }
                  : { label: a.name, to: `/structure/${a.id}`, replace: true },
            )}
          />
        }
      />
```

Note: this replaces the earlier `<PageHeader title="Structure" subtitle="Open one branch at a time" />` call (currently right after the opening `<div className="flex h-full min-h-0 flex-col">`) — the rest of the screen is unchanged.

- [ ] **Step 4: Manually verify in the dev app**

Run `npm run tauri dev` (or the project's existing dev-run flow), navigate: Home → search a member → Member Detail → "View in structure" → confirm the breadcrumb shows `← Back to <member name>` then the ancestor trail ending in the current member's name, and clicking an ancestor crumb re-roots the tree without changing the back link's target.

- [ ] **Step 5: Commit**

```bash
git add src/screens/structure.tsx
git commit -m "feat(structure): render ancestor-trail breadcrumb and dynamic back link"
```

---

### Task 7: Member Detail screen — Home crumb + back link

**Files:**
- Modify: `src/screens/member-detail.tsx`

**Interfaces:**
- Consumes: `useRouteLabel`/`useBackTarget` (Task 3), `Breadcrumb` (Task 5).

- [ ] **Step 1: Add the imports**

In `src/screens/member-detail.tsx`, add:

```tsx
import { Breadcrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";
```

- [ ] **Step 2: Register the route label and read the back target**

`MemberDetail` has two early returns — `if (error) return <EmptyState .../>;` then `if (!detail) return <LoadingState />;` — before it destructures `member` off `detail`. Hooks must run unconditionally on every render (React's Rules of Hooks: the same hooks, same order, every render — an early return before a hook call means that hook silently stops running on some renders and not others), so both new hooks go at the *top* of the component, alongside the other `useState` declarations, using `detail?.member.name` (safely `undefined` while still loading) rather than waiting for the later `const { member, rewards } = detail;` line:

```tsx
  const [busy, setBusy] = useState(false);

  const backTarget = useBackTarget();
  useRouteLabel(detail?.member.name);
```

(Insert these two lines immediately after the existing `const [busy, setBusy] = useState(false);` line, before the `lockStatus`/`selectedMonth` block.)

- [ ] **Step 3: Render the breadcrumb above the existing header card**

This screen has no `PageHeader` — its custom header card renders directly. Add the `<Breadcrumb>` immediately before it, inside the returned `<>...</>` fragment (right after the opening `<>`, before `<div className="rounded-lg border border-border bg-surface p-4.5">`). This part of the render only happens once `detail` is defined (past both early returns), so `member.name` is safe to use directly here even though the hook above used the optional-chained version:

```tsx
      <Breadcrumb
        backLabel={backTarget.label}
        onBack={backTarget.go}
        crumbs={[{ label: "Home", to: "/" }, { label: member.name }]}
      />
```

- [ ] **Step 4: Manually verify in the dev app**

Confirm: Member Detail always shows `← Back to <previous screen>` (defaulting to "Back to Home" on a fresh load with no navigation history) followed by `Home / <member name>`; clicking `Home` always lands on `/`.

- [ ] **Step 5: Commit**

```bash
git add src/screens/member-detail.tsx
git commit -m "feat(member-detail): render Home/back breadcrumb"
```

---

### Task 8: Volume Entry screen — back link

**Files:**
- Modify: `src/screens/business-volume-entry.tsx`

**Interfaces:**
- Consumes: `useRouteLabel`/`useBackTarget` (Task 3), `Breadcrumb` (Task 5).

- [ ] **Step 1: Add the imports**

In `src/screens/business-volume-entry.tsx`, add:

```tsx
import { Breadcrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";
```

- [ ] **Step 2: Register the route label and read the back target**

Inside `BusinessVolumeEntry()`, alongside the other top-level hook calls (near the `useToast()`/`bounds` declarations):

```tsx
  const backTarget = useBackTarget();
  useRouteLabel("Volume entry");
```

- [ ] **Step 3: Render the breadcrumb**

Change `<PageHeader title="Volume Entry" />` to:

```tsx
      <PageHeader
        title="Volume Entry"
        breadcrumb={<Breadcrumb backLabel={backTarget.label} onBack={backTarget.go} crumbs={[]} />}
      />
```

- [ ] **Step 4: Manually verify in the dev app**

Confirm: from Member Detail, clicking "Record volume" lands here showing `← Back to <that member's name>`, and clicking it returns to that exact Member Detail page. From the sidebar directly, it shows `← Back to Home` (or wherever was previously open).

- [ ] **Step 5: Commit**

```bash
git add src/screens/business-volume-entry.tsx
git commit -m "feat(volume-entry): render dynamic back link"
```

---

### Task 9: E2E coverage for the three navigation flows

**Files:**
- Modify: `e2e/specs/structure.e2e.js`

**Interfaces:**
- Consumes: `navigateTo`, `idOfPhone` from `e2e/helpers/seed.js` (`idOfPhone` is not currently imported in this file — add it).

This file already runs last (alphabetically last among `e2e/specs/*.e2e.js`) and its existing test reuses "Asha Patel" (phone `9876500002`), the member `business-volume-entry.e2e.js` onboards under "Root Member" — these new tests reuse the same shared session and member.

- [ ] **Step 1: Add the `idOfPhone` import**

Change the top of `e2e/specs/structure.e2e.js` from:

```js
import { navigateTo } from "../helpers/seed.js";
```

to:

```js
import { navigateTo, idOfPhone } from "../helpers/seed.js";
```

- [ ] **Step 2: Write the three flows**

Append to `e2e/specs/structure.e2e.js`, after the existing `describe("Structure — inactive-node treatment", ...)` block:

```js
// Back-navigation breadcrumbs (Structure / Member Detail / Volume Entry).
// Reuses "Root Member" and "Asha Patel" (phone 9876500002), onboarded by
// business-volume-entry.e2e.js earlier in this shared session — this file
// runs last, so both members and Asha's now-inactive state (the test
// above deactivates her without reactivating) are already in place.
describe("Back-navigation breadcrumbs", () => {
  it("Structure -> Member Detail -> back returns to Structure", async () => {
    const memberId = await idOfPhone("9876500002");
    await navigateTo("Structure");
    await browser.waitUntil(async () => (await browser.getUrl()).includes("/structure"), {
      timeout: 3000,
    });

    await $(`button[aria-label="View Asha Patel's member detail"]`).click();
    await browser.waitUntil(
      async () => (await browser.getUrl()).includes(`/member/${memberId}`),
      { timeout: 3000 },
    );

    const backLink = $("*=Back to Structure");
    await backLink.waitForExist({ timeout: 3000 });
    await backLink.click();

    await browser.waitUntil(async () => (await browser.getUrl()).includes("/structure"), {
      timeout: 3000,
    });
  });

  it("Member Detail's Home breadcrumb link navigates to Home", async () => {
    await idOfPhone("9876500002"); // lands on Asha Patel's Member Detail

    await $("main").$("a=Home").click();

    await browser.waitUntil(async () => (await browser.getUrl()).endsWith("/"), {
      timeout: 3000,
    });
    await expect($("#home-search")).toExist();
  });

  it("Volume Entry reached from Member Detail's Record Volume back-links to that member", async () => {
    await idOfPhone("9876500002"); // lands on Asha Patel's Member Detail

    await $("button=Record volume").click();
    await browser.waitUntil(async () => (await browser.getUrl()).includes("/entry"), {
      timeout: 3000,
    });

    const backLink = $("*=Back to Asha Patel");
    await backLink.waitForExist({ timeout: 3000 });
    await backLink.click();

    await browser.waitUntil(async () => (await browser.getUrl()).includes("/member/"), {
      timeout: 3000,
    });
    await expect($("*=Asha Patel")).toExist();
  });
});
```

- [ ] **Step 3: Run the e2e suite**

Run: `npm run test:e2e`
Expected: all specs PASS, including the three new tests in `structure.e2e.js`.

- [ ] **Step 4: Commit**

```bash
git add e2e/specs/structure.e2e.js
git commit -m "test(e2e): cover the three back-navigation breadcrumb flows"
```

---

## Final verification (after all 9 tasks)

- [ ] Run `cd src-tauri && cargo test` — full backend suite green.
- [ ] Run `npm run test` — full frontend unit suite green.
- [ ] Run `npm run lint` — no new lint errors.
- [ ] Run `npm run test:e2e` — full e2e suite green.
- [ ] Manually exercise all three flows from the spec's §1 one more time end-to-end in the running app.
