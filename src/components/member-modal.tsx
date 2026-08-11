import { useEffect, useState } from "react";

import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "@/components/ui/dialog";
import { Input, InputHint } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AlertNote } from "@/components/ui/alert-note";
import { Pill } from "@/components/ui/pill";
import { SearchResultsList } from "@/components/search-results-list";
import {
  addMember,
  editMember,
  reactivateMember,
  searchMembers,
  type AddMemberInput,
  type AddMemberOutcome,
  type EditMemberInput,
} from "@/lib/ipc/m1-members";
import type { Member, SearchResult } from "@/lib/ipc/entities";
import { toErrorPresentation, type AppErrorPresentation } from "@/lib/ipc/errors";

// US-M1.1/M1.2/M1.3 — one shared modal for add/edit, matching
// ui-prototype-v2.html's `memberModalHtml()`: the same form for all three
// cases (add/edit/reactivate), never three separate screens. "Reactivate"
// is reached internally from "add" when a phone collides with an inactive
// member (Rule-34) — never a mode a caller opens directly, exactly like the
// prototype's `switchToReactivate`.
//
// Non-dismissable (Cancel/✕ only), Cancel-first, Save disabled until
// consent is ticked in add mode (Rule-40). The introducer is a live search
// (Rule-30, active-only) in add mode; in edit mode it's shown read-only —
// there is no field that could send it (Rule-37).

// Deliberately narrower than `Member`: edit-mode prefill only ever reads
// these five fields, and a `SearchResult` (which carries them too, see its
// doc comment) satisfies this directly — no `get_member_detail` (S8) round
// trip needed to open Edit from a search result this sprint.
type EditableMember = Pick<Member, "id" | "name" | "phone" | "email" | "address" | "introducerMemberId">;

interface MemberModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  mode: "add" | "edit";
  /** Required when `mode` is "edit". */
  member?: EditableMember;
  onSubmitAdd?: (input: AddMemberInput) => Promise<AddMemberOutcome>;
  onSubmitEdit?: (input: EditMemberInput) => Promise<Member>;
  onSubmitReactivate?: (id: number) => Promise<void>;
  onSearchRef?: (query: string) => Promise<SearchResult[]>;
  onSaved?: (member: Member) => void;
}

const EMPTY_FORM = {
  name: "",
  phone: "",
  email: "",
  address: "",
  consentGiven: false,
};

function MemberModal({
  open,
  onOpenChange,
  mode,
  member,
  onSubmitAdd = addMember,
  onSubmitEdit = editMember,
  onSubmitReactivate = reactivateMember,
  onSearchRef = (q) => searchMembers(q, true),
  onSaved,
}: MemberModalProps) {
  const [form, setForm] = useState(EMPTY_FORM);
  const [selectedRef, setSelectedRef] = useState<SearchResult | null>(null);
  const [refQuery, setRefQuery] = useState("");
  const [refResults, setRefResults] = useState<SearchResult[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<AppErrorPresentation | null>(null);
  const [reactivationOffer, setReactivationOffer] = useState<Member | null>(null);
  // Set once the operator follows "Reactivate them instead" — the id being
  // reactivated, with whatever the operator has typed carried forward.
  const [reactivatingId, setReactivatingId] = useState<number | null>(null);
  // Rule-1/Rule-32: advisory-only warnings pause the close (once) so the
  // operator actually sees them — the save already happened either way.
  const [savedWithWarnings, setSavedWithWarnings] = useState<{ member: Member; warnings: string[] } | null>(
    null,
  );

  // Reset-on-open, computed during render rather than in an effect (react.dev
  // "Adjusting state when a prop changes") — `resetKey` changes exactly when
  // a fresh open (of a possibly different member) should clear every piece
  // of transient state below, and nothing else ever should.
  const resetKey = open ? `${mode}:${member?.id ?? "new"}` : null;
  const [lastResetKey, setLastResetKey] = useState<string | null>(null);
  if (resetKey !== null && resetKey !== lastResetKey) {
    setLastResetKey(resetKey);
    if (mode === "edit" && member) {
      setForm({
        name: member.name,
        phone: member.phone,
        email: member.email ?? "",
        address: member.address,
        consentGiven: true,
      });
    } else {
      setForm(EMPTY_FORM);
    }
    setSelectedRef(null);
    setRefQuery("");
    setRefResults([]);
    setError(null);
    setReactivationOffer(null);
    setReactivatingId(null);
    setSavedWithWarnings(null);
  }

  const trimmedRefQuery = refQuery.trim();

  useEffect(() => {
    if (!trimmedRefQuery) return;
    let cancelled = false;
    const timer = setTimeout(() => {
      onSearchRef(trimmedRefQuery).then((found) => {
        if (!cancelled) setRefResults(found);
      });
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [trimmedRefQuery]);

  const displayedRefResults = trimmedRefQuery ? refResults : [];

  function handleOpenChange(next: boolean) {
    onOpenChange(next);
  }

  function handleDone() {
    if (savedWithWarnings) onSaved?.(savedWithWarnings.member);
    handleOpenChange(false);
  }

  async function handleSave() {
    setError(null);
    setReactivationOffer(null);
    setSubmitting(true);
    try {
      if (reactivatingId !== null) {
        const updated = await onSubmitEdit({
          id: reactivatingId,
          name: form.name,
          phone: form.phone,
          email: form.email.trim() || null,
          address: form.address,
        });
        await onSubmitReactivate(reactivatingId);
        onSaved?.(updated);
        handleOpenChange(false);
        return;
      }

      if (mode === "edit" && member) {
        const updated = await onSubmitEdit({
          id: member.id,
          name: form.name,
          phone: form.phone,
          email: form.email.trim() || null,
          address: form.address,
        });
        onSaved?.(updated);
        handleOpenChange(false);
        return;
      }

      if (!selectedRef) {
        setError({
          kind: "validation",
          message: "Choose an introducer.",
          field: "introducerMemberId",
        });
        return;
      }
      const outcome = await onSubmitAdd({
        name: form.name,
        phone: form.phone,
        email: form.email.trim() || undefined,
        address: form.address,
        consentGiven: form.consentGiven,
        introducerMemberId: selectedRef.id,
      });
      if (outcome.status === "reactivation_offer") {
        setReactivationOffer(outcome.existingMember);
      } else if (outcome.warnings.length > 0) {
        setSavedWithWarnings({ member: outcome.member, warnings: outcome.warnings });
      } else {
        onSaved?.(outcome.member);
        handleOpenChange(false);
      }
    } catch (raw) {
      setError(toErrorPresentation(raw));
    } finally {
      setSubmitting(false);
    }
  }

  const isReactivating = reactivatingId !== null;
  const title = isReactivating ? "Reactivate member" : mode === "edit" ? "Edit member" : "Add member";
  const saveLabel = isReactivating ? "Reactivate" : "Save";
  const showConsentCheckbox = mode === "add" && !isReactivating;

  return (
    <Modal open={open} onOpenChange={handleOpenChange} dismissable={false}>
      <ModalHeader title={title} />
      <ModalBody>
        <div className="flex flex-col gap-3">
          {error && <AlertNote variant="danger">{error.message}</AlertNote>}
          {isReactivating && (
            <AlertNote variant="warn">
              Saving will reactivate this member and preserve their original member number and
              history.
            </AlertNote>
          )}
          {reactivationOffer && (
            <AlertNote variant="warn">
              This number matches an inactive member — <strong>{reactivationOffer.name}</strong> (#
              {reactivationOffer.id}).{" "}
              <button
                type="button"
                className="font-[650] underline"
                onClick={() => {
                  setReactivatingId(reactivationOffer.id);
                  setReactivationOffer(null);
                }}
              >
                Reactivate them instead
              </button>
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

          {mode === "edit" && !isReactivating ? (
            <div>
              <span className="text-label mb-1 block">Introducer</span>
              <p className="text-body text-muted-text">
                {member?.introducerMemberId ? `#${member.introducerMemberId}` : "None (root member)"}
              </p>
              <InputHint>Fixed at creation — never changes (Rule-37).</InputHint>
            </div>
          ) : !isReactivating ? (
            <div>
              <label htmlFor="member-ref-search" className="text-label mb-1 block">
                Introducer *
              </label>
              {selectedRef ? (
                <div className="flex items-center justify-between rounded-sm border border-border bg-surface px-3 py-2">
                  <div>
                    <div className="text-title-sm">{selectedRef.name}</div>
                    <div className="mono text-[11px] text-muted-text">#{selectedRef.id}</div>
                  </div>
                  <Button variant="ghost" size="sm" onClick={() => setSelectedRef(null)}>
                    Change
                  </Button>
                </div>
              ) : (
                <>
                  <Input
                    id="member-ref-search"
                    placeholder="Search an active member by name, number or phone"
                    value={refQuery}
                    onChange={(e) => setRefQuery(e.target.value)}
                    autoComplete="off"
                  />
                  <div className="mt-1.5">
                    <SearchResultsList
                      results={displayedRefResults}
                      query={refQuery}
                      emptyLabel="No active member matches"
                      onSelect={(r) => {
                        setSelectedRef(r);
                        setRefQuery("");
                        setRefResults([]);
                      }}
                    />
                  </div>
                </>
              )}
            </div>
          ) : null}

          {showConsentCheckbox ? (
            <label htmlFor="member-consent" className="flex items-start gap-2 text-body">
              <input
                id="member-consent"
                type="checkbox"
                checked={form.consentGiven}
                onChange={(e) => setForm({ ...form, consentGiven: e.target.checked })}
                className="mt-0.5"
              />
              <span>
                The member has consented to their name, contact number and address being recorded in
                this system.
              </span>
            </label>
          ) : (
            <div>
              <span className="text-label mb-1 block">Consent</span>
              <Pill variant="active">Captured</Pill>
            </div>
          )}
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
            <Button
              variant="primary"
              disabled={(showConsentCheckbox && !form.consentGiven) || submitting}
              onClick={handleSave}
            >
              {saveLabel}
            </Button>
          </>
        )}
      </ModalFooter>
    </Modal>
  );
}

export { MemberModal };
