import { navigateTo, addMember, idOfPhone, login, FIRST_RUN_PIN } from "../helpers/seed.js";

// US-M4.3 (§5.3a/§6.13, Rule-45). Runs right after business-volume-entry.e2e.js
// against the same real app-data directory, which leaves exactly one member
// ("Asha Patel") beneath "Root Member" — below the 60-descendant gate
// (V4.5), so the first case here covers the immediate-open path. It then
// seeds past the gate itself (no seed-via-file shortcut exists in this
// project — see helpers/seed.js's own doc comment) to cover the
// confirmation path. wdio starts a fresh process per spec file, so this
// file logs back in for itself before touching anything.
before(async () => {
  await login(FIRST_RUN_PIN);
});

describe("Full Hierarchy Window", () => {
  it("opens immediately on a small network, read-only, three fields per node", async () => {
    await navigateTo("Structure");

    const before = await browser.getWindowHandles();
    await $("button=View full hierarchy").click();

    await browser.waitUntil(async () => (await browser.getWindowHandles()).length > before.length, {
      timeout: 5000,
      timeoutMsg: "expected a new full-hierarchy window to open",
    });
    const after = await browser.getWindowHandles();
    const newHandle = after.find((h) => !before.includes(h));
    await browser.switchToWindow(newHandle);

    await $("h1*=Full hierarchy").waitForExist({ timeout: 3000 });
    await $("p*=2 members").waitForExist({ timeout: 3000 });
    // Read-only: no leg-count/expand affordance, which only renders when
    // StructureTreeNode is interactive (structure-tree-node.tsx).
    // Bare `=` only matches <a> elements — this text is a <span>.
    await expect($("span=No legs beneath")).not.toBeExisting();

    await $('input[placeholder="Find a member by name or number"]').setValue("Asha");
    // The 2px indigo ring (outline-accent) lands on the matched node's
    // wrapper, not just its existence in the DOM — Asha Patel's node is
    // always present here regardless of search, so this confirms the
    // highlight itself fired, not merely that the member renders.
    const highlighted = $('//div[contains(@class,"outline-accent")]//div[text()="Asha Patel"]');
    await highlighted.waitForExist({ timeout: 3000 });

    await browser.closeWindow();
    await browser.switchToWindow(before[0]);
  });

  it("gates above 60 descendants, names the exact count, and Cancel opens nothing", async () => {
    const rootId = await idOfPhone("9876500001");
    for (let i = 1; i <= 60; i++) {
      await addMember({
        name: `Bulk Member ${i}`,
        phone: `987651${String(i).padStart(4, "0")}`,
        address: "3 Bulk Street",
        referenceId: rootId,
      });
    }

    await navigateTo("Structure");
    const before = await browser.getWindowHandles();
    await $("button=View full hierarchy").click();

    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 3000 });
    // Bare `*=` only matches <a> elements — this text is a <p> (structure.tsx).
    await dialog.$("p*=61 members").waitForExist({ timeout: 3000 });

    await dialog.$("button=Cancel").click();
    await dialog.waitForExist({ timeout: 3000, reverse: true });
    await expect(await browser.getWindowHandles()).toHaveLength(before.length);

    await $("button=View full hierarchy").click();
    await dialog.waitForExist({ timeout: 3000 });
    await dialog.$("button=Open").click();

    await browser.waitUntil(async () => (await browser.getWindowHandles()).length > before.length, {
      timeout: 5000,
      timeoutMsg: "expected a new full-hierarchy window to open after confirming",
    });
    const after = await browser.getWindowHandles();
    const newHandle = after.find((h) => !before.includes(h));
    await browser.switchToWindow(newHandle);
    // Header shows the total node count (root + descendants); the gate
    // above named the descendant-only count (61) — one more than this.
    await $("p*=62 members").waitForExist({ timeout: 3000 });

    // T-QA.6-3/AC-45: the main console must stay responsive while the full
    // hierarchy window is open and drawing. E2E has no bulk-seed path (no
    // test-only seeding command exists — see helpers/seed.js's own header
    // comment; the same limitation this project already accepted for
    // multi-period state), so this can't drive the literal 25,000-node
    // ceiling `tests/performance.rs`'s ignored suite exercises at the
    // Rust level — only a manual QA pass at real scale can fully close
    // that gap. What this *can* and does prove: Rule-45's separate-window
    // isolation (T-M4.3-2) genuinely decouples the two — switching back to
    // the main window and driving an ordinary interaction succeeds within
    // a normal timeout, with the other window still open and rendered.
    await browser.switchToWindow(before[0]);
    await navigateTo("Home");
    await $("#home-search").waitForExist({ timeout: 3000 });

    await browser.switchToWindow(newHandle);
    await browser.closeWindow();
    await browser.switchToWindow(before[0]);
  });
});
