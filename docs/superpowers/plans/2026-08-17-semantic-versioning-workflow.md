# Semantic Versioning Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce a Conventional-Commits-style PR title on every PR into `develop`, and automatically compute/tag/release semver versions (with macOS + Windows Tauri installers attached) when `develop` is promoted to `main`.

**Architecture:** Two independent GitHub Actions workflows. `validate-pr-title.yml` gates PR titles into `develop` using `amannn/action-semantic-pull-request`. `release.yml` runs on push to `main` using `googleapis/release-please-action` to maintain a release PR (version bump + changelog), then on that PR's merge, tags + creates a GitHub Release, then builds and attaches installers via `tauri-apps/tauri-action`. Neither workflow ever pushes directly to `main` — that's structurally blocked by branch protection, so `release-please`'s PR-based model is used instead of custom scripting.

**Tech Stack:** GitHub Actions, `amannn/action-semantic-pull-request@v5`, `googleapis/release-please-action@v4`, `tauri-apps/tauri-action@v0`, existing `scripts/sync-version.mjs`, `js-yaml` (already a repo dependency, used here only for local YAML syntax checks).

**Spec:** [docs/superpowers/specs/2026-08-17-semantic-versioning-workflow-design.md](../specs/2026-08-17-semantic-versioning-workflow-design.md)

## Global Constraints

- PR title types: `feat` (minor), `fix`/`perf`/`refactor`/`docs`/`test`/`style`/`build`/`ci`/`chore`/`revert` (patch), any type with `!` suffix (major).
- `requireScope: false` — scope is optional on every PR title.
- No workflow may push a commit directly to `main` (branch protection: required PR + required status checks — confirmed, not assumed).
- Installer build platforms: macOS and Windows only. Explicitly **not** Linux (user's explicit choice).
- Installers ship unsigned. No signing secrets are configured in this plan.
- Reuse `scripts/sync-version.mjs` for propagating `package.json`'s version into `tauri.conf.json` and `Cargo.toml` — do not reimplement that logic in `release-please-config.json`.
- `develop → main` promotion PRs use merge-commit strategy (confirmed) — `release-please` relies on seeing the full commit graph on `main`.
- No changes to the existing `.github/workflows/ci.yml`.

---

## File Structure

- `.github/workflows/validate-pr-title.yml` — new. PR title gate for PRs into `develop`.
- `release-please-config.json` — new. Tells `release-please-action` this is a single Node-style package at repo root, with Conventional Commits bump rules.
- `.release-please-manifest.json` — new. Bootstraps `release-please`'s idea of the current version (`0.1.0`, matching `package.json`) so the first run doesn't miscompute history it can't see (no tags exist yet).
- `.github/workflows/release.yml` — new. Runs `release-please-action` on push to `main`; pushes a `sync-version` commit onto the open release PR; on release-PR merge, builds + attaches macOS/Windows installers.

---

### Task 1: PR title validation workflow

**Files:**
- Create: `.github/workflows/validate-pr-title.yml`

**Interfaces:**
- Produces: a GitHub Actions status check named `validate-pr-title` (job id), which the user will add as a required check on `develop`'s branch protection rule (manual GitHub UI step, outside this repo's code — call this out at the end of the task).

- [ ] **Step 1: Write the workflow file**

```yaml
name: Validate PR title

on:
  pull_request:
    types: [opened, edited, synchronize, reopened]
    branches: [develop]

permissions:
  pull-requests: read

jobs:
  validate-pr-title:
    name: validate-pr-title
    runs-on: ubuntu-latest
    steps:
      - uses: amannn/action-semantic-pull-request@v5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          types: |
            feat
            fix
            perf
            refactor
            docs
            test
            style
            build
            ci
            chore
            revert
          requireScope: false
          subjectPattern: ^(?![A-Z]).+$
          subjectPatternError: |
            The PR title's subject (after "type: " or "type(scope): ") must not start with an uppercase letter.
```

- [ ] **Step 2: Validate YAML syntax locally**

Run:
```bash
node -e "require('js-yaml').load(require('fs').readFileSync('.github/workflows/validate-pr-title.yml', 'utf8')); console.log('valid yaml')"
```
Expected: `valid yaml`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/validate-pr-title.yml
git commit -m "ci: add PR title validation workflow for develop"
```

- [ ] **Step 4: Push branch and open a throwaway test PR to prove the gate works**

```bash
git push -u origin feat/semver-release-workflow
gh pr create --base develop --head feat/semver-release-workflow --title "wip: semver release workflow" --body "Draft — validating PR title gate and release workflow. Do not merge yet." --draft
```

Expected: PR opens. Because this task's own PR title starts with `wip:` (not in the allowed `types` list), the `validate-pr-title` check should **fail** on this PR — that failure is itself the proof the gate works. Confirm the check shows red in the PR's checks tab.

- [ ] **Step 5: Retitle the PR to a valid conventional title and confirm the check goes green**

```bash
gh pr edit --title "ci: add semantic versioning workflow (PR title gate + release automation)"
```

Expected: `validate-pr-title` check re-runs and passes. Leave the PR open as a draft — later tasks add commits to this same branch/PR.

---

### Task 2: release-please configuration

**Files:**
- Create: `release-please-config.json`
- Create: `.release-please-manifest.json`

**Interfaces:**
- Consumes: `package.json`'s current `version` field (`0.1.0`, verified in Task 0 exploration).
- Produces: on-disk config that `googleapis/release-please-action` (Task 3) reads via `config-file: release-please-config.json` and `manifest-file: .release-please-manifest.json` inputs.

- [ ] **Step 1: Write the manifest, bootstrapped to the current version**

```json
{
  ".": "0.1.0"
}
```

Save as `.release-please-manifest.json`.

- [ ] **Step 2: Write the config**

```json
{
  "$schema": "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json",
  "release-type": "node",
  "packages": {
    ".": {
      "package-name": "bvconsole"
    }
  },
  "pull-request-title-pattern": "chore(main): release${component} v${version}",
  "changelog-sections": [
    { "type": "feat", "section": "Features" },
    { "type": "fix", "section": "Bug Fixes" },
    { "type": "perf", "section": "Performance" },
    { "type": "revert", "section": "Reverts" },
    { "type": "refactor", "section": "Code Refactoring" },
    { "type": "docs", "section": "Documentation" },
    { "type": "test", "section": "Tests", "hidden": true },
    { "type": "style", "section": "Styles", "hidden": true },
    { "type": "build", "section": "Build System", "hidden": true },
    { "type": "ci", "section": "Continuous Integration", "hidden": true },
    { "type": "chore", "section": "Chores", "hidden": true }
  ]
}
```

`release-type: node` makes release-please read/write `version` in `package.json` at repo root — matching the existing single-source-of-truth. `Cargo.toml` and `tauri.conf.json` are deliberately not listed as `extra-files` here; Task 3 propagates them via the existing `scripts/sync-version.mjs` instead, so that logic lives in one place.

- [ ] **Step 3: Validate JSON syntax locally**

```bash
node -e "JSON.parse(require('fs').readFileSync('release-please-config.json', 'utf8')); JSON.parse(require('fs').readFileSync('.release-please-manifest.json', 'utf8')); console.log('valid json')"
```
Expected: `valid json`

- [ ] **Step 4: Commit**

```bash
git add release-please-config.json .release-please-manifest.json
git commit -m "chore: add release-please config bootstrapped at v0.1.0"
```

---

### Task 3: Release workflow — version bump PR + tag/release + installer build

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `release-please-config.json` / `.release-please-manifest.json` (Task 2); `scripts/sync-version.mjs` (existing, run via `npm run sync-version`); `src-tauri` Tauri project (existing, built via `tauri-apps/tauri-action`).
- Produces: on push to `main` — either an open/updated release PR, or (when that PR was just merged) a git tag, a GitHub Release, and attached macOS + Windows installer artifacts.

- [ ] **Step 1: Write the workflow file**

```yaml
name: Release

on:
  push:
    branches: [main]

permissions:
  contents: write
  pull-requests: write

jobs:
  release-please:
    name: release-please
    runs-on: ubuntu-latest
    outputs:
      release_created: ${{ steps.release.outputs.release_created }}
      tag_name: ${{ steps.release.outputs.tag_name }}
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          config-file: release-please-config.json
          manifest-file: .release-please-manifest.json

  sync-version-on-release-pr:
    name: sync-version-on-release-pr
    needs: release-please
    if: needs.release-please.outputs.release_created != 'true'
    runs-on: ubuntu-latest
    steps:
      - name: Find open release-please PR
        id: find-pr
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          branch=$(gh pr list --base main --state open --json headRefName \
            --jq '[.[] | select(.headRefName | startswith("release-please--branches--main"))][0].headRefName')
          echo "branch=$branch" >> "$GITHUB_OUTPUT"

      - uses: actions/checkout@v5
        if: steps.find-pr.outputs.branch != ''
        with:
          ref: ${{ steps.find-pr.outputs.branch }}

      - uses: actions/setup-node@v5
        if: steps.find-pr.outputs.branch != ''
        with:
          node-version: 22

      - name: Sync version into Cargo.toml and tauri.conf.json
        if: steps.find-pr.outputs.branch != ''
        run: npm run sync-version

      - name: Commit if changed
        if: steps.find-pr.outputs.branch != ''
        run: |
          if ! git diff --quiet; then
            git config user.name "github-actions[bot]"
            git config user.email "github-actions[bot]@users.noreply.github.com"
            git add src-tauri/Cargo.toml src-tauri/tauri.conf.json
            git commit -m "chore: sync Cargo.toml and tauri.conf.json version"
            git push origin HEAD:${{ steps.find-pr.outputs.branch }}
          fi

  build-installers:
    name: build-installers
    needs: release-please
    if: needs.release-please.outputs.release_created == 'true'
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
          - platform: windows-latest
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v5
        with:
          ref: ${{ needs.release-please.outputs.tag_name }}

      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: npm

      - name: Install dependencies
        run: npm ci

      - uses: dtolnay/rust-toolchain@1.97.1

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ needs.release-please.outputs.tag_name }}
          releaseName: ${{ needs.release-please.outputs.tag_name }}
          releaseDraft: false
          prerelease: false
```

`tauri-action` finds-or-creates a release by `tagName` — since `release-please` already created the release for that tag, `tauri-action` attaches installer assets to that existing release rather than making a duplicate. The `sync-version-on-release-pr` job locates the open release PR by its well-known branch-naming convention (`release-please--branches--main...`) via `gh pr list`, rather than depending on an unverified action output field — more robust than guessing at `release-please-action`'s exact output shape.

- [ ] **Step 2: Validate YAML syntax locally**

```bash
node -e "require('js-yaml').load(require('fs').readFileSync('.github/workflows/release.yml', 'utf8')); console.log('valid yaml')"
```
Expected: `valid yaml`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add release workflow (release-please + tauri installer build)"
```

- [ ] **Step 4: Push**

```bash
git push
```

Expected: this task's commits land on the already-open PR from Task 1 (`feat/semver-release-workflow` → `develop`); `validate-pr-title` re-runs and stays green since the PR title is unchanged.

---

### Task 4: Mark PR ready and document the two manual follow-ups

**Files:**
- Modify: none (GitHub PR metadata + user actions outside this repo)

**Interfaces:**
- Consumes: the open PR from Task 1.
- Produces: nothing new in-repo — this task is where the two manual, outside-of-code steps identified in the spec actually get done, since neither is possible from within this session.

- [ ] **Step 1: Mark the PR ready for review**

```bash
gh pr ready
```

- [ ] **Step 2: Tell the user to add the required status check**

Confirm with the user that after this PR merges to `develop`, they add `validate-pr-title` as a required status check on `develop`'s branch protection rule (GitHub Settings → Branches → `develop` rule → Require status checks to pass → select `validate-pr-title`). This cannot be done from this session without `gh auth login`.

- [ ] **Step 3: Confirm `main`'s required-status-checks list won't block the release-please PR**

Ask the user whether `main`'s branch protection required-status-checks list names specific check names. If it does, `release.yml`'s `release-please` job (job id `release-please`) must be added to that list, or the release PR will be unmergeable. If `main`'s protection only requires "some PR + some checks" without naming specific ones, no action needed.

- [ ] **Step 4: Wait for user review, then merge per normal workflow**

Do not merge this PR without the user's explicit go-ahead (per this repo's standing branch workflow: feature branch → PR → user reviews/merges to `develop`).

---

## Self-Review Notes

- **Spec coverage:** Workflow 1 → Task 1. Workflow 2 (release-please + sync-version + tag/release + installer build) → Tasks 2–3. Manual follow-ups from the spec's "Manual follow-up required from user" section → Task 4. All spec sections covered.
- **Placeholder scan:** no TBD/TODO; every step has runnable commands or complete file content.
- **Type/name consistency:** `release-please` job id and its outputs (`release_created`, `tag_name`) are defined once in Task 3 Step 1 and referenced identically (`needs.release-please.outputs.*`) by the two downstream jobs in the same file — no drift.
