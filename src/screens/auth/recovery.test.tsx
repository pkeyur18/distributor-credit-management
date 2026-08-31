import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { Recovery } from "./recovery";
import * as m8Auth from "@/lib/ipc/m8-auth";

function renderRecovery() {
  return render(
    <MemoryRouter initialEntries={["/auth/recovery"]}>
      <Routes>
        <Route path="/auth/recovery" element={<Recovery />} />
        <Route path="/auth/login" element={<div>Login screen</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

async function enterCode(user: ReturnType<typeof userEvent.setup>, code = "ABCDE-FGHIJ-KLMNO") {
  await user.type(screen.getByLabelText("Recovery code"), code);
  await user.click(screen.getByRole("button", { name: "Verify code" }));
}

describe("Recovery — step 0: code entry", () => {
  it("refuses to advance on an empty code", async () => {
    const user = userEvent.setup();
    renderRecovery();

    await user.click(screen.getByRole("button", { name: "Verify code" }));

    // The same sentence is both the screen's subtitle and the error hint,
    // so a second match is what proves the hint rendered.
    await waitFor(() =>
      expect(
        screen.getAllByText("Enter the recovery code you saved when this console was first set up."),
      ).toHaveLength(2),
    );
    expect(screen.queryByText("Set a new credential")).not.toBeInTheDocument();
  });

  it("advances to step 1 once a non-empty code is entered", async () => {
    const user = userEvent.setup();
    renderRecovery();

    await enterCode(user);

    expect(await screen.findByRole("heading", { name: "Set a new credential" })).toBeInTheDocument();
  });
});

describe("Recovery — step 1: new credential validation", () => {
  it("rejects a malformed new PIN", async () => {
    const user = userEvent.setup();
    renderRecovery();
    await enterCode(user);

    await user.type(screen.getByLabelText(/New 6-digit PIN/), "12");
    await user.type(screen.getByLabelText(/Confirm PIN/), "12");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("PIN must be exactly 6 digits.")).toBeInTheDocument();
  });

  it("rejects mismatched new PINs", async () => {
    const user = userEvent.setup();
    renderRecovery();
    await enterCode(user);

    await user.type(screen.getByLabelText(/New 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "654321");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("PINs do not match.")).toBeInTheDocument();
  });

  it("rejects a weak new password", async () => {
    const user = userEvent.setup();
    renderRecovery();
    await enterCode(user);

    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText(/New password/), "short1");
    await user.type(screen.getByLabelText(/Confirm password/), "short1");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(
      await screen.findByText("Password must be at least 8 characters, with a letter and a number."),
    ).toBeInTheDocument();
  });
});

describe("Recovery — completing the flow", () => {
  it("calls use_recovery_code with the trimmed code and new PIN, then shows fresh recovery codes", async () => {
    const useRecoveryCodeSpy = vi.spyOn(m8Auth, "useRecoveryCode").mockResolvedValue({
      recoveryCodes: ["FRESH-CODE-1", "FRESH-CODE-2"],
    });
    const user = userEvent.setup();
    renderRecovery();

    await enterCode(user, "  ABCDE-FGHIJ-KLMNO  ");
    await user.type(screen.getByLabelText(/New 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "123456");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(useRecoveryCodeSpy).toHaveBeenCalledWith({
      code: "ABCDE-FGHIJ-KLMNO",
      newPin: "123456",
    });
    expect(
      await screen.findByRole("heading", { name: "Save your new recovery codes" }),
    ).toBeInTheDocument();
    expect(screen.getByText("FRESH-CODE-1")).toBeInTheDocument();

    const finishButton = screen.getByRole("button", { name: "Sign in with new credential" });
    expect(finishButton).toBeDisabled();
    await user.click(screen.getByLabelText(/I have saved these recovery codes/));
    await user.click(finishButton);

    expect(await screen.findByText("Login screen")).toBeInTheDocument();
  });

  it("submits newPassword instead of newPin in password mode", async () => {
    const useRecoveryCodeSpy = vi.spyOn(m8Auth, "useRecoveryCode").mockResolvedValue({
      recoveryCodes: ["FRESH-CODE-1"],
    });
    const user = userEvent.setup();
    renderRecovery();

    await enterCode(user);
    await user.click(screen.getByRole("radio", { name: "Password" }));
    await user.type(screen.getByLabelText(/New password/), "correct-horse-1");
    await user.type(screen.getByLabelText(/Confirm password/), "correct-horse-1");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(useRecoveryCodeSpy).toHaveBeenCalledWith({
      code: "ABCDE-FGHIJ-KLMNO",
      newPassword: "correct-horse-1",
    });
  });

  it("sends a refused code back to step 0, showing the server's message", async () => {
    vi.spyOn(m8Auth, "useRecoveryCode").mockRejectedValue({
      kind: "validation",
      field: "code",
      message: "This recovery code has already been used.",
    });
    const user = userEvent.setup();
    renderRecovery();

    await enterCode(user);
    await user.type(screen.getByLabelText(/New 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "123456");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("This recovery code has already been used.")).toBeInTheDocument();
    expect(screen.getByLabelText("Recovery code")).toBeInTheDocument();
    expect(screen.queryByText("Set a new credential")).not.toBeInTheDocument();
  });

  it("keeps a non-code validation error on step 1 rather than bouncing back", async () => {
    vi.spyOn(m8Auth, "useRecoveryCode").mockRejectedValue({
      kind: "validation",
      field: "newPin",
      message: "That PIN is not allowed.",
    });
    const user = userEvent.setup();
    renderRecovery();

    await enterCode(user);
    await user.type(screen.getByLabelText(/New 6-digit PIN/), "123456");
    await user.type(screen.getByLabelText(/Confirm PIN/), "123456");
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("That PIN is not allowed.")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Set a new credential" })).toBeInTheDocument();
  });
});
