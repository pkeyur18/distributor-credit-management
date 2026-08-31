import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { Login } from "./login";
import * as m8Auth from "@/lib/ipc/m8-auth";
import * as authContext from "@/lib/auth-context";

function mockAuth(overrides: Partial<ReturnType<typeof authContext.useAuth>> = {}) {
  return vi.spyOn(authContext, "useAuth").mockReturnValue({
    state: "needs-login",
    markAuthenticated: vi.fn(),
    markLocked: vi.fn(),
    markSignedOut: vi.fn(),
    signOutNotice: null,
    clearSignOutNotice: vi.fn(),
    ...overrides,
  });
}

function renderLogin() {
  return render(
    <MemoryRouter initialEntries={["/auth/login"]}>
      <Routes>
        <Route path="/auth/login" element={<Login />} />
        <Route path="/" element={<div>Home screen</div>} />
        <Route path="/auth/data-recovery" element={<div>Data recovery screen</div>} />
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

describe("Login — PIN mode", () => {
  it("auto-submits at the 6th digit, marks authenticated and navigates home on success", async () => {
    const loginSpy = vi.spyOn(m8Auth, "login").mockResolvedValue(undefined);
    const markAuthenticated = vi.fn();
    mockAuth({ markAuthenticated });
    const user = userEvent.setup();
    renderLogin();

    await typeDigits(user, "123456");

    await waitFor(() => expect(loginSpy).toHaveBeenCalledWith({ pin: "123456" }));
    await waitFor(() => expect(markAuthenticated).toHaveBeenCalled());
    expect(await screen.findByText("Home screen")).toBeInTheDocument();
  });

  it("shows the invalid-credential message with attempts remaining, and clears the buffer", async () => {
    vi.spyOn(m8Auth, "login").mockRejectedValue({
      kind: "invalid_credential",
      attemptsRemaining: 4,
    });
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await typeDigits(user, "111111");

    expect(await screen.findByText(/4 attempts remaining/)).toBeInTheDocument();
    expect(screen.queryByText("1")).toBeInTheDocument(); // keypad still visible, buffer reset
  });

  it("shows a countdown when the account is locked, hiding the keypad", async () => {
    vi.spyOn(m8Auth, "login").mockRejectedValue({
      kind: "account_locked",
      retryAfterSeconds: 45,
    });
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await typeDigits(user, "000000");

    expect(await screen.findByText("Too many attempts")).toBeInTheDocument();
    expect(screen.getByText("45s")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "1" })).not.toBeInTheDocument();
  });

  it("routes to data-recovery on a data_unreadable failure instead of showing an error", async () => {
    vi.spyOn(m8Auth, "login").mockRejectedValue({ kind: "data_unreadable" });
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await typeDigits(user, "222222");

    expect(await screen.findByText("Data recovery screen")).toBeInTheDocument();
  });

  it("supports physical keyboard digits and Backspace", async () => {
    const loginSpy = vi.spyOn(m8Auth, "login").mockResolvedValue(undefined);
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await user.keyboard("12345");
    await user.keyboard("{Backspace}");
    await user.keyboard("59");

    await waitFor(() => expect(loginSpy).toHaveBeenCalledWith({ pin: "123459" }));
  });

  it("ignores keyboard digits while locked out", async () => {
    const loginSpy = vi
      .spyOn(m8Auth, "login")
      .mockRejectedValue({ kind: "account_locked", retryAfterSeconds: 30 });
    mockAuth();
    const user = userEvent.setup();
    renderLogin();
    await typeDigits(user, "000000");
    expect(await screen.findByText("Too many attempts")).toBeInTheDocument();

    loginSpy.mockClear();
    await user.keyboard("123456");
    expect(loginSpy).not.toHaveBeenCalled();
  });
});

describe("Login — password mode", () => {
  it("submits the typed password, marks authenticated and navigates home on success", async () => {
    const loginSpy = vi.spyOn(m8Auth, "login").mockResolvedValue(undefined);
    const markAuthenticated = vi.fn();
    mockAuth({ markAuthenticated });
    const user = userEvent.setup();
    renderLogin();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText("Password"), "correct-horse-1");
    await user.click(screen.getByRole("button", { name: "Unlock" }));

    await waitFor(() => expect(loginSpy).toHaveBeenCalledWith({ password: "correct-horse-1" }));
    await waitFor(() => expect(markAuthenticated).toHaveBeenCalled());
    expect(await screen.findByText("Home screen")).toBeInTheDocument();
  });

  it("submits on Enter as well as the Unlock button", async () => {
    const loginSpy = vi.spyOn(m8Auth, "login").mockResolvedValue(undefined);
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText("Password"), "correct-horse-1{Enter}");

    await waitFor(() => expect(loginSpy).toHaveBeenCalledWith({ password: "correct-horse-1" }));
  });

  it("keeps Unlock disabled until a password is typed", async () => {
    mockAuth();
    const user = userEvent.setup();
    renderLogin();
    await user.click(screen.getByRole("radio", { name: "Password" }));
    expect(screen.getByRole("button", { name: "Unlock" })).toBeDisabled();
  });

  it("shows the generic invalid-credential message on a wrong password", async () => {
    vi.spyOn(m8Auth, "login").mockRejectedValue({ kind: "invalid_credential" });
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText("Password"), "wrong-pass1");
    await user.click(screen.getByRole("button", { name: "Unlock" }));

    expect(await screen.findByText("Incorrect PIN or password.")).toBeInTheDocument();
  });

  it("switching mode resets the PIN buffer and any error", async () => {
    vi.spyOn(m8Auth, "login").mockRejectedValue({ kind: "invalid_credential" });
    mockAuth();
    const user = userEvent.setup();
    renderLogin();

    await typeDigits(user, "111111");
    expect(await screen.findByText("Incorrect PIN or password.")).toBeInTheDocument();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    expect(screen.queryByText("Incorrect PIN or password.")).not.toBeInTheDocument();
  });
});

describe("Login — sign-out notice", () => {
  it("shows a carried-over sign-out notice", () => {
    mockAuth({ signOutNotice: "Restore complete." });
    renderLogin();
    expect(screen.getByText("Restore complete.")).toBeInTheDocument();
  });

  it("shows nothing when there is no notice", () => {
    mockAuth({ signOutNotice: null });
    renderLogin();
    expect(screen.queryByText(/restore complete/i)).not.toBeInTheDocument();
  });

  it("clears the notice on unmount", () => {
    const clearSignOutNotice = vi.fn();
    mockAuth({ signOutNotice: "Restore complete.", clearSignOutNotice });
    const { unmount } = renderLogin();
    expect(clearSignOutNotice).not.toHaveBeenCalled();
    unmount();
    expect(clearSignOutNotice).toHaveBeenCalled();
  });
});
