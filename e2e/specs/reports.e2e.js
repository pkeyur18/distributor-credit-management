import { navigateTo } from "../helpers/seed.js";

// US-M6.1/M6.2/M6.3/M6.4 (S13). Runs after business-volume-entry.e2e.js in
// the same shared session, reusing its root member/entry rather than
// re-onboarding. Every export button opens a native OS save dialog on
// click, which WebdriverIO/tauri-driver cannot drive (the same constraint
// settings.e2e.js's restore-from-file flow already works around by never
// clicking the button that opens one) — so this spec covers everything
// reachable up to that point: the screen renders, the column picker and
// threshold field behave, and the closed-month card's conditional
// rendering is correct, without ever triggering a dialog.
describe("Reports", () => {
  it("renders the three always-available cards with an editable column picker", async () => {
    await navigateTo("Reports");

    await $("=Monthly data").waitForExist({ timeout: 3000 });
    await $("=Yearly average").waitForExist({ timeout: 3000 });
    await $("=Low-contribution report").waitForExist({ timeout: 3000 });

    // T-M6.1-3: the optional column picker (Rule-33), not including
    // Active/inactive status — that one is force-included server-side
    // regardless of the picker (NFR-8), so it isn't offered as a toggle.
    const emailCheckbox = $("label*=Email").$('input[type="checkbox"]');
    await emailCheckbox.waitForExist({ timeout: 3000 });
    await expect(emailCheckbox).not.toBeChecked();
    await emailCheckbox.click();
    await expect(emailCheckbox).toBeChecked();

    // NFR-8: Active/inactive status is force-included server-side
    // regardless of selection, so it's dropped from the picker entirely
    // rather than offered as a checkbox nobody's answer can change.
    await expect($("label*=Active/inactive status")).not.toExist();
  });

  it("T-M6.3-2: the low-contribution threshold field is pre-filled from settings and editable", async () => {
    await navigateTo("Reports");
    const threshold = $("#low-threshold");
    await threshold.waitForExist({ timeout: 3000 });
    await expect(threshold).toHaveValue("100.00");
    await threshold.setValue("50.00");
    await expect(threshold).toHaveValue("50.00");
  });

  it("T-M6.4: the closed-month snapshot card stays hidden with no closed month yet", async () => {
    // Nothing in this shared session has ever closed a period, so
    // list_backups returns empty and the whole card must not render.
    await navigateTo("Reports");
    await $("=Monthly data").waitForExist({ timeout: 3000 });
    await expect($("=Closed month snapshot")).not.toExist();
  });
});
