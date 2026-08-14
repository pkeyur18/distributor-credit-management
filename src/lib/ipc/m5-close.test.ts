// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { confirmBackupAndClose } from "./m5-close";

afterEach(() => {
  clearMocks();
});

describe("confirmBackupAndClose", () => {
  it("sends its struct payload nested under `input`, matching the Rust command's single struct param", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("confirm_backup_and_close");
      expect(payload).toEqual({
        input: { periodId: 1, externalMediumPath: "/tmp/out.db" },
      });
      return { externalMediumCopyFailed: false };
    });

    await confirmBackupAndClose({ periodId: 1, externalMediumPath: "/tmp/out.db" });
  });
});
