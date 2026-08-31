import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { AuthProvider, useAuth } from "./auth-context";
import * as preflight from "@/lib/ipc/preflight";

// A thin consumer that exposes every AuthContext value as clickable
// controls/text, so each transition can be driven and asserted without
// reaching into React internals.
function Consumer() {
  const { state, markAuthenticated, markLocked, markSignedOut, signOutNotice, clearSignOutNotice } =
    useAuth();
  return (
    <div>
      <div data-testid="state">{state}</div>
      <div data-testid="notice">{signOutNotice ?? "none"}</div>
      <button onClick={markAuthenticated}>authenticate</button>
      <button onClick={markLocked}>lock</button>
      <button onClick={() => markSignedOut("Restore complete.")}>sign-out-with-notice</button>
      <button onClick={() => markSignedOut()}>sign-out-no-notice</button>
      <button onClick={clearSignOutNotice}>clear-notice</button>
    </div>
  );
}

function renderWithProvider() {
  return render(
    <AuthProvider>
      <Consumer />
    </AuthProvider>,
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("AuthProvider — initial resolution", () => {
  it("starts in 'loading' before check_data_readable resolves", () => {
    vi.spyOn(preflight, "checkDataReadable").mockReturnValue(new Promise(() => {}));
    renderWithProvider();
    expect(screen.getByTestId("state")).toHaveTextContent("loading");
  });

  it("resolves to 'needs-login' when data is readable", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(true);
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-login"));
  });

  it("resolves to 'needs-setup' when no data exists yet", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(false);
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-setup"));
  });

  it("resolves to 'needs-recovery' when check_data_readable rejects", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockRejectedValue(new Error("data unreadable"));
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-recovery"));
  });
});

describe("AuthProvider — transitions", () => {
  it("markAuthenticated moves to 'authenticated'", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(true);
    const user = userEvent.setup();
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-login"));

    await user.click(screen.getByText("authenticate"));
    expect(screen.getByTestId("state")).toHaveTextContent("authenticated");
  });

  it("markLocked moves to 'locked'", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(true);
    const user = userEvent.setup();
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-login"));

    await user.click(screen.getByText("authenticate"));
    await user.click(screen.getByText("lock"));
    expect(screen.getByTestId("state")).toHaveTextContent("locked");
  });

  it("markSignedOut(notice) routes to 'needs-login' and carries the notice", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(true);
    const user = userEvent.setup();
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-login"));

    await user.click(screen.getByText("authenticate"));
    await user.click(screen.getByText("lock"));
    await user.click(screen.getByText("sign-out-with-notice"));

    expect(screen.getByTestId("state")).toHaveTextContent("needs-login");
    expect(screen.getByTestId("notice")).toHaveTextContent("Restore complete.");
  });

  it("markSignedOut() with no notice clears any prior notice", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(true);
    const user = userEvent.setup();
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-login"));

    await user.click(screen.getByText("sign-out-with-notice"));
    expect(screen.getByTestId("notice")).toHaveTextContent("Restore complete.");

    await user.click(screen.getByText("sign-out-no-notice"));
    expect(screen.getByTestId("notice")).toHaveTextContent("none");
  });

  it("clearSignOutNotice resets the notice without changing state", async () => {
    vi.spyOn(preflight, "checkDataReadable").mockResolvedValue(true);
    const user = userEvent.setup();
    renderWithProvider();
    await waitFor(() => expect(screen.getByTestId("state")).toHaveTextContent("needs-login"));

    await user.click(screen.getByText("sign-out-with-notice"));
    expect(screen.getByTestId("notice")).toHaveTextContent("Restore complete.");

    await user.click(screen.getByText("clear-notice"));
    expect(screen.getByTestId("notice")).toHaveTextContent("none");
    expect(screen.getByTestId("state")).toHaveTextContent("needs-login");
  });
});

describe("useAuth", () => {
  it("throws when used outside an AuthProvider", () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(() => render(<Consumer />)).toThrow("useAuth must be used inside AuthProvider");
    consoleError.mockRestore();
  });
});
