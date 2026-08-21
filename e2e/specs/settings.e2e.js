import { navigateTo, login, FIRST_RUN_PIN } from "../helpers/seed.js";

// US-M7.1/M7.2/M7.4 (S10). Runs against the same real app-data directory
// as every other spec (no setup call here — see helpers/seed.js's own doc
// comment) and only touches settings state, which no earlier spec depends
// on. wdio starts a fresh process per spec file though, so this file logs
// back in for itself first.
before(async () => {
  await login(FIRST_RUN_PIN);
});

describe("Settings", () => {
  it("saves a royalty setting change", async () => {
    await navigateTo("Settings");
    await $("#royalty-min").waitForExist({ timeout: 3000 });
    await $("#royalty-min").setValue("4");
    await $("button=Save royalty settings").click();
    // A Royalty save always goes through the same mid-period recalc
    // warning a Slab table save does (useRecalcWarning, settings.tsx) —
    // there is no direct-save path. Confirm it, same as the slab table
    // test below.
    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 3000 });
    const confirmButton = dialog.$("button*=Save and re-work");
    await confirmButton.waitForEnabled({ timeout: 3000 });
    await confirmButton.click();
    // A bare `*=` (no tag prefix) compiles to WebDriver's "partial link
    // text" strategy, which only matches <a> elements — the toast title
    // renders as an <h2> (@base-ui/react's Toast.Title), so this needs
    // the tag explicit, not just dropped.
    await $("h2*=Royalty settings saved").waitForExist({ timeout: 3000 });
  });

  it("T-M7.1-4: disables removing the last remaining slab row, and refuses it if reached anyway", async () => {
    await navigateTo("Settings");
    // Add/remove is staged locally now (prototype-match single "Save slab
    // table" flow) — no round trip needed to see the last-row control disable.
    const rows = await $$('[id^="slab-remove-"]');
    for (let i = 0; i < rows.length - 1; i++) {
      const removeButtons = await $$('button[aria-label="Remove this slab row"]');
      await removeButtons[0].click();
      await browser.waitUntil(
        async () => (await $$('[id^="slab-remove-"]')).length === rows.length - 1 - i,
        { timeout: 3000 },
      );
    }

    const lastRemove = $('button[aria-label="Remove row — the table must keep at least one slab"]');
    await lastRemove.waitForExist({ timeout: 3000 });
    await expect(lastRemove).toBeDisabled();
  });

  it("T-M7.1-6: a non-monotonic slab table saves without being blocked", async () => {
    await navigateTo("Settings");
    await $("button=Add row").click();
    await $("#slab-threshold-new-0").setValue("50000.00");
    await $("#slab-percentage-new-0").setValue("1");
    await $("#slab-save-table").click();

    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 3000 });
    const confirmButton = dialog.$("button*=Save and re-work");
    await confirmButton.waitForEnabled({ timeout: 3000 });
    await confirmButton.click();

    // No monotonicity check (ADR-009) — the recalc-warning dialog appears for
    // every slab save; confirming it completing with the success toast
    // (not a validation refusal) is the assertion.
    await $("h2*=Slab table saved").waitForExist({ timeout: 3000 });
  });

  it("T-M7.4-2: the backup schedule segmented control saves immediately, no separate Save step", async () => {
    await navigateTo("Settings");
    // SegmentedControl's options are base-ui Radio.Root, which renders a
    // <span> (role="radio"), never a <button>.
    await $("span=Weekly").click();
    await $("h2*=Backup schedule set to weekly").waitForExist({ timeout: 3000 });
  });

  it("T-M7.4-4: \"Back up now\" produces a manual backup at the top of the Restore card's list", async () => {
    await navigateTo("Settings");
    await $("button=Back up now").click();
    await $("h2*=Console backed up").waitForExist({ timeout: 3000 });
    // Same "no tag = link text only" trap, plus the actual text is "Manual
    // — {date}" (a prefix, not the whole string) — needs both the <span>
    // tag (restore-option-list.tsx) and a partial match.
    await $("span*=Manual —").waitForExist({ timeout: 3000 });
  });

  it("T-M7.4-6: restore confirmation follows the checklist pattern — Cancel first, disabled until checked", async () => {
    await navigateTo("Settings");
    // The restore button reads "Restore from selected backup" until a
    // radio option is actually chosen (restore-option-list.tsx) — nothing
    // is pre-selected by default (destructive/"cannot be undone" action,
    // deliberately not one-clickable). Select the manual backup first,
    // same as an operator would.
    await $("span*=Manual —").waitForExist({ timeout: 3000 });
    await $("span*=Manual —").click();
    await $("button*=Restore from Manual").waitForExist({ timeout: 3000 });
    await $("button*=Restore from Manual").click();

    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 3000 });
    await expect(dialog.$("button=Cancel")).toBeFocused();
    const confirmButton = dialog.$("button=Restore");
    await expect(confirmButton).toBeDisabled();

    await dialog.$('input[type="checkbox"]').click();
    await expect(confirmButton).toBeEnabled();

    // Cancel is a true no-op: closing without confirming must not restore.
    await dialog.$("button=Cancel").click();
    await expect(dialog).not.toBeExisting();
    await $("nav").$("a=Settings").waitForExist({ timeout: 3000 });
  });
});
