import { describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ToastProvider, Toaster, useToast } from "./toast";
import { Button } from "./button";

function Fixture() {
  const toast = useToast();
  return (
    <>
      <Button
        onClick={() =>
          toast.add({ title: "Business Volume saved", type: "success", timeout: 50 })
        }
      >
        Save
      </Button>
      <Toaster />
    </>
  );
}

describe("Toaster", () => {
  it("shows a confirmation toast and auto-dismisses it, never requiring action", async () => {
    render(
      <ToastProvider>
        <Fixture />
      </ToastProvider>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Business Volume saved")).toBeInTheDocument();

    await waitFor(() => expect(screen.queryByText("Business Volume saved")).not.toBeInTheDocument(), {
      timeout: 2000,
    });
  });
});
