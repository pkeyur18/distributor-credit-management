import { navigateTo } from "../helpers/seed.js";

// T-M4.2-6 — the inactive-node treatment on the Structure chart (Rule-28:
// deactivation has zero calculation effect, but the node must still show
// the distinct colour plus a labelled pill, per T-M1.3-4). Runs after
// business-volume-entry.e2e.js in the same shared session and deactivates
// the member that spec already onboarded ("Asha Patel"), then confirms the
// pill survives the trip from Member Detail into the Structure chart.
describe("Structure — inactive-node treatment", () => {
  it("shows the Inactive pill on a deactivated member's node", async () => {
    await navigateTo("Home");
    await $("#home-search").setValue("Asha Patel");
    const result = $("button*=Asha Patel");
    await result.waitForExist({ timeout: 3000 });
    await result.click();

    await $("button=Deactivate").click();
    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 3000 });
    await dialog.$("button=Deactivate").click();

    const reactivateButton = $("button=Reactivate");
    await reactivateButton.waitForExist({ timeout: 3000 });

    await $("button=View in structure").click();
    await $("=Inactive").waitForExist({ timeout: 3000 });
  });
});
