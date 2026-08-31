import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Audit } from "./audit";
import * as m9Audit from "@/lib/ipc/m9-audit";
import type { AuditLogEntry } from "@/lib/ipc/entities";

function entry(overrides: Partial<AuditLogEntry> = {}): AuditLogEntry {
  return {
    id: 1,
    entityType: "member",
    entityId: 284913,
    field: "phone",
    oldValue: "9876500000",
    newValue: "9876543210",
    changedAt: "2026-06-15T10:00:00Z",
    cause: "edit",
    memberName: "Asha Patel",
    ...overrides,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Audit", () => {
  it("loads with no filter and renders every column of a row", async () => {
    vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue([entry()]);
    render(<Audit />);

    expect(await screen.findByText("Asha Patel")).toBeInTheDocument();
    expect(screen.getByText("phone")).toBeInTheDocument();
    expect(screen.getByText("9876500000")).toBeInTheDocument();
    expect(screen.getByText("9876543210")).toBeInTheDocument();
    expect(screen.getByText("Record edited")).toBeInTheDocument();
  });

  it("shows an em dash for a null member name and null before/after values", async () => {
    vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue([
      entry({ memberName: null, oldValue: null, field: "royalty_min_children" }),
    ]);
    render(<Audit />);

    await screen.findByText("royalty_min_children");
    const dashes = screen.getAllByText("—");
    expect(dashes.length).toBeGreaterThanOrEqual(2); // member name + old value
  });

  it("maps every audit cause to its human-readable label", async () => {
    const causes: AuditLogEntry["cause"][] = [
      "entry",
      "edit",
      "correction",
      "settings_change",
      "period_close",
      "manual_backup",
      "console_backup",
    ];
    vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue(
      causes.map((cause, i) => entry({ id: i, cause, field: `field-${i}` })),
    );
    render(<Audit />);

    await screen.findByText("field-0");
    expect(screen.getByText("New record recorded")).toBeInTheDocument();
    expect(screen.getByText("Record edited")).toBeInTheDocument();
    expect(
      screen.getByText("Closed-month record corrected — new snapshot version created"),
    ).toBeInTheDocument();
    expect(screen.getByText("Setting changed by administrator")).toBeInTheDocument();
    expect(
      screen.getByText("Month closed — permanent record written, live figures cleared"),
    ).toBeInTheDocument();
    expect(screen.getByText("Manual backup created")).toBeInTheDocument();
    expect(screen.getByText("Console backup created")).toBeInTheDocument();
  });

  it("shows the empty state when nothing matches", async () => {
    vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue([]);
    render(<Audit />);

    expect(await screen.findByText("No matching entries")).toBeInTheDocument();
  });

  it("debounces the filter and calls get_audit_log with the trimmed memberQuery", async () => {
    const auditSpy = vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue([entry()]);
    const user = userEvent.setup();
    render(<Audit />);

    await waitFor(() => expect(auditSpy).toHaveBeenCalledWith({}));
    auditSpy.mockClear();
    auditSpy.mockResolvedValue([]);

    await user.type(screen.getByPlaceholderText(/Filter by member name/i), "  9876500000  ");

    await waitFor(() => expect(auditSpy).toHaveBeenCalledWith({ memberQuery: "9876500000" }));
  });

  it("shows the recorded-changes count in the subtitle, singular for exactly one", async () => {
    vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue([entry()]);
    render(<Audit />);

    expect(await screen.findByText(/1 recorded change —/)).toBeInTheDocument();
  });

  it("pluralises the subtitle for more than one entry", async () => {
    vi.spyOn(m9Audit, "getAuditLog").mockResolvedValue([entry({ id: 1 }), entry({ id: 2 })]);
    render(<Audit />);

    expect(await screen.findByText(/2 recorded changes —/)).toBeInTheDocument();
  });
});
