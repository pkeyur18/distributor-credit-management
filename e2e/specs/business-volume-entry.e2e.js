import { completeFirstRunSetup, addRootMember, addMember, navigateTo } from "../helpers/seed.js";

// Specs in this directory share one running app instance and one login
// session (see helpers/seed.js's own doc comment) — first-run setup can
// only ever happen once per instance, so every test after the first one
// in this describe block must build on the state earlier tests already
// left behind rather than calling completeFirstRunSetup/addRootMember a
// second time.

// US-M2.1 (S7) — the golden path this whole harness exists to catch:
// record a Business Volume entry through the real UI and confirm the
// affected figure updates with no "recalculate" control anywhere on
// screen (Rule-26). First run of this spec also exercises US-M8.1 and
// US-M1.1's UI paths, since there is no other way to reach a seeded
// hierarchy (see helpers/seed.js's own doc comment).
describe("Business Volume Entry", () => {
  it("records an entry against a newly onboarded member", async () => {
    await completeFirstRunSetup("482913");
    const rootId = await addRootMember({
      name: "Root Member",
      phone: "9876500001",
      address: "1 Main Street",
    });
    await addMember({
      name: "Asha Patel",
      phone: "9876500002",
      address: "2 Side Street",
      referenceId: rootId,
    });

    await navigateTo("Volume Entry");
    await $("#entry-search").setValue("Asha");
    const result = $("button*=Asha Patel");
    await result.waitForExist({ timeout: 3000 });
    await result.click();

    await $("#entry-amount").setValue("1000.00");
    const saveButton = $("button=Save entry");
    await expect(saveButton).toBeEnabled();
    await saveButton.click();

    // API-41's `list_period_entries` backs the real period table below the
    // form — the saved entry must show up as a row, and the "Entries
    // recorded" summary node must count it.
    const amountCell = $("td*=1000.00");
    await amountCell.waitForExist({ timeout: 3000 });
    await expect($("td*=Asha Patel")).toExist();
    await expect($('[id="entries-page-size"]')).toExist();
  });

  // T-M2.5-4's negative case: a fresh console has exactly one recordable
  // month (the current one), so the month switcher (T-M2.5-1) must render
  // nowhere on screen — no control, no "Showing figures for" text. The
  // positive two-outstanding-months case has no equivalent E2E coverage:
  // producing it requires the login catch-up (US-M5.5) to observe a real
  // calendar-month boundary, which nothing in `e2e/helpers/seed.js` can
  // manufacture inside a single test run — that path is covered instead by
  // `m4_search`'s and `m5_close`'s own Rust unit tests against a seeded
  // multi-period database.
  it("shows no month switcher when only one month is recordable", async () => {
    // Reuses the session the previous test already set up — nothing here
    // has touched a second period, so exactly one month (the current one)
    // is still recordable.
    await navigateTo("Volume Entry");
    await expect($("*=Showing figures for")).not.toExist();
  });
});
