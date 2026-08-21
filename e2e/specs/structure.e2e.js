import { navigateTo, idOfPhone, login, FIRST_RUN_PIN } from "../helpers/seed.js";

// T-M4.2-6 — the inactive-node treatment on the Structure chart (Rule-28:
// deactivation has zero calculation effect, but the node must still show
// the distinct colour plus a labelled pill, per T-M1.3-4). Runs after
// business-volume-entry.e2e.js against the same real app-data directory
// and deactivates the member that spec already onboarded ("Asha Patel"),
// then confirms the pill survives the trip from Member Detail into the
// Structure chart. wdio starts a fresh process per spec file, so this
// file logs back in for itself first — covers both describe blocks below,
// since they share this one file/session.
before(async () => {
  await login(FIRST_RUN_PIN);
});

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
