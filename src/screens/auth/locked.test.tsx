import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { Locked } from "./locked";
import * as m8Auth from "@/lib/ipc/m8-auth";
import * as authContext from "@/lib/auth-context";

function mockAuth(overrides: Partial<ReturnType<typeof authContext.useAuth>> = {}) {
  return vi.spyOn(authContext, "useAuth").mockReturnValue({
    state: "locked",
    markAuthenticated: vi.fn(),
    markLocked: vi.fn(),
    markSignedOut: vi.fn(),
    signOutNotice: null,
    clearSignOutNotice: vi.fn(),
    ...overrides,
  });
}

function renderLocked() {
  return render(
    <MemoryRouter initialEntries={["/auth/locked"]}>
      <Routes>
        <Route path="/auth/locked" element={<Locked />} />
        <Route path="/" element={<div>Home screen</div>} />
        <Route path="/auth/login" element={<div>Login screen</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

async function typeDigits(user: ReturnType<typeof userEvent.setup>, digits: string) {
  for (const digit of digits) {
    await user.click(screen.getByRole("button", { name: digit }));
  }
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Locked — resuming the session", () => {
  it("auto-submits at the 6th PIN digit, marks authenticated and returns home", async () => {
    const unlockSpy = vi.spyOn(m8Auth, "unlockSession").mockResolvedValue(undefined);
    const markAuthenticated = vi.fn();
    mockAuth({ markAuthenticated });
    const user = userEvent.setup();
    renderLocked();

    await typeDigits(user, "123456");

    await waitFor(() => expect(unlockSpy).toHaveBeenCalledWith({ pin: "123456" }));
    await waitFor(() => expect(markAuthenticated).toHaveBeenCalled());
    expect(await screen.findByText("Home screen")).toBeInTheDocument();
  });

  it("unlocks with a password", async () => {
    const unlockSpy = vi.spyOn(m8Auth, "unlockSession").mockResolvedValue(undefined);
    mockAuth();
    const user = userEvent.setup();
    renderLocked();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText("Password"), "correct-horse-1");
    await user.click(screen.getByRole("button", { name: "Unlock" }));

    await waitFor(() => expect(unlockSpy).toHaveBeenCalledWith({ password: "correct-horse-1" }));
  });

  it("shows the invalid-credential message on a wrong PIN", async () => {
    vi.spyOn(m8Auth, "unlockSession").mockRejectedValue({ kind: "invalid_credential", attemptsRemaining: 2 });
    mockAuth();
    const user = userEvent.setup();
    renderLocked();

    await typeDigits(user, "000000");

    expect(await screen.findByText(/2 attempts remaining/)).toBeInTheDocument();
  });

  it("shows a lockout countdown and hides the keypad and 'Sign out instead' link", async () => {
    vi.spyOn(m8Auth, "unlockSession").mockRejectedValue({
      kind: "account_locked",
      retryAfterSeconds: 60,
    });
    mockAuth();
    const user = userEvent.setup();
    renderLocked();

    await typeDigits(user, "000000");

    expect(await screen.findByText("Too many attempts")).toBeInTheDocument();
    expect(screen.getByText("60s")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "1" })).not.toBeInTheDocument();
    expect(screen.queryByText("Sign out instead")).not.toBeInTheDocument();
  });
});

describe("Locked — signing out instead", () => {
  it("calls markSignedOut and routes to Login without calling unlock_session", async () => {
    const unlockSpy = vi.spyOn(m8Auth, "unlockSession");
    const markSignedOut = vi.fn();
    mockAuth({ markSignedOut });
    const user = userEvent.setup();
    renderLocked();

    await user.click(screen.getByText("Sign out instead"));

    expect(markSignedOut).toHaveBeenCalledWith();
    expect(await screen.findByText("Login screen")).toBeInTheDocument();
    expect(unlockSpy).not.toHaveBeenCalled();
  });
});
