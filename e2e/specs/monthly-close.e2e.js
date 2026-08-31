import { navigateTo, login, FIRST_RUN_PIN } from "../helpers/seed.js";

// US-M5.1 (S11). Runs after reports.e2e.js — its own T-M6.4 test asserts
// "no closed month yet" against this same shared app-data directory, so
// nothing before it may close one. wdio starts a fresh process per spec
// file, so this file logs back in for itself first.
//
// The wizard's "Generate backup" step (`chooseBackupLocation`,
// monthly-close.tsx) calls the Tauri dialog plugin's native save picker
// unconditionally — there is no alternate path around it, unlike
// settings.e2e.js's restore-from-file flow or reports.e2e.js's exports,
// which simply never click the button that opens one. WebDriver/
// tauri-driver cannot drive a native OS dialog at all, so actually
// completing a close (and therefore ever producing a real closed month
// for CorrectionPanel/Audit to exercise) is not reachable through this
// harness — see reports.e2e.js's own header comment for the same
// constraint on its export buttons. This spec covers everything up to
// that point: the outstanding-month list, opening the wizard, its
// confirm-step summary counts, Cancel being a true no-op, and advancing
// into the backup step's initial (unconfirmed) render.
before(async () => {
  await login(FIRST_RUN_PIN);
});

describe("Monthly close — wizard, up to the native backup-file dialog", () => {
  it("lists the single outstanding month as closable now", async () => {
    await navigateTo("Monthly Close");
    await $("h1=Monthly close").waitForExist({ timeout: 5000 });

    // business-volume-entry.e2e.js's own second test already established
    // that exactly one month (the current one) is outstanding in this
    // shared session.
    await $("span=Oldest — closable now").waitForExist({ timeout: 3000 });
    await $("button=Close").waitForExist({ timeout: 3000 });
  });

  it("Cancel from the confirm step is a true no-op, returning to the plain list", async () => {
    await navigateTo("Monthly Close");
    await $("button=Close").click();

    await $('h1*=Close ').waitForExist({ timeout: 5000 });
    await $(".text-numeric").waitForExist({ timeout: 3000 });
    // The confirm step's three summary counts (Members / With an entry /
    // On top slab) are all present, whatever their values.
    await expect($$(".text-numeric")).toBeElementsArrayOfSize(3);

    await $("button=Cancel").click();

    await $("h1=Monthly close").waitForExist({ timeout: 5000 });
    // Read-only stat query (begin_close) — nothing was mutated, so the
    // month is still exactly where it was.
    await $("span=Oldest — closable now").waitForExist({ timeout: 3000 });
  });

  it("Continue advances to the backup step, unconfirmed until a location is chosen", async () => {
    await navigateTo("Monthly Close");
    await $("button=Close").click();
    await $('h1*=Close ').waitForExist({ timeout: 5000 });

    await $("button=Continue").click();

    await $("h1=Confirm a backup").waitForExist({ timeout: 5000 });
    await $("button=Generate backup").waitForExist({ timeout: 3000 });
    // Not yet confirmed — the commit button only appears once
    // `chooseBackupLocation` (native dialog) has run.
    await expect($("button*=cannot be undone")).not.toExist();

    // Back returns to the confirm step without having touched anything
    // that needs a native dialog.
    await $("button=Back").click();
    await $('h1*=Close ').waitForExist({ timeout: 5000 });
    await $("button=Cancel").click();
    await $("h1=Monthly close").waitForExist({ timeout: 5000 });
  });
});
