import { addMember, navigateTo, recordEntry } from "../helpers/seed.js";

// US-M4.1/M4.2/M4.4 (S8) — the project's go/no-go exit gate (M-5,
// 02-roadmap.md): "all six golden totals reproduce through the real UI,"
// not just the engine's own unit tests (T-M3.1-8, S6). The same six
// scenarios `src-tauri/tests/fixtures/mod.rs` encodes as Rust data are
// transcribed here — this harness has no path to that fixture file, so a
// seventh scenario is a new array entry in both places, never new test
// code in either. Figures are real units (not ×100) exactly as the source
// document and the Rust fixture state them; the UI's own amount field
// takes real units too (`displayToCents` does the ×100 conversion).
//
// Runs after business-volume-entry.e2e.js in the same unseeded app-data
// directory (wdio.conf.js's own onPrepare comment) and hangs every
// scenario subtree off that spec's root member as a direct child — enough,
// since Rule-6/8/10/46 only ever look at a node's own descendants, never
// its siblings or ancestors. Nesting six independent scenario trees under
// a shared root doesn't perturb any of their own figures.
const SCENARIOS = [
  { id: "S1", root: ["D", 500], children: [["A", 300], ["B", 50], ["C", 1_000]], total: "65.00" },
  { id: "S2", root: ["D", 500], children: [["A", 300], ["B", 50], ["C", 3_000]], total: "62.00" },
  {
    id: "S3",
    root: ["A", 500],
    children: [
      ["B", 1_250],
      ["C", 1_250],
      ["D", 1_250],
      ["E", 1_250],
      ["F", 1_250],
      ["G", 1_250],
    ],
    total: "510.00",
  },
  {
    id: "S4",
    root: ["P", 0],
    children: [["A", 10_000], ["B", 20_000], ["C", 30_000], ["D", 40_000]],
    total: "1000.00",
  },
  {
    id: "S5",
    root: ["P", 0],
    children: [
      ["A", 10_000],
      ["B", 10_000],
      ["C", 10_000],
      ["D", 10_000],
      ["E", 2_000],
      ["F", 3_000],
      ["G", 4_000],
    ],
    total: "980.00",
  },
  { id: "S6", root: ["A", 100], children: [["B", 100], ["C", 100], ["D", 100]], total: "10.00" },
];

// Every scenario reuses the source document's bare letter names (A, B, C…)
// — prefixed per-scenario here so the shared search box never returns an
// ambiguous match across the six subtrees living in one session.
let phoneCounter = 9_878_000_000;
function nextPhone() {
  return String(phoneCounter++);
}

async function buildScenario(scenario, containerRootId) {
  const [rootName, rootBv] = scenario.root;
  const uniqueRootName = `${scenario.id}-${rootName}`;
  const rootId = await addMember({
    name: uniqueRootName,
    phone: nextPhone(),
    address: "Golden scenario fixture",
    referenceId: containerRootId,
  });
  // Rule-16a: a member with no activity has no entry, not a zero entry —
  // scenarios 4 and 5's own root (P) has own BV 0 by the source document's
  // own confirmed simplification.
  if (rootBv > 0) {
    await recordEntry({ memberName: uniqueRootName, amount: rootBv.toFixed(2) });
  }

  for (const [childName, childBv] of scenario.children) {
    const uniqueChildName = `${scenario.id}-${childName}`;
    await addMember({
      name: uniqueChildName,
      phone: nextPhone(),
      address: "Golden scenario fixture",
      referenceId: rootId,
    });
    await recordEntry({ memberName: uniqueChildName, amount: childBv.toFixed(2) });
  }
}

async function rewardsThisPeriod(memberName) {
  await navigateTo("Home");
  await $("#home-search").setValue(memberName);
  const row = $(`button*=${memberName}`);
  await row.waitForExist({ timeout: 3000 });
  await row.click();
  return $('//div[text()="Rewards this period"]/following-sibling::div[1]').getText();
}

describe("Golden scenarios reproduce through the real UI", () => {
  it("all six scenario totals match Rewards this period on Member Detail", async () => {
    await navigateTo("Home");
    await $("#home-search").setValue("Root Member");
    const rootRow = $("button*=Root Member");
    await rootRow.waitForExist({ timeout: 3000 });
    await rootRow.click();
    const url = await browser.getUrl();
    const containerRootId = url.match(/\/member\/(\d+)/)[1];

    for (const scenario of SCENARIOS) {
      await buildScenario(scenario, containerRootId);
      const uniqueRootName = `${scenario.id}-${scenario.root[0]}`;
      const value = await rewardsThisPeriod(uniqueRootName);
      expect(value).toBe(scenario.total);
    }
  });
});
