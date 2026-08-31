import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { DataRecovery } from "./data-recovery";
import * as preflight from "@/lib/ipc/preflight";
import * as authContext from "@/lib/auth-context";
import * as dialog from "@tauri-apps/plugin-dialog";
import type { BackupRecord } from "@/lib/ipc/entities";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

function mockAuth(overrides: Partial<ReturnType<typeof authContext.useAuth>> = {}) {
  return vi.spyOn(authContext, "useAuth").mockReturnValue({
    state: "needs-recovery",
    markAuthenticated: vi.fn(),
    markLocked: vi.fn(),
    markSignedOut: vi.fn(),
    signOutNotice: null,
    clearSignOutNotice: vi.fn(),
    ...overrides,
  });
}

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/auth/data-recovery" element={<DataRecovery />} />
        <Route path="/auth/login" element={<div>Login screen</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

const BACKUP_A: BackupRecord = {
  id: 5,
  periodId: 2,
  kind: "period_close",
  scheduleKind: null,
  version: 2,
  checksum: "abc",
  isOriginal: false,
  createdAt: "2026-06-01T00:00:00Z",
};
const BACKUP_B: BackupRecord = {
  id: 4,
  periodId: null,
  kind: "manual",
  scheduleKind: null,
  version: 1,
  checksum: "def",
  isOriginal: true,
  createdAt: "2026-05-15T00:00:00Z",
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("DataRecovery — forced entry (data could not be opened)", () => {
  it("lists retained backups newest first and selects the newest by default", async () => {
    vi.spyOn(preflight, "listRestorePoints").mockResolvedValue([BACKUP_A, BACKUP_B]);
    mockAuth();
    renderAt("/auth/data-recovery");

    expect(
      await screen.findByRole("heading", { name: "This console could not open its data" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("button", { name: /Restore from Closed month/ }),
    ).toBeInTheDocument();
  });

  it("shows the empty state when there are no backups yet", async () => {
    vi.spyOn(preflight, "listRestorePoints").mockResolvedValue([]);
    mockAuth();
    renderAt("/auth/data-recovery");

    expect(
      await screen.findByText("No backup has been taken yet, so there is nothing to restore from."),
    ).toBeInTheDocument();
  });

  it("falls back to the empty state if listing restore points fails", async () => {
    vi.spyOn(preflight, "listRestorePoints").mockRejectedValue(new Error("io error"));
    mockAuth();
    renderAt("/auth/data-recovery");

    expect(
      await screen.findByText("No backup has been taken yet, so there is nothing to restore from."),
    ).toBeInTheDocument();
  });

  it("restores the selected backup, signs out, and lands on Login", async () => {
    vi.spyOn(preflight, "listRestorePoints").mockResolvedValue([BACKUP_A, BACKUP_B]);
    const restoreSpy = vi.spyOn(preflight, "restoreFromBackup").mockResolvedValue(undefined);
    const markSignedOut = vi.fn();
    mockAuth({ markSignedOut });
    const user = userEvent.setup();
    renderAt("/auth/data-recovery");

    const restoreButton = await screen.findByRole("button", { name: /Restore from Closed month/ });
    await user.click(restoreButton);

    await waitFor(() => expect(restoreSpy).toHaveBeenCalledWith(BACKUP_A.id));
    expect(markSignedOut).toHaveBeenCalled();
    expect(await screen.findByText("Login screen")).toBeInTheDocument();
  });

  it("shows an inline error and stays put when restoring fails", async () => {
    vi.spyOn(preflight, "listRestorePoints").mockResolvedValue([BACKUP_A]);
    vi.spyOn(preflight, "restoreFromBackup").mockRejectedValue({
      kind: "validation",
      message: "That backup's checksum does not match.",
    });
    mockAuth();
    const user = userEvent.setup();
    renderAt("/auth/data-recovery");

    const restoreButton = await screen.findByRole("button", { name: /Restore from Closed month/ });
    await user.click(restoreButton);

    expect(await screen.findByText("That backup's checksum does not match.")).toBeInTheDocument();
  });

  it("also offers browsing a file, restoring through restore_from_backup_file", async () => {
    vi.spyOn(preflight, "listRestorePoints").mockResolvedValue([BACKUP_A]);
    const restoreFileSpy = vi.spyOn(preflight, "restoreFromBackupFile").mockResolvedValue(undefined);
    vi.mocked(dialog.open).mockResolvedValue("/path/to/backup.sqlite");
    const markSignedOut = vi.fn();
    mockAuth({ markSignedOut });
    const user = userEvent.setup();
    renderAt("/auth/data-recovery");

    await screen.findByRole("button", { name: /Restore from Closed month/ });
    await user.click(screen.getByRole("button", { name: "Browse a different backup file" }));

    await waitFor(() =>
      expect(restoreFileSpy).toHaveBeenCalledWith("/path/to/backup.sqlite"),
    );
    expect(markSignedOut).toHaveBeenCalled();
    expect(await screen.findByText("Login screen")).toBeInTheDocument();
  });
});

describe("DataRecovery — voluntary entry (?from=setup)", () => {
  it("skips the restore-points fetch and goes straight to the file picker", async () => {
    const listSpy = vi.spyOn(preflight, "listRestorePoints");
    mockAuth();
    renderAt("/auth/data-recovery?from=setup");

    expect(
      await screen.findByRole("heading", { name: "Restore from a backup file" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose backup file…" })).toBeInTheDocument();
    expect(listSpy).not.toHaveBeenCalled();
  });

  it("restores from the chosen file and signs out to Login", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/path/from/setup.sqlite");
    const restoreFileSpy = vi.spyOn(preflight, "restoreFromBackupFile").mockResolvedValue(undefined);
    const markSignedOut = vi.fn();
    mockAuth({ markSignedOut });
    const user = userEvent.setup();
    renderAt("/auth/data-recovery?from=setup");

    await user.click(await screen.findByRole("button", { name: "Choose backup file…" }));

    await waitFor(() => expect(restoreFileSpy).toHaveBeenCalledWith("/path/from/setup.sqlite"));
    expect(markSignedOut).toHaveBeenCalled();
    expect(await screen.findByText("Login screen")).toBeInTheDocument();
  });

  it("does nothing when the file dialog is cancelled", async () => {
    vi.mocked(dialog.open).mockResolvedValue(null);
    const restoreFileSpy = vi.spyOn(preflight, "restoreFromBackupFile");
    mockAuth();
    const user = userEvent.setup();
    renderAt("/auth/data-recovery?from=setup");

    await user.click(await screen.findByRole("button", { name: "Choose backup file…" }));

    expect(restoreFileSpy).not.toHaveBeenCalled();
  });
});
