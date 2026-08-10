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

  it("carries the field name through a validation error", () => {
    const result = toErrorPresentation({
      kind: "validation",
      message: "Name is required.",
      field: "name",
    });
    expect(result.kind).toBe("validation");
    expect(result.field).toBe("name");
    expect(result.message).toBe("Name is required.");
  });

  it("surfaces a conflict error's message verbatim", () => {
    const result = toErrorPresentation({
      kind: "conflict",
      message: "This phone number is already in use by Asha Patel (#284913).",
    });
    expect(result.message).toContain("Asha Patel");
  });

  it("maps auth_required to a generic sign-in prompt", () => {
    const result = toErrorPresentation({ kind: "auth_required" });
    expect(result.kind).toBe("auth_required");
    expect(result.message).toBe("Sign in to do that.");
  });
});
