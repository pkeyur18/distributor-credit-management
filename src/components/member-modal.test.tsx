import type { ReactElement } from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, type RenderResult } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { MemberModal } from "./member-modal";
import { ToastProvider } from "@/components/ui/toast";
import type { AddMemberOutcome } from "@/lib/ipc/m1-members";
import type { Member, SearchResult } from "@/lib/ipc/entities";

function renderModal(ui: ReactElement): RenderResult {
  return render(<ToastProvider>{ui}</ToastProvider>);
}

const MEMBER: Member = {
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

const REF_RESULT: SearchResult = {
  id: 100001,
  name: "Top Member",
  phone: "9876500000",
  totalBusinessVolume: 0,
  slabPct: 0,
  isActive: true,
  email: null,
  address: "1 Main Street",
  introducerMemberId: null,
};

async function fillRequiredFields(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByLabelText(/^Name/), "Asha Patel");
  await user.type(screen.getByLabelText(/^Phone/), "9876543210");
  await user.type(screen.getByLabelText(/^Address/), "2 Side Street");
}

async function pickIntroducer(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByLabelText(/Introducer/), "Top");
  const option = await screen.findByText("Top Member");
  await user.click(option);
}

describe("MemberModal — add mode", () => {
  it("disables Save until consent is ticked", () => {
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="add"
        onSubmitAdd={vi.fn()}
        onSearchRef={async () => [REF_RESULT]}
      />,
    );
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("calls onSubmitAdd with the typed fields and the selected introducer", async () => {
    const user = userEvent.setup();
    const onSubmitAdd = vi.fn<(input: unknown) => Promise<AddMemberOutcome>>().mockResolvedValue({
      status: "created",
      member: MEMBER,
      warnings: [],
    });
    const onSaved = vi.fn();
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="add"
        onSubmitAdd={onSubmitAdd}
        onSearchRef={async () => [REF_RESULT]}
        onSaved={onSaved}
      />,
    );

    await fillRequiredFields(user);
    await pickIntroducer(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(onSaved).toHaveBeenCalledWith(MEMBER));
    expect(onSubmitAdd).toHaveBeenCalledWith(
      expect.objectContaining({
        name: "Asha Patel",
        phone: "9876543210",
        introducerMemberId: 100001,
        consentGiven: true,
      }),
    );
  });

  it("pauses on an advisory warning, then completes on Done", async () => {
    const user = userEvent.setup();
    const onSubmitAdd = vi.fn<(input: unknown) => Promise<AddMemberOutcome>>().mockResolvedValue({
      status: "created",
      member: MEMBER,
      warnings: ["Level 2 now has 10 members, above the configured width of 9."],
    });
    const onSaved = vi.fn();
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="add"
        onSubmitAdd={onSubmitAdd}
        onSearchRef={async () => [REF_RESULT]}
        onSaved={onSaved}
      />,
    );

    await fillRequiredFields(user);
    await pickIntroducer(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText(/above the configured width/)).toBeInTheDocument();
    expect(onSaved).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Done" }));
    expect(onSaved).toHaveBeenCalledWith(MEMBER);
  });

  it("offers reactivation on a phone conflict, and reactivating calls edit then reactivate", async () => {
    const user = userEvent.setup();
    const onSubmitAdd = vi.fn<(input: unknown) => Promise<AddMemberOutcome>>().mockResolvedValue({
      status: "reactivation_offer",
      existingMember: MEMBER,
    });
    const onSubmitEdit = vi.fn().mockResolvedValue({ ...MEMBER, isActive: true });
    const onSubmitReactivate = vi.fn().mockResolvedValue(undefined);
    const onSaved = vi.fn();
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="add"
        onSubmitAdd={onSubmitAdd}
        onSubmitEdit={onSubmitEdit}
        onSubmitReactivate={onSubmitReactivate}
        onSearchRef={async () => [REF_RESULT]}
        onSaved={onSaved}
      />,
    );

    await fillRequiredFields(user);
    await pickIntroducer(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText(/matches an inactive member/)).toBeInTheDocument();
    await user.click(screen.getByText("Reactivate them instead"));

    expect(screen.getByRole("heading", { name: "Reactivate member" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Reactivate" }));

    await waitFor(() => expect(onSubmitReactivate).toHaveBeenCalledWith(MEMBER.id));
    expect(onSubmitEdit).toHaveBeenCalledWith(expect.objectContaining({ id: MEMBER.id }));
    expect(onSaved).toHaveBeenCalled();
  });

  it("keeps Save disabled without an introducer selected", async () => {
    const user = userEvent.setup();
    const onSubmitAdd = vi.fn();
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="add"
        onSubmitAdd={onSubmitAdd}
        onSearchRef={async () => [REF_RESULT]}
      />,
    );

    await fillRequiredFields(user);
    await user.click(screen.getByLabelText(/consented/));

    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
    expect(onSubmitAdd).not.toHaveBeenCalled();
  });

  it("skips the introducer field and calls onSubmitAddRoot when the directory is empty", async () => {
    const user = userEvent.setup();
    const onSubmitAdd = vi.fn();
    const onSubmitAddRoot = vi.fn().mockResolvedValue({ ...MEMBER, introducerMemberId: null });
    const onSaved = vi.fn();
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="add"
        noMembersYet
        onSubmitAdd={onSubmitAdd}
        onSubmitAddRoot={onSubmitAddRoot}
        onSearchRef={async () => [REF_RESULT]}
        onSaved={onSaved}
      />,
    );

    expect(screen.queryByLabelText(/Introducer \*/)).not.toBeInTheDocument();

    await fillRequiredFields(user);
    await user.click(screen.getByLabelText(/consented/));
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(onSubmitAddRoot).toHaveBeenCalledWith(
        expect.objectContaining({ name: "Asha Patel", consentGiven: true }),
      ),
    );
    expect(onSubmitAdd).not.toHaveBeenCalled();
    expect(onSaved).toHaveBeenCalled();
  });
});

describe("MemberModal — edit mode", () => {
  it("prefills from the member and shows the introducer read-only", () => {
    renderModal(
      <MemberModal open onOpenChange={() => {}} mode="edit" member={MEMBER} onSubmitEdit={vi.fn()} />,
    );
    expect(screen.getByLabelText(/^Name/)).toHaveValue("Asha Patel");
    expect(screen.getByLabelText(/^Phone/)).toHaveValue("9876543210");
    expect(screen.getByDisplayValue("#100001")).toBeInTheDocument();
    expect(screen.queryByLabelText(/Introducer \*/)).not.toBeInTheDocument();
  });

  it("saves edited fields without a consent checkbox", async () => {
    const user = userEvent.setup();
    const onSubmitEdit = vi.fn().mockResolvedValue({ ...MEMBER, name: "Asha P. Renamed" });
    const onSaved = vi.fn();
    renderModal(
      <MemberModal
        open
        onOpenChange={() => {}}
        mode="edit"
        member={MEMBER}
        onSubmitEdit={onSubmitEdit}
        onSaved={onSaved}
      />,
    );

    const nameField = screen.getByLabelText(/^Name/);
    await user.clear(nameField);
    await user.type(nameField, "Asha P. Renamed");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(onSubmitEdit).toHaveBeenCalledWith(
        expect.objectContaining({ id: MEMBER.id, name: "Asha P. Renamed" }),
      ),
    );
    expect(onSaved).toHaveBeenCalled();
  });

  it("shows a refused-conflict error inline", async () => {
    const user = userEvent.setup();
    const onSubmitEdit = vi.fn().mockRejectedValue({
      kind: "conflict",
      message: "This phone number is already in use by Rahul Shah (#512004).",
    });
    renderModal(
      <MemberModal open onOpenChange={() => {}} mode="edit" member={MEMBER} onSubmitEdit={onSubmitEdit} />,
    );

    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText(/Rahul Shah/)).toBeInTheDocument();
  });
});
