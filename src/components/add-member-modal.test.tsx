import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { AddMemberModal } from "./add-member-modal";
import type { AddMemberOutcome } from "@/lib/ipc/m1-members";

const MEMBER = {
  id: 284913,
  name: "Asha Patel",
  phone: "9876543210",
  email: null,
  address: "1 Main Street",
  introducerMemberId: 100001,
  level: 2,
  isActive: true,
  joiningDate: "2026-01-01",
  consentGiven: true,
  consentDate: "2026-01-01",
  createdAt: "2026-01-01",
};

async function fillRequiredFields(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByLabelText(/^Name/), "Asha Patel");
  await user.type(screen.getByLabelText(/^Phone/), "9876543210");
  await user.type(screen.getByLabelText(/^Address/), "2 Side Street");
  await user.type(screen.getByLabelText(/Introducer/), "100001");
}

describe("AddMemberModal", () => {
  it("disables Save until consent is ticked", () => {
    render(<AddMemberModal open onOpenChange={() => {}} onSubmit={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("calls onSubmit with the typed fields and reports created on success", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn<(input: unknown) => Promise<AddMemberOutcome>>().mockResolvedValue({
      status: "created",
      member: MEMBER,
    });
    const onCreated = vi.fn();
    render(<AddMemberModal open onOpenChange={() => {}} onSubmit={onSubmit} onCreated={onCreated} />);

    await fillRequiredFields(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(onCreated).toHaveBeenCalledWith(MEMBER));
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({
        name: "Asha Patel",
        phone: "9876543210",
        introducerMemberId: 100001,
        consentGiven: true,
      }),
    );
  });

  it("shows the reactivation-offer note without closing, and does not call onCreated", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn<(input: unknown) => Promise<AddMemberOutcome>>().mockResolvedValue({
      status: "reactivation_offer",
      existingMember: MEMBER,
    });
    const onCreated = vi.fn();
    render(<AddMemberModal open onOpenChange={() => {}} onSubmit={onSubmit} onCreated={onCreated} />);

    await fillRequiredFields(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText(/matches an inactive member/)).toBeInTheDocument();
    expect(screen.getByText(/Asha Patel/)).toBeInTheDocument();
    expect(onCreated).not.toHaveBeenCalled();
  });

  it("shows a refused-conflict error inline", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn<(input: unknown) => Promise<AddMemberOutcome>>().mockRejectedValue({
      kind: "conflict",
      message: "This phone number is already in use by Rahul Shah (#512004).",
    });
    render(<AddMemberModal open onOpenChange={() => {}} onSubmit={onSubmit} />);

    await fillRequiredFields(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText(/Rahul Shah/)).toBeInTheDocument();
  });

  it("refuses a malformed introducer ID without calling onSubmit", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    render(<AddMemberModal open onOpenChange={() => {}} onSubmit={onSubmit} />);

    await user.type(screen.getByLabelText(/^Name/), "Asha Patel");
    await user.type(screen.getByLabelText(/^Phone/), "9876543210");
    await user.type(screen.getByLabelText(/^Address/), "2 Side Street");
    await user.type(screen.getByLabelText(/Introducer/), "not-a-number");
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText(/6-digit member number/)).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });
});
