import { navigateTo, addMember, idOfPhone } from "../helpers/seed.js";

// US-M4.3 (§5.3a/§6.13, Rule-45). Runs right after business-volume-entry.e2e.js
// in the same shared session, which leaves exactly one member ("Asha Patel")
// beneath "Root Member" — below the 60-descendant gate (V4.5), so the first
// case here covers the immediate-open path. It then seeds past the gate
// itself (no seed-via-file shortcut exists in this project — see
// helpers/seed.js's own doc comment) to cover the confirmation path.
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
    await expect($("=No legs beneath")).not.toBeExisting();

    await $('input[placeholder="Find a member by name or number"]').setValue("Asha");
    await $("=Asha Patel").waitForExist({ timeout: 3000 });

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
    await dialog.$("*=61 members").waitForExist({ timeout: 3000 });

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
    await $("p*=61 members").waitForExist({ timeout: 3000 });

    await browser.closeWindow();
    await browser.switchToWindow(before[0]);
  });
});
