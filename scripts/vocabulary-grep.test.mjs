import { test } from "node:test";
import assert from "node:assert/strict";
import { findViolations, EXCLUDED_WORDS } from "./vocabulary-grep.mjs";

test("EXCLUDED_WORDS matches 01-product-and-scope.md §3 exactly", () => {
  assert.deepEqual(
    [...EXCLUDED_WORDS].sort(),
    ["cash", "commission", "invoice", "order", "payment", "purchase", "sale"].sort(),
  );
});

test("flags a banned word inside JSX text", () => {
  const src = `export const X = () => <button>Record a sale</button>;`;
  const violations = findViolations(src, "X.tsx");
  assert.equal(violations.length, 1);
  assert.equal(violations[0].word, "sale");
});

test("flags a banned word inside a plain string literal", () => {
  const src = `const msg = "Payment could not be processed";`;
  const violations = findViolations(src, "x.ts");
  assert.equal(violations.length, 1);
  assert.equal(violations[0].word, "payment");
});

test("flags a banned word inside a template literal", () => {
  const src = "const msg = `Invoice for ${name}`;";
  const violations = findViolations(src, "x.ts");
  assert.equal(violations.length, 1);
  assert.equal(violations[0].word, "invoice");
});

test("is case-insensitive", () => {
  const src = `export const X = () => <h1>Order History</h1>;`;
  const violations = findViolations(src, "X.tsx");
  assert.equal(violations.length, 1);
  assert.equal(violations[0].word, "order");
});

test("does not flag 'sort order' as ordinary English", () => {
  const src = `const label = "Change the sort order of rows";`;
  assert.deepEqual(findViolations(src, "x.ts"), []);
});

test("does not flag 'post-order traversal' as ordinary English", () => {
  const src = `const label = "post-order traversal of the tree";`;
  assert.deepEqual(findViolations(src, "x.ts"), []);
});

test("does not flag 'out of order' as ordinary English", () => {
  const src = `const label = "closing out of order is refused";`;
  assert.deepEqual(findViolations(src, "x.ts"), []);
});

test("does not flag a Tailwind order-N utility class string", () => {
  const src = `export const X = () => <div className="order-2 flex" />;`;
  assert.deepEqual(findViolations(src, "X.tsx"), []);
});

test("does not flag code comments", () => {
  const src = `// process the sale record for audit\nconst x = 1;`;
  assert.deepEqual(findViolations(src, "x.ts"), []);
});

test("does not flag identifiers like sortOrder", () => {
  const src = `const sortOrder = row.sortOrder;`;
  assert.deepEqual(findViolations(src, "x.ts"), []);
});

test("reports the 1-indexed line number", () => {
  const src = `const a = 1;\nconst b = 2;\nconst msg = "cash only";`;
  const violations = findViolations(src, "x.ts");
  assert.equal(violations.length, 1);
  assert.equal(violations[0].line, 3);
});

test("flags each distinct banned word found, even in one file", () => {
  const src = `const a = "sale"; const b = "purchase"; const c = "commission";`;
  const violations = findViolations(src, "x.ts");
  assert.deepEqual(violations.map((v) => v.word).sort(), ["commission", "purchase", "sale"]);
});
