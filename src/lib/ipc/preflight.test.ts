// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import {
  checkDataReadable,
  listRestorePoints,
  restoreFromBackup,
  restoreFromBackupFile,
} from "./preflight";

afterEach(() => {
  clearMocks();
});

describe("preflight IPC wrappers", () => {
  it("checkDataReadable takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("check_data_readable");
      expect(payload).toEqual({});
      return true;
    });
    await checkDataReadable();
  });

  it("listRestorePoints takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("list_restore_points");
      expect(payload).toEqual({});
      return [];
    });
    await listRestorePoints();
  });

  it("restoreFromBackup sends a flat scalar `backupId`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("restore_from_backup");
      expect(payload).toEqual({ backupId: 5 });
      return null;
    });
    await restoreFromBackup(5);
  });

  it("restoreFromBackupFile sends a flat scalar `filePath`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("restore_from_backup_file");
      expect(payload).toEqual({ filePath: "/tmp/backup.sqlite" });
      return null;
    });
    await restoreFromBackupFile("/tmp/backup.sqlite");
  });
});
