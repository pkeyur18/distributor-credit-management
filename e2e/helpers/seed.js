// T-QA.3-2 — test-data seeding helpers. There is no seed-via-file or
// test-only IPC command: the database is SQLCipher-encrypted from its
// first write and the app-data directory is real OS state, not injectable
// at launch. "A known hierarchy and period state" is therefore built the
// only way an operator could — driving the real Setup wizard and Add
// Member modal, exactly as any spec would exercise them anyway. These are
// as much an implicit exercise of US-M8.1/US-M1.1's UI paths as they are
// setup.
export async function completeFirstRunSetup(pin) {
  await $("#setup-pin").setValue(pin);
  await $("#setup-pin2").setValue(pin);
  await $("button=Continue").click();
  await $("#setup-confirm-saved").click();
  await $("button=Enter the console").click();
}

export async function addRootMember({ name, phone, address }) {
  await $("button=Add member").click();
  await $("#member-name").setValue(name);
  await $("#member-phone").setValue(phone);
  await $("#member-address").setValue(address);
  await $("#member-consent").click();
  await $("button=Save").click();
}

export async function addMember({ name, phone, address, referenceId }) {
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
}

export async function navigateTo(navLabel) {
  await $(`nav a=${navLabel}`).click();
}
