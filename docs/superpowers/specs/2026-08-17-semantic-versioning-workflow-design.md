# Semantic versioning workflow — design

Status: approved by user 2026-08-17
Branch: feat/semver-release-workflow

## Problem

No PR title convention is enforced before merge to `develop`. No automated
version bump / release exists — `package.json`'s `version` (propagated to
`src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` via
`scripts/sync-version.mjs`) is bumped by hand, if at all. No git tags exist.
No GitHub Releases exist.

## Constraints discovered (not assumptions)

- `develop` merges use merge-commit strategy (not squash) — verified via
  `git log --merges`. Individual commits inside feature branches already
  lean conventional-format by developer habit, not by enforcement.
- `main` branch protection requires: PR before merging, and passing status
  checks. **No direct push to `main` is possible for any actor, including
  a GitHub Actions bot.** Any version-bump commit must land via a PR.
- Existing release/branch flow (already established, unchanged by this
  work): feature branch → PR → `develop` (validated by `ci.yml`); user
  manually opens/merges the `develop → main` promotion PR when ready to
  ship.
- `scripts/sync-version.mjs` is the existing single-source-of-truth
  propagator: `package.json` → `tauri.conf.json` + `Cargo.toml`. Reused,
  not duplicated.
- Tauri `bundle.targets` is `"all"`, but current `ci.yml` only builds Rust
  on `ubuntu-latest`. No code-signing config exists for macOS/Windows.

## Decisions (from user, 2026-08-17)

| Question | Decision |
|---|---|
| When does version bump/release happen? | On merge to `main` only |
| PR title validation mechanism | `amannn/action-semantic-pull-request` |
| Major version signal | `!` suffix on type (`feat!:`, `fix!:`) |
| Release workflow scope | Version bump + tag + GitHub Release notes + build & attach Tauri installers |
| Installer platforms | macOS (.dmg/.app) and Windows (.msi/.exe) only — **not** Linux |
| Code signing | Unsigned for now; signing is a separate future sprint |
| Main branch protection | Confirmed: required PR + required status checks. No bot direct-push. |
| `develop → main` merge method | Confirmed: merge commit (not squash) |

## Workflow 1 — `validate-pr-title.yml`

Trigger: `pull_request` (`opened`, `edited`, `synchronize`) targeting `develop`.

Action: `amannn/action-semantic-pull-request`

```yaml
types:
  - feat
  - fix
  - perf
  - refactor
  - docs
  - test
  - style
  - build
  - ci
  - chore
  - revert
requireScope: false
```

- Minor bump: `feat`
- Patch bump: `fix`, `perf`, `refactor`, `docs`, `test`, `style`, `build`,
  `ci`, `chore`, `revert`
- Major bump: `!` after any type (e.g. `feat!:`, `fix!:`)
- Scope optional — matches existing repo history, where both `feat(x): ...`
  and `docs: ...` (no scope) already occur.

This must be added as a **required status check** on the `develop` branch
protection rule in GitHub settings — a manual step for the user, since it
can't be done without `gh auth login` in this session.

## Workflow 2 — `release.yml`

Because `main` cannot take a direct push, version bump + tag + release use
`googleapis/release-please-action`, which is built around exactly this
constraint: it never pushes to the tracked branch directly, it maintains a
release PR and only tags/releases once that PR is merged by a human.

Sequence:

1. User merges the `develop → main` promotion PR (existing process,
   unchanged by this work).
2. That push to `main` triggers `release.yml`. `release-please-action` walks
   conventional commits since the last tag, computes the next version using
   the same feat=minor / fix-etc=patch / `!`=major rules as Workflow 1, and
   opens or updates a PR (e.g. `chore(main): release v0.2.0`) with a
   generated CHANGELOG entry and bumped `package.json`.
3. A follow-up step pushes an additional commit onto that release PR's
   branch running `npm run sync-version`, so `Cargo.toml` and
   `tauri.conf.json` stay in lockstep with `package.json` — reusing the
   existing script rather than re-implementing version propagation in
   release-please config.
4. User reviews and merges the release PR (second manual click — required
   by main's branch protection, not avoidable).
5. That merge re-triggers `release.yml`; `release-please-action` detects
   the release PR was just merged, creates the git tag and GitHub Release
   with auto-generated notes.
6. Conditioned on that release being created in the same run:
   `tauri-apps/tauri-action` builds macOS and Windows installers (matrix:
   `macos-latest`, `windows-latest`) and attaches them to the GitHub
   Release. Unsigned — no signing secrets configured.

## Known limitation (flagged, not hidden)

`release-please` reads every commit on `main` since the last tag, not only
validated PR titles. Because `develop` merges preserve full commit history
(no squash), a stray non-conventional commit inside a feature branch is
simply ignored (contributes no bump) — harmless, but it means the title
gate covers the PR title, not literally every commit underneath it. Making
that airtight would require a squash-merge policy change on `develop`,
which is out of scope for this work.

## Files touched

- `.github/workflows/validate-pr-title.yml` (new)
- `.github/workflows/release.yml` (new)
- `release-please-config.json` (new)
- `.release-please-manifest.json` (new)
- No changes to existing `.github/workflows/ci.yml`

## Manual follow-up required from user (outside this repo's code)

- Add "validate-pr-title" as a required status check on `develop` branch
  protection.
- `develop → main` promotion PRs already use merge-commit strategy
  (confirmed by user) — no change needed, `release-please` will see the
  full commit graph.
