import { login, navigateTo, FIRST_RUN_PIN } from "../helpers/seed.js";

// US-M8.2/M8.3 (S5/S7). Runs against the same real app-data directory and
// credential as every other spec (no setup call here — see
// helpers/seed.js's own doc comment), so this file logs back in for
// itself first, same as settings.e2e.js/structure.e2e.js.
//
// Ordering: this file is deliberately run LAST (see wdio.conf.js's
// explicit `specs` list) for two reasons. First, the failed-login-attempt
// counter (D-2's lockout ladder) is shared with every other spec's use of
// `login()` — this file's own final test drives it to a real lockout, and
// nothing may run after that expects a working login. Second, it is the
// one spec that exercises the lock/sign-out session boundary at all;
// nothing else in the suite depends on the authenticated session it
// leaves behind.
before(async () => {
  await login(FIRST_RUN_PIN);
});

describe("Session lock and resume", () => {
  it("Lock session routes to the Locked screen, and the correct PIN resumes into the console", async () => {
    await $("button=Lock session").click();
    await $("h1*=Session locked").waitForExist({ timeout: 5000 });

    await browser.keys(FIRST_RUN_PIN.split(""));

    await $("nav").waitForExist({ timeout: 5000 });
    await expect($("h1*=Session locked")).not.toExist();
  });

  it("a wrong PIN on the Locked screen shows the generic invalid-credential message", async () => {
    await $("button=Lock session").click();
    await $("h1*=Session locked").waitForExist({ timeout: 5000 });

    await browser.keys("000000".split(""));

    await $("p*=Incorrect PIN or password").waitForExist({ timeout: 5000 });

    // Recover the shared attempt counter before the next test — a
    // successful credential check resets it (Rule D-2), so this one
    // failure does not carry forward toward the ladder's threshold.
    await browser.keys(FIRST_RUN_PIN.split(""));
    await $("nav").waitForExist({ timeout: 5000 });
  });

  it("'Sign out instead' from the Locked screen returns to Login without unlocking", async () => {
    await $("button=Lock session").click();
    await $("h1*=Session locked").waitForExist({ timeout: 5000 });

    await $("button=Sign out instead").click();

    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });
    await browser.keys(FIRST_RUN_PIN.split(""));
    await $("nav").waitForExist({ timeout: 5000 });
  });

  it("Sign out from the sidebar goes straight back to Login", async () => {
    await $("button=Sign out").click();

    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });
    await expect($("nav")).not.toExist();

    await browser.keys(FIRST_RUN_PIN.split(""));
    await $("nav").waitForExist({ timeout: 5000 });
  });
});

describe("Recovery screen — format validation", () => {
  it("refuses an empty code before ever calling the server", async () => {
    await $("button=Sign out").click();
    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });

    await $("a=Forgot your PIN or password?").click();
    await $("h1*=Recover access").waitForExist({ timeout: 5000 });

    await $("button=Verify code").click();
    await $("p*=Enter the recovery code you saved").waitForExist({ timeout: 5000 });
    await expect($("h1*=Set a new credential")).not.toExist();

    await $("a=Back to sign in").click();
    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });
    await browser.keys(FIRST_RUN_PIN.split(""));
    await $("nav").waitForExist({ timeout: 5000 });
  });
});

// D-2: five consecutive failures locks the account. Deliberately the last
// test in the last spec file of the whole suite run (see the ordering
// note above) — nothing after this depends on being able to log in again
// inside the lockout window.
describe("Login lockout ladder", () => {
  it("exactly 5 consecutive wrong PINs locks the account with a live countdown, hiding the keypad", async () => {
    await navigateTo("Home");
    await $("button=Sign out").click();
    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });

    for (let attempt = 1; attempt <= 4; attempt++) {
      await browser.keys("000000".split(""));
      await $("p*=Incorrect PIN or password").waitForExist({ timeout: 5000 });
    }
    await browser.keys("000000".split(""));

    await $("p=Too many attempts").waitForExist({ timeout: 5000 });
    const countdown = await $(".text-numeric-lg").getText();
    expect(countdown).toMatch(/^\d+s$/);
    await expect($('button[aria-label="Backspace"]')).not.toExist();
  });
});
