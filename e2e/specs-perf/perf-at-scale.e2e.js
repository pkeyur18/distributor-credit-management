import { navigateTo } from "../helpers/seed.js";

// UI perf at real ceiling scale (25,000 members) — the one NFR-1 leg
// (screen < 2s) untested past trivial member counts elsewhere in this
// suite (see full-hierarchy.e2e.js's own note on this exact gap). Runs
// against app-data pre-seeded by
// `cargo test --release --test e2e_seed seed_the_real_app_data_directory_for_e2e_perf_at_scale`
// (wdio.perf.conf.js's onPrepare) — auth.json already exists with the PIN
// below, so this logs in rather than running first-run Setup.
const KNOWN_PIN = "246810"; // must match tests/e2e_seed.rs's KNOWN_PIN
const SCREEN_BUDGET_MS = 2000; // NFR-1

async function loginWithKnownPin() {
  for (const digit of KNOWN_PIN) {
    await $(`button=${digit}`).click();
  }
  await browser.waitUntil(async () => (await browser.getUrl()).endsWith("/"), {
    timeout: 5000,
    timeoutMsg: "expected login with the known PIN to land on Home",
  });
}

describe("UI performance at ceiling scale (25,000 members)", () => {
  it("logs in and Home renders within budget", async () => {
    const start = Date.now();
    await loginWithKnownPin();
    await $("#home-search").waitForExist({ timeout: 5000 });
    const elapsed = Date.now() - start;
    console.log(`[perf] login -> Home: ${elapsed}ms`);
    expect(elapsed).toBeLessThan(SCREEN_BUDGET_MS);
  });

  it("Structure screen renders within budget", async () => {
    const start = Date.now();
    await navigateTo("Structure");
    await browser.waitUntil(async () => (await browser.getUrl()).includes("/structure"), {
      timeout: 5000,
    });
    const elapsed = Date.now() - start;
    console.log(`[perf] Structure: ${elapsed}ms`);
    expect(elapsed).toBeLessThan(SCREEN_BUDGET_MS);
  });

  it("Reports screen renders within budget", async () => {
    const start = Date.now();
    await navigateTo("Reports");
    await $("=Monthly data").waitForExist({ timeout: 5000 });
    const elapsed = Date.now() - start;
    console.log(`[perf] Reports: ${elapsed}ms`);
    expect(elapsed).toBeLessThan(SCREEN_BUDGET_MS);
  });

  // AC-45/TR-7: the full hierarchy window itself is explicitly outside the
  // 2s screen budget (FR-10 — a separate window drawing the whole
  // network). What NFR-1 actually binds here is that the *main* console
  // stays responsive while it draws — same claim full-hierarchy.e2e.js's
  // small-scale version makes, now at the real 25,000-member ceiling.
  it("full hierarchy opens at real ceiling scale, main console stays responsive", async () => {
    await navigateTo("Structure");
    const before = await browser.getWindowHandles();
    const openStart = Date.now();
    await $("button=View full hierarchy").click();

    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 5000 });
    await dialog.$("button=Open").click();

    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length > before.length,
      { timeout: 15000, timeoutMsg: "expected a new full-hierarchy window to open" },
    );
    const after = await browser.getWindowHandles();
    const newHandle = after.find((h) => !before.includes(h));
    await browser.switchToWindow(newHandle);
    await $("h1*=Full hierarchy").waitForExist({ timeout: 15000 });
    console.log(`[perf] full hierarchy open (25,000 members): ${Date.now() - openStart}ms`);

    await browser.switchToWindow(before[0]);
    const responsiveStart = Date.now();
    await navigateTo("Home");
    await $("#home-search").waitForExist({ timeout: SCREEN_BUDGET_MS });
    const responsiveElapsed = Date.now() - responsiveStart;
    console.log(`[perf] main console after full-hierarchy open: ${responsiveElapsed}ms`);
    expect(responsiveElapsed).toBeLessThan(SCREEN_BUDGET_MS);

    await browser.switchToWindow(newHandle);
    await browser.closeWindow();
    await browser.switchToWindow(before[0]);
  });
});
