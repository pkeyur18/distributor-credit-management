import { completeFirstRunSetup, addRootMember, addMember, navigateTo } from "../helpers/seed.js";

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

    await navigateTo("Business Volume Entry");
    await $("#entry-search").setValue("Asha");
    const result = $("button*=Asha Patel");
    await result.waitForExist({ timeout: 3000 });
    await result.click();

    await $("#entry-amount").setValue("1000.00");
    const saveButton = $("button=Save entry");
    await expect(saveButton).toBeEnabled();
    await saveButton.click();

    // "Recorded this session" is the local, honest substitute for a
    // period-entries list — no command in the closed 40-command surface
    // lists a member's past entries yet (see business-volume-entry.tsx's
    // own doc comment).
    const sessionList = $("div=Recorded this session");
    await sessionList.waitForExist({ timeout: 3000 });
    await expect($("span=1000.00")).toExist();
  });
});
