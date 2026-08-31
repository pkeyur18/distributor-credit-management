// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { getAuditLog } from "./m9-audit";

afterEach(() => {
  clearMocks();
});

describe("m9-audit IPC wrappers", () => {
  it("getAuditLog spreads an empty filter flat when omitted", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_audit_log");
      expect(payload).toEqual({});
      return [];
    });
    await getAuditLog();
  });

  it("getAuditLog spreads a memberQuery filter flat, not wrapped", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_audit_log");
      expect(payload).toEqual({ memberQuery: "9876500000" });
      return [];
    });
    await getAuditLog({ memberQuery: "9876500000" });
  });
});
