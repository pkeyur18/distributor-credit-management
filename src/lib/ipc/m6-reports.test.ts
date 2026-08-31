// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import {
  exportMonthly,
  exportYearlyAverage,
  exportLowContribution,
  listBackups,
  redownloadBackup,
  previewMonthlyData,
  previewYearlyAverage,
} from "./m6-reports";

afterEach(() => {
  clearMocks();
});

describe("m6-reports IPC wrappers", () => {
  it("exportMonthly sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("export_monthly");
      expect(payload).toEqual({
        input: {
          periodMonth: "2026-06",
          sortField: "name",
          sortDirection: "asc",
          outputPath: "/tmp/monthly.csv",
        },
      });
      return { filePath: "/tmp/monthly.csv" };
    });
    await exportMonthly({
      periodMonth: "2026-06",
      sortField: "name",
      sortDirection: "asc",
      outputPath: "/tmp/monthly.csv",
    });
  });

  it("exportYearlyAverage sends flat scalar params, no wrapper", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("export_yearly_average");
      expect(payload).toEqual({
        outputPath: "/tmp/yearly.csv",
        sortField: "avgBusinessVolume",
        sortDirection: "desc",
      });
      return { filePath: "/tmp/yearly.csv" };
    });
    await exportYearlyAverage("/tmp/yearly.csv", "avgBusinessVolume", "desc");
  });

  it("exportLowContribution sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("export_low_contribution");
      expect(payload).toEqual({
        input: { sortField: "name", sortDirection: "asc", outputPath: "/tmp/low.csv" },
      });
      return { filePath: "/tmp/low.csv" };
    });
    await exportLowContribution({ sortField: "name", sortDirection: "asc", outputPath: "/tmp/low.csv" });
  });

  it("listBackups takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("list_backups");
      expect(payload).toEqual({});
      return [];
    });
    await listBackups();
  });

  it("redownloadBackup sends flat scalar params", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("redownload_backup");
      expect(payload).toEqual({ periodId: 1, outputPath: "/tmp/redo.db" });
      return { filePath: "/tmp/redo.db" };
    });
    await redownloadBackup(1, "/tmp/redo.db");
  });

  it("previewMonthlyData sends a flat scalar `periodMonth`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("preview_monthly_data");
      expect(payload).toEqual({ periodMonth: "2026-06" });
      return [];
    });
    await previewMonthlyData("2026-06");
  });

  it("previewYearlyAverage takes no payload", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("preview_yearly_average");
      expect(payload).toEqual({});
      return [];
    });
    await previewYearlyAverage();
  });
});
