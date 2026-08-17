// T-QA.4: every user-visible string in the build — screen labels, buttons,
// toasts, tooltips, placeholders, error messages, and test fixtures/mock
// data — must stay inside the restricted vocabulary (01-product-and-scope.md
// §3). Scans string literals and JSX text via the TypeScript compiler API
// (already a project dependency) rather than a whole-file regex, so it
// naturally never sees comments or identifiers — only what actually renders
// or ships in an extract.
import ts from "typescript";
import { readFileSync } from "node:fs";
import { glob } from "node:fs/promises";

export const EXCLUDED_WORDS = [
  "sale",
  "purchase",
  "order",
  "cash",
  "payment",
  "commission",
  "invoice",
];

// Ordinary-English uses of "order" that are not the commercial term (T-QA.4-3).
// Checked against the *whole extracted string*, not just the matched word, so
// "Change the sort order of rows" is allowlisted without also allowlisting
// a bare "order" appearing elsewhere in the same string.
const ORDER_ALLOWLIST = [
  /\bsort order\b/i,
  /\bpost-order\b/i,
  /\bpre-order\b/i, // traversal terms, not the commercial verb
  /\bin-order\b/i,
  /\bout of order\b/i,
  /(^|\s)-?order-(first|last|none|\d+)(\s|$)/i, // Tailwind's `order-*` utility class
];

function isAllowlisted(word, text) {
  if (word !== "order") return false;
  return ORDER_ALLOWLIST.some((pattern) => pattern.test(text));
}

function extractTextNodes(sourceText, fileName) {
  const scriptKind = fileName.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
  const sourceFile = ts.createSourceFile(
    fileName,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    scriptKind,
  );

  /** @type {{text: string, pos: number}[]} */
  const nodes = [];

  function visit(node) {
    if (
      ts.isStringLiteralLike(node) ||
      node.kind === ts.SyntaxKind.TemplateHead ||
      node.kind === ts.SyntaxKind.TemplateMiddle ||
      node.kind === ts.SyntaxKind.TemplateTail
    ) {
      nodes.push({ text: node.text, pos: node.getStart(sourceFile) });
    } else if (ts.isJsxText(node)) {
      const text = node.getText(sourceFile).trim();
      if (text) nodes.push({ text, pos: node.getStart(sourceFile) });
    }
    ts.forEachChild(node, visit);
  }
  visit(sourceFile);

  return nodes.map(({ text, pos }) => ({
    text,
    line: sourceFile.getLineAndCharacterOfPosition(pos).line + 1,
  }));
}

/**
 * @param {string} sourceText
 * @param {string} fileName
 * @returns {{file: string, line: number, word: string, text: string}[]}
 */
export function findViolations(sourceText, fileName) {
  const violations = [];
  for (const { text, line } of extractTextNodes(sourceText, fileName)) {
    for (const word of EXCLUDED_WORDS) {
      const re = new RegExp(`\\b${word}\\b`, "i");
      if (re.test(text) && !isAllowlisted(word, text)) {
        violations.push({ file: fileName, line, word, text });
      }
    }
  }
  return violations;
}

export async function scanFiles(root = "src") {
  const violations = [];
  for await (const path of glob(`${root}/**/*.{ts,tsx}`, {
    exclude: ["**/*.test.ts", "**/*.test.tsx"],
  })) {
    const contents = readFileSync(path, "utf8");
    violations.push(...findViolations(contents, path));
  }
  return violations;
}

async function main() {
  const violations = await scanFiles();
  if (violations.length === 0) {
    console.log("vocabulary grep: clean");
    return;
  }
  console.error(`vocabulary grep: ${violations.length} violation(s) found\n`);
  for (const v of violations) {
    console.error(`  ${v.file}:${v.line} — "${v.word}" in: ${v.text}`);
  }
  process.exit(1);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
