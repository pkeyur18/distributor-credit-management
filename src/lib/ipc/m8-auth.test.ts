// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import {
  setupFirstRun,
  login,
  lockSession,
  unlockSession,
  useRecoveryCode,
  getOutstandingAlert,
  runConsoleBackupNow,
} from "./m8-auth";

afterEach(() => {
  clearMocks();
});

describe("m8-auth IPC wrappers", () => {
  it("setupFirstRun sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("setup_first_run");
      expect(payload).toEqual({ input: { pin: "123456" } });
      return { recoveryCodes: [] };
    });
    await setupFirstRun({ pin: "123456" });
  });

  it("login sends the credential nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("login");
      expect(payload).toEqual({ input: { pin: "123456" } });
      return null;
    });
    await login({ pin: "123456" });
  });

  it("lockSession takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("lock_session");
      expect(payload).toEqual({});
      return null;
    });
    await lockSession();
  });

  it("unlockSession sends the credential nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("unlock_session");
      expect(payload).toEqual({ input: { password: "correct-horse-1" } });
      return null;
    });
    await unlockSession({ password: "correct-horse-1" });
  });

  it("useRecoveryCode sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("use_recovery_code");
      expect(payload).toEqual({ input: { code: "ABCDE-FGHIJ-KLMNO", newPin: "123456" } });
      return { recoveryCodes: [] };
    });
    await useRecoveryCode({ code: "ABCDE-FGHIJ-KLMNO", newPin: "123456" });
  });

  it("getOutstandingAlert takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_outstanding_alert");
      expect(payload).toEqual({});
      return { outstandingMonths: [], currentMonth: "2026-06" };
    });
    await getOutstandingAlert();
  });

  it("runConsoleBackupNow takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("run_console_backup_now");
      expect(payload).toEqual({});
      return {};
    });
    await runConsoleBackupNow();
  });
});
