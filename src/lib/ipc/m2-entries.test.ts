// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import {
  getPeriodLockStatus,
  recordEntry,
  listPeriodEntries,
  editEntry,
  addClosedMonthEntry,
} from "./m2-entries";

afterEach(() => {
  clearMocks();
});

describe("m2-entries IPC wrappers", () => {
  it("getPeriodLockStatus takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_period_lock_status");
      expect(payload).toEqual({});
      return { recordablePeriodMonths: ["2026-06"], blockingMonth: null };
    });
    await getPeriodLockStatus();
  });

  it("recordEntry sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("record_entry");
      expect(payload).toEqual({
        input: { memberId: 284913, amount: 100000, entryDate: "2026-06-15" },
      });
      return {};
    });
    await recordEntry({ memberId: 284913, amount: 100000, entryDate: "2026-06-15" });
  });

  it("listPeriodEntries sends a flat scalar `periodMonth`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("list_period_entries");
      expect(payload).toEqual({ periodMonth: "2026-06" });
      return { periodMonth: "2026-06", entries: [] };
    });
    await listPeriodEntries("2026-06");
  });

  it("editEntry sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("edit_entry");
      expect(payload).toEqual({ input: { id: 501, amount: 150000, entryDate: "2026-06-15" } });
      return {};
    });
    await editEntry({ id: 501, amount: 150000, entryDate: "2026-06-15" });
  });

  it("addClosedMonthEntry sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("add_closed_month_entry");
      expect(payload).toEqual({
        input: { memberId: 284913, amount: 250000, entryDate: "2026-06-10" },
      });
      return {};
    });
    await addClosedMonthEntry({ memberId: 284913, amount: 250000, entryDate: "2026-06-10" });
  });
});
