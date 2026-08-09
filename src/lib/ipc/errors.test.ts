import { describe, expect, it } from "vitest";
import { toErrorPresentation } from "./errors";

describe("toErrorPresentation", () => {
  it("maps a known kind to its presentation message", () => {
    const result = toErrorPresentation({ kind: "database", message: "constraint failed" });
    expect(result.kind).toBe("database");
    expect(result.message).toBe("Something went wrong saving that. Try again.");
  });

  it("interpolates month/blockingMonth into the period_not_accepting_entries message", () => {
    const result = toErrorPresentation({
      kind: "period_not_accepting_entries",
      month: "August 2026",
      blockingMonth: "June 2026",
    });
    expect(result.kind).toBe("period_not_accepting_entries");
    expect(result.message).toContain("August 2026");
    expect(result.message).toContain("June 2026");
  });

  it("maps period_closed with its month", () => {
    const result = toErrorPresentation({ kind: "period_closed", month: "May 2026" });
    expect(result.message).toContain("May 2026");
  });

  it("falls back to 'unknown' for an unrecognized kind, never inventing a new presentation silently", () => {
    const result = toErrorPresentation({ kind: "period_locked", message: "stale variant" });
    expect(result.kind).toBe("unknown");
    expect(result.message).toBe("stale variant");
  });

  it("falls back to 'unknown' for a non-object thrown value", () => {
    const result = toErrorPresentation("plain string failure");
    expect(result.kind).toBe("unknown");
    expect(result.message).toBe("Something went wrong.");
  });
});
