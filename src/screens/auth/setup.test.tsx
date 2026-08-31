import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { Setup } from "./setup";
import * as m8Auth from "@/lib/ipc/m8-auth";
import * as authContext from "@/lib/auth-context";

function mockAuth(overrides: Partial<ReturnType<typeof authContext.useAuth>> = {}) {
  return vi.spyOn(authContext, "useAuth").mockReturnValue({
    state: "needs-setup",
    markAuthenticated: vi.fn(),
    markLocked: vi.fn(),
    markSignedOut: vi.fn(),
    signOutNotice: null,
    clearSignOutNotice: vi.fn(),
    ...overrides,
  });
}

function renderSetup() {
  return render(
    <MemoryRouter initialEntries={["/auth/setup"]}>
      <Routes>
        <Route path="/auth/setup" element={<Setup />} />
        <Route path="/" element={<div>Home screen</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Setup — PIN mode validation", () => {
  it("rejects a PIN that isn't exactly 6 digits", async () => {
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.type(screen.getByLabelText(/Choose a 6-digit PIN/), "123");
    await user.type(screen.getByLabelText(/Confirm PIN/), "123");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("PIN must be exactly 6 digits.")).toBeInTheDocument();
  });

  it("rejects mismatched PINs", async () => {
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.type(screen.getByLabelText(/Choose a 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "654321");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("PINs do not match.")).toBeInTheDocument();
  });

  it("strips non-digit characters as they're typed", async () => {
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.type(screen.getByLabelText(/Choose a 6-digit PIN/), "1a2b3c4d5e6f");
    expect(screen.getByLabelText(/Choose a 6-digit PIN/)).toHaveValue("123456");
  });
});

describe("Setup — password mode validation", () => {
  it("rejects a password shorter than 8 characters or missing a letter/digit", async () => {
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText(/Choose a password/), "short1");
    await user.type(screen.getByLabelText(/Confirm password/), "short1");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(
      await screen.findByText("Password must be at least 8 characters, with a letter and a number."),
    ).toBeInTheDocument();
  });

  it("rejects mismatched passwords", async () => {
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText(/Choose a password/), "correct-horse-1");
    await user.type(screen.getByLabelText(/Confirm password/), "correct-horse-2");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("Passwords do not match.")).toBeInTheDocument();
  });
});

describe("Setup — the full first-run flow", () => {
  it("calls setup_first_run, reveals recovery codes, and gates entry behind the confirm checkbox", async () => {
    const setupFirstRunSpy = vi.spyOn(m8Auth, "setupFirstRun").mockResolvedValue({
      recoveryCodes: Array.from({ length: 10 }, (_, i) => `CODE-${i}`),
    });
    const markAuthenticated = vi.fn();
    mockAuth({ markAuthenticated });
    const user = userEvent.setup();
    renderSetup();

    await user.type(screen.getByLabelText(/Choose a 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "123456");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(setupFirstRunSpy).toHaveBeenCalledWith({ pin: "123456" });
    expect(
      await screen.findByRole("heading", { name: "Save your recovery codes" }),
    ).toBeInTheDocument();
    expect(screen.getByText("CODE-0")).toBeInTheDocument();
    expect(screen.getByText("CODE-9")).toBeInTheDocument();

    const enterButton = screen.getByRole("button", { name: "Enter the console" });
    expect(enterButton).toBeDisabled();

    await user.click(screen.getByLabelText(/I have saved these recovery codes/));
    expect(enterButton).toBeEnabled();

    await user.click(enterButton);
    expect(markAuthenticated).toHaveBeenCalled();
    expect(await screen.findByText("Home screen")).toBeInTheDocument();
  });

  it("shows the server error and stays on step 0 when setup_first_run fails", async () => {
    vi.spyOn(m8Auth, "setupFirstRun").mockRejectedValue({
      kind: "validation",
      message: "Setup has already run.",
    });
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.type(screen.getByLabelText(/Choose a 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "123456");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("Setup has already run.")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Save your recovery codes" })).not.toBeInTheDocument();
  });

  it("submits a validated password credential", async () => {
    const setupFirstRunSpy = vi.spyOn(m8Auth, "setupFirstRun").mockResolvedValue({
      recoveryCodes: ["ONE-CODE"],
    });
    mockAuth();
    const user = userEvent.setup();
    renderSetup();

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText(/Choose a password/), "correct-horse-1");
    await user.type(screen.getByLabelText(/Confirm password/), "correct-horse-1");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(setupFirstRunSpy).toHaveBeenCalledWith({ password: "correct-horse-1" });
    expect(await screen.findByText("ONE-CODE")).toBeInTheDocument();
  });
});
