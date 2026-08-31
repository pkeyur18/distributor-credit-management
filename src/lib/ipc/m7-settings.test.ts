// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import {
  getSettings,
  updateSettings,
  addSlabRow,
  removeSlabRow,
  updateSlabRow,
  getConsoleBackupSettings,
  updateConsoleBackupSettings,
} from "./m7-settings";

afterEach(() => {
  clearMocks();
});

describe("m7-settings IPC wrappers", () => {
  it("getSettings takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_settings");
      expect(payload).toEqual({});
      return {};
    });
    await getSettings();
  });

  it("updateSettings sends the patch nested under `patch`, not `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("update_settings");
      expect(payload).toEqual({ patch: { royaltyQualifyingCount: 4 } });
      return {};
    });
    await updateSettings({ royaltyQualifyingCount: 4 });
  });

  it("addSlabRow spreads the input flat, no wrapper", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("add_slab_row");
      expect(payload).toEqual({ threshold: 50000, percentage: 10 });
      return {};
    });
    await addSlabRow({ threshold: 50000, percentage: 10 });
  });

  it("removeSlabRow sends a flat scalar `id`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("remove_slab_row");
      expect(payload).toEqual({ id: 3 });
      return null;
    });
    await removeSlabRow(3);
  });

  it("updateSlabRow sends flat `id` plus the spread input", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("update_slab_row");
      expect(payload).toEqual({ id: 3, threshold: 60000, percentage: 12 });
      return {};
    });
    await updateSlabRow(3, { threshold: 60000, percentage: 12 });
  });

  it("getConsoleBackupSettings takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_console_backup_settings");
      expect(payload).toEqual({});
      return { schedule: "off", retentionCount: 5, folder: "" };
    });
    await getConsoleBackupSettings();
  });

  it("updateConsoleBackupSettings spreads the input flat, no wrapper", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("update_console_backup_settings");
      expect(payload).toEqual({ schedule: "weekly", retentionCount: 10, folder: "/backups" });
      return {};
    });
    await updateConsoleBackupSettings({ schedule: "weekly", retentionCount: 10, folder: "/backups" });
  });
});
