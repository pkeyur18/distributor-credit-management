import { navigateTo, login, FIRST_RUN_PIN } from "../helpers/seed.js";

// US-M5.1 (S11). Runs after reports.e2e.js — its own T-M6.4 test asserts
// "no closed month yet" against this same shared app-data directory, so
// nothing before it may close one. wdio starts a fresh process per spec
// file, so this file logs back in for itself first.
//
// The close wizard itself (confirm → backup → commit) is not reachable
// through this harness at all: `get_outstanding_periods` only ever
// returns a period whose status is `awaiting_close` (m5_close/mod.rs), and
// the period this shared session has had since business-volume-entry.e2e.js's
// setup is `open` — the *current* month, never yet due for closing. A
// period only becomes `awaiting_close` once a later login observes that a
// new real calendar month has started (US-M5.5's catch-up), which nothing
// in this suite can manufacture inside one run — the same limitation
// business-volume-entry.e2e.js's own second test already documents for the
// month-switcher case. That state is covered instead by m5_close's own
// Rust unit tests against a seeded multi-period database
// (`get_outstanding_periods_lists_oldest_first` and neighbours). So this
// spec covers only what's actually reachable here: the plain status page's
// fully-caught-up rendering.
before(async () => {
  await login(FIRST_RUN_PIN);
});

describe("Monthly close — status page", () => {
  it("shows 'Fully caught up' with no Close button when nothing is awaiting close", async () => {
    await navigateTo("Monthly Close");
    await $("h1=Monthly close").waitForExist({ timeout: 5000 });

    await $("p=Fully caught up").waitForExist({ timeout: 5000 });
    await expect($("span*=closable now")).not.toExist();
    await expect($("button=Close")).not.toExist();
  });

  it("shows no closed months yet", async () => {
    await navigateTo("Monthly Close");
    await $("h1=Monthly close").waitForExist({ timeout: 5000 });

    await $("p=No months are closed yet.").waitForExist({ timeout: 5000 });
  });
});
