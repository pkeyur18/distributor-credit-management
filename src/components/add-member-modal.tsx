import { useState } from "react";

import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "@/components/ui/dialog";
import { Input, InputHint } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AlertNote } from "@/components/ui/alert-note";
import { addMember, type AddMemberInput, type AddMemberOutcome } from "@/lib/ipc/m1-members";
import type { Member } from "@/lib/ipc/entities";
import { toErrorPresentation, type AppErrorPresentation } from "@/lib/ipc/errors";

// US-M1.1 (T-UI.3-5/T-M1.1-7): the non-root Add Member modal. Non-dismissable
// (Cancel/✕ only), Cancel-first, Save disabled until consent is ticked
// (Rule-40) — matching ui-prototype-v2.html's #member-modal.
//
// Two prototype affordances are deliberately not ported this sprint:
// - The reference field is a typed 6-digit ID, not a live-search dropdown —
//   `search_members` (Rule-44) is US-M1.4, Sprint 5. Resolution still
//   happens (server-side, on Save), just without the as-you-type UI.
// - The phone-conflict banner only appears after Save, not on blur — there
//   is no cheap live lookup yet either. An inactive-phone match still shows
//   the reactivation-offer note; the "Reactivate them instead" action isn't
//   wired until edit_member/reactivate_member ship (US-M1.2/M1.3, S5).

interface AddMemberModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit?: (input: AddMemberInput) => Promise<AddMemberOutcome>;
  onCreated?: (member: Member) => void;
}

const EMPTY_FORM = {
  name: "",
  phone: "",
  email: "",
  address: "",
  introducerIdText: "",
  consentGiven: false,
};

function AddMemberModal({ open, onOpenChange, onSubmit = addMember, onCreated }: AddMemberModalProps) {
  const [form, setForm] = useState(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<AppErrorPresentation | null>(null);
  const [reactivationOffer, setReactivationOffer] = useState<Member | null>(null);
  // Rule-1/Rule-32: advisory-only warnings pause the close (once) so the
  // operator actually sees them — the save already happened either way.
  const [savedWithWarnings, setSavedWithWarnings] = useState<{ member: Member; warnings: string[] } | null>(
    null,
  );

  function reset() {
    setForm(EMPTY_FORM);
    setError(null);
    setReactivationOffer(null);
    setSavedWithWarnings(null);
  }

  function handleOpenChange(next: boolean) {
    if (!next) reset();
    onOpenChange(next);
  }

  function handleDone() {
    if (savedWithWarnings) onCreated?.(savedWithWarnings.member);
    handleOpenChange(false);
  }

  async function handleSave() {
    setError(null);
    setReactivationOffer(null);

    const introducerMemberId = Number(form.introducerIdText.trim());
    if (!form.introducerIdText.trim() || !Number.isInteger(introducerMemberId)) {
      setError({ kind: "validation", message: "Enter the introducer's 6-digit member number.", field: "introducerMemberId" });
      return;
    }

    setSubmitting(true);
    try {
      const outcome = await onSubmit({
        name: form.name,
        phone: form.phone,
        email: form.email.trim() || undefined,
        address: form.address,
        consentGiven: form.consentGiven,
        introducerMemberId,
      });
      if (outcome.status === "reactivation_offer") {
        setReactivationOffer(outcome.existingMember);
      } else if (outcome.warnings.length > 0) {
        setSavedWithWarnings({ member: outcome.member, warnings: outcome.warnings });
      } else {
        onCreated?.(outcome.member);
        handleOpenChange(false);
      }
    } catch (raw) {
      setError(toErrorPresentation(raw));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onOpenChange={handleOpenChange} dismissable={false}>
      <ModalHeader title="Add member" />
      <ModalBody>
        <div className="flex flex-col gap-3">
          {error && <AlertNote variant="danger">{error.message}</AlertNote>}
          {reactivationOffer && (
            <AlertNote variant="warn">
              This number matches an inactive member — <strong>{reactivationOffer.name}</strong> (#
              {reactivationOffer.id}). Reactivation isn't available in this build yet.
            </AlertNote>
          )}
          {savedWithWarnings && (
            <AlertNote variant="warn">
              <strong>{savedWithWarnings.member.name}</strong> was added (#{savedWithWarnings.member.id}).
              {savedWithWarnings.warnings.map((w) => (
                <div key={w}>{w}</div>
              ))}
            </AlertNote>
          )}
          <div>
            <label htmlFor="member-name" className="text-label mb-1 block">
              Name *
            </label>
            <Input
              id="member-name"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </div>
          <div>
            <label htmlFor="member-phone" className="text-label mb-1 block">
              Phone *
            </label>
            <Input
              id="member-phone"
              type="tel"
              value={form.phone}
              onChange={(e) => setForm({ ...form, phone: e.target.value })}
            />
          </div>
          <div>
            <label htmlFor="member-email" className="text-label mb-1 block">
              Email
            </label>
            <Input
              id="member-email"
              type="email"
              value={form.email}
              onChange={(e) => setForm({ ...form, email: e.target.value })}
            />
          </div>
          <div>
            <label htmlFor="member-address" className="text-label mb-1 block">
              Address *
            </label>
            <Input
              id="member-address"
              value={form.address}
              onChange={(e) => setForm({ ...form, address: e.target.value })}
            />
          </div>
          <div>
            <label htmlFor="member-introducer" className="text-label mb-1 block">
              Introducer (Reference ID) *
            </label>
            <Input
              id="member-introducer"
              inputMode="numeric"
              placeholder="6-digit member number"
              value={form.introducerIdText}
              onChange={(e) => setForm({ ...form, introducerIdText: e.target.value })}
            />
            <InputHint>Must resolve to an existing, active member. Locked once saved.</InputHint>
          </div>
          <label htmlFor="member-consent" className="flex items-start gap-2 text-body">
            <input
              id="member-consent"
              type="checkbox"
              checked={form.consentGiven}
              onChange={(e) => setForm({ ...form, consentGiven: e.target.checked })}
              className="mt-0.5"
            />
            <span>The member has consented to their name, contact number and address being recorded in this system.</span>
          </label>
        </div>
      </ModalBody>
      <ModalFooter>
        {savedWithWarnings ? (
          <Button variant="primary" onClick={handleDone}>
            Done
          </Button>
        ) : (
          <>
            <ModalCancel />
            <Button variant="primary" disabled={!form.consentGiven || submitting} onClick={handleSave}>
              Save
            </Button>
          </>
        )}
      </ModalFooter>
    </Modal>
  );
}

export { AddMemberModal };
