// T-QA.3-2 — test-data seeding helpers. There is no seed-via-file or
// test-only IPC command: the database is SQLCipher-encrypted from its
// first write and the app-data directory is real OS state, not injectable
// at launch. "A known hierarchy and period state" is therefore built the
// only way an operator could — driving the real Setup wizard and Add
// Member modal, exactly as any spec would exercise them anyway. These are
// as much an implicit exercise of US-M8.1/US-M1.1's UI paths as they are
// setup.
export async function completeFirstRunSetup(pin) {
  // Setup only renders once the webview has loaded and mounted (the
  // check_data_readable round-trip itself is trivial sync fs I/O, not the
  // bottleneck) — CI's xvfb/no-GPU webkit2gtk cold start alone has been
  // observed taking ~30s before the window even exists, on top of bundle
  // parse/mount. 15s wasn't enough; give it real headroom.
  await $("#setup-pin").waitForExist({ timeout: 45000 });
  await $("#setup-pin").setValue(pin);
  await $("#setup-pin2").setValue(pin);
  await $("button=Continue").click();
  await $("#setup-confirm-saved").click();
  await $("button=Enter the console").click();
}

// PIN set once by completeFirstRunSetup, above — every other spec *file*
// gets its own fresh wdio session (and so its own fresh app process; the
// on-disk app-data survives across them, but the in-memory login session
// does not), landing on Login rather than still-authenticated. Every spec
// file but the first one needs this as its very first action.
export const FIRST_RUN_PIN = "482913";

export async function login(pin) {
  // Same cold-start reality as completeFirstRunSetup — this is also a
  // fresh process launch.
  await $("h1*=Member Rewards Console").waitForExist({ timeout: 45000 });
  await browser.keys(pin.split(""));
  await $("nav").waitForExist({ timeout: 5000 });
}

export async function navigateTo(navLabel) {
  await $("nav").$(`a=${navLabel}`).click();
}

// T-M1.1-1: member IDs are random (100001-999999), never sequential — a
// helper can't just guess the ID a save produced. Search by the phone
// number the caller chose (deterministic, under the caller's control),
// click through to Member Detail, and read the ID back off the URL.
export async function idOfPhone(phone) {
  await navigateTo("Home");
  await $("#home-search").setValue(phone);
  const row = $(`button*=${phone}`);
  await row.waitForExist({ timeout: 3000 });
  await row.click();
  const url = await browser.getUrl();
  const match = url.match(/\/member\/(\d+)/);
  return match[1];
}

// US-M8.1's setup wizard creates no member of its own — the first member
// still has to go through Add Member, which (US-M1.1, fixed S8) skips the
// introducer requirement when the directory is genuinely empty and calls
// `create_root_member` instead of `add_member`.
export async function addRootMember({ name, phone, address }) {
  await navigateTo("Home");
  await $("button=Add member").click();
  await $("#member-name").setValue(name);
  await $("#member-phone").setValue(phone);
  await $("#member-address").setValue(address);
  await $("#member-consent").click();
  await $("button=Save").click();
  // The dialog's backdrop is `fixed inset-0` and fades out on close (see
  // ui/dialog.tsx) — it covers the whole viewport, including the sidebar,
  // for the length of that transition. idOfPhone below clicks the nav
  // immediately; without this wait that click races the fade-out and,
  // under sustained load (many members added back-to-back), loses.
  await $('div[role="dialog"]').waitForExist({ timeout: 3000, reverse: true });
  return idOfPhone(phone);
}

export async function addMember({ name, phone, address, referenceId }) {
  await navigateTo("Home");
  await $("button=Add member").click();
  await $("#member-name").setValue(name);
  await $("#member-phone").setValue(phone);
  await $("#member-address").setValue(address);
  await $("#member-ref-search").setValue(referenceId);
  // T-M1.1-3/T-M1.4-1: reference resolution is a search-and-select through
  // the same shared `SearchResultsList` every lookup box in the console
  // uses (a debounced 200ms query, then a button per result row).
  const resultRow = $(`button*=#${referenceId}`);
  await resultRow.waitForExist({ timeout: 3000 });
  await resultRow.click();
  await $("#member-consent").click();
  await $("button=Save").click();
  // Same race as addRootMember, above.
  await $('div[role="dialog"]').waitForExist({ timeout: 3000, reverse: true });
  return idOfPhone(phone);
}

// US-M2.1 (§5.4). One amount field, the fast path — no currency, no mode
// toggle. Defaults to today's date, which is always within the current
// (only) recordable month until US-M2.3/M2.5 (S12/S13) exist.
export async function recordEntry({ memberName, amount }) {
  await navigateTo("Volume Entry");
  await $("#entry-search").setValue(memberName);
  // Plain `button*=${memberName}` also matches the page's own "‹ Back to
  // {memberName}" breadcrumb whenever the member just came from their own
  // Detail page (addMember/idOfPhone leave you there) — that breadcrumb
  // sits above the search results, so a bare substring match clicks it
  // instead and bounces back to Member Detail. Exclude it explicitly.
  const result = $(
    `.//button[contains(., "${memberName}") and not(starts-with(normalize-space(.), "Back to"))]`,
  );
  await result.waitForExist({ timeout: 3000 });
  await result.click();
  await $("#entry-amount").setValue(amount);
  await $("button=Save entry").click();
  // API-41's period table refetches after a successful save — the new row
  // appearing is the save's own confirmation, not just a UI convenience.
  await $(`td*=${memberName}`).waitForExist({ timeout: 3000 });
}
