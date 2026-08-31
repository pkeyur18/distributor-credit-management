import { login, navigateTo, jsClick, FIRST_RUN_PIN } from "../helpers/seed.js";

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
//
// PIN entry here clicks the onscreen keypad rather than reusing
// helpers/seed.js's `browser.keys(pin.split(""))` pattern. That pattern is
// safe for FIRST_RUN_PIN (six distinct digits) and is proven reliable by
// every other spec's `login()` call, but Login/Locked's own keydown
// listener re-subscribes on every `pinBuffer` change
// (login.tsx/locked.tsx's own `useEffect` dependency array) — a real
// physical keyboard could never out-race that resubscription, but
// WebDriver's synthetic keydown dispatch can fire faster than React can
// repaint between them, and a *repeated* digit (this file's deliberately
// wrong "000000" attempts) is exactly the case a stale closure can't tell
// apart from a single keypress. Clicking six separate on-screen buttons
// goes through ordinary React onClick handlers instead, one real DOM
// event at a time, sidestepping the race entirely.
async function pressDigits(pin) {
  for (const digit of pin) {
    await jsClick($(`button=${digit}`));
  }
}

before(async () => {
  await login(FIRST_RUN_PIN);
});

describe("Session lock and resume", () => {
  it("Lock session routes to the Locked screen, and the correct PIN resumes into the console", async () => {
    await jsClick($("button=Lock session"));
    await $("h1*=Session locked").waitForExist({ timeout: 5000 });

    await pressDigits(FIRST_RUN_PIN);

    await $("nav").waitForExist({ timeout: 5000 });
    await expect($("h1*=Session locked")).not.toExist();
  });

  it("a wrong PIN on the Locked screen shows the generic invalid-credential message", async () => {
    await jsClick($("button=Lock session"));
    await $("h1*=Session locked").waitForExist({ timeout: 5000 });

    await pressDigits("000000");

    await $("p*=Incorrect PIN or password").waitForExist({ timeout: 5000 });

    // Recover the shared attempt counter before the next test — a
    // successful credential check resets it (Rule D-2), so this one
    // failure does not carry forward toward the ladder's threshold.
    await pressDigits(FIRST_RUN_PIN);
    await $("nav").waitForExist({ timeout: 5000 });
  });

  it("'Sign out instead' from the Locked screen returns to Login without unlocking", async () => {
    await jsClick($("button=Lock session"));
    await $("h1*=Session locked").waitForExist({ timeout: 5000 });

    await jsClick($("button=Sign out instead"));

    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });
    await pressDigits(FIRST_RUN_PIN);
    await $("nav").waitForExist({ timeout: 5000 });
  });

  it("Sign out from the sidebar goes straight back to Login", async () => {
    await jsClick($("button=Sign out"));

    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });
    await expect($("nav")).not.toExist();

    await pressDigits(FIRST_RUN_PIN);
    await $("nav").waitForExist({ timeout: 5000 });
  });
});

describe("Recovery screen — format validation", () => {
  it("refuses an empty code before ever calling the server", async () => {
    await jsClick($("button=Sign out"));
    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });

    await jsClick($("a=Forgot your PIN or password?"));
    await $("h1*=Recover access").waitForExist({ timeout: 5000 });

    await jsClick($("button=Verify code"));
    await $("p*=Enter the recovery code you saved").waitForExist({ timeout: 5000 });
    await expect($("h1*=Set a new credential")).not.toExist();

    await jsClick($("a=Back to sign in"));
    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });
    await pressDigits(FIRST_RUN_PIN);
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
    await jsClick($("button=Sign out"));
    await $("h1*=Member Rewards Console").waitForExist({ timeout: 5000 });

    for (let attempt = 1; attempt <= 4; attempt++) {
      await pressDigits("000000");
      await $("p*=Incorrect PIN or password").waitForExist({ timeout: 5000 });
    }
    await pressDigits("000000");

    await $("p=Too many attempts").waitForExist({ timeout: 5000 });
    const countdown = await $(".text-numeric-lg").getText();
    expect(countdown).toMatch(/^\d+s$/);
    await expect($('button[aria-label="Backspace"]')).not.toExist();
  });
});
