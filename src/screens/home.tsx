import { useState } from "react";
import { PlusCircle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/ui/pill";
import { MemberModal } from "@/components/member-modal";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { SearchResultsList } from "@/components/search-results-list";
import { useMemberSearch } from "@/lib/use-member-search";
import { deactivateMember, reactivateMember } from "@/lib/ipc/m1-members";
import { useToast } from "@/components/ui/toast";
import type { SearchResult } from "@/lib/ipc/entities";

// US-M1.1 (T-M1.1-7) + US-M1.2/M1.3/M1.4 (S5). Stat cards and the
// slab-distribution charts (US-M4.4) land with the rest of Home in S8 —
// this screen carries only what's shipped so far: adding a member,
// searching the directory, and editing/deactivating/reactivating whichever
// member a search turns up. A dedicated Member Detail view (US-M4.1) is
// where these actions properly belong; until it exists in S8, they're
// reachable from here.
export function Home() {
  const [addMemberOpen, setAddMemberOpen] = useState(false);
  const [query, setQuery] = useState("");
  const { results } = useMemberSearch(query);
  const [selected, setSelected] = useState<SearchResult | null>(null);
  const [editOpen, setEditOpen] = useState(false);
  const [confirmAction, setConfirmAction] = useState<"deactivate" | "reactivate" | null>(null);
  const [busy, setBusy] = useState(false);
  const toast = useToast();

  const isRoot = selected?.introducerMemberId == null;

  async function handleConfirm() {
    if (!selected || !confirmAction) return;
    setBusy(true);
    try {
      if (confirmAction === "deactivate") {
        await deactivateMember(selected.id);
        toast.add({ title: "Member deactivated", type: "success" });
      } else {
        await reactivateMember(selected.id);
        toast.add({ title: "Member reactivated", type: "success" });
      }
      setSelected({ ...selected, isActive: confirmAction === "reactivate" });
      setConfirmAction(null);
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <div className="flex items-center justify-between">
        <h1 className="text-headline">Home</h1>
        <Button variant="primary" onClick={() => setAddMemberOpen(true)}>
          <PlusCircle className="size-4" />
          Add member
        </Button>
      </div>

      <div className="mt-5 max-w-md">
        <Input
          placeholder="Search by name, 6-digit member number or phone"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setSelected(null);
          }}
        />
        <div className="mt-1.5">
          <SearchResultsList results={results} query={query} onSelect={setSelected} />
        </div>
      </div>

      {selected && (
        <div className="mt-4 max-w-md rounded-lg border border-border bg-surface p-4.5">
          <div className="flex items-center gap-1.5 text-title-sm">
            {selected.name}
            {!selected.isActive && <Pill variant="inactive">Inactive</Pill>}
          </div>
          <div className="mono text-[11px] text-muted-text">
            #{selected.id} · {selected.phone}
          </div>
          <div className="mt-3 flex gap-2">
            <Button variant="secondary" size="sm" onClick={() => setEditOpen(true)}>
              Edit
            </Button>
            {selected.isActive ? (
              <Button
                variant="danger"
                size="sm"
                disabled={isRoot}
                title={isRoot ? "The root member cannot be deactivated." : undefined}
                onClick={() => setConfirmAction("deactivate")}
              >
                Deactivate
              </Button>
            ) : (
              <Button variant="secondary" size="sm" onClick={() => setConfirmAction("reactivate")}>
                Reactivate
              </Button>
            )}
          </div>
        </div>
      )}

      <MemberModal open={addMemberOpen} onOpenChange={setAddMemberOpen} mode="add" />

      {selected && (
        <MemberModal
          open={editOpen}
          onOpenChange={setEditOpen}
          mode="edit"
          member={selected}
          onSaved={(m) =>
            setSelected({ ...selected, name: m.name, phone: m.phone, email: m.email, address: m.address })
          }
        />
      )}

      {selected && confirmAction && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && setConfirmAction(null)}
          title={
            confirmAction === "deactivate"
              ? `Deactivate ${selected.name}?`
              : `Reactivate ${selected.name}?`
          }
          body={
            confirmAction === "deactivate"
              ? "This marks the member inactive everywhere they appear. It has no effect on any calculation — their Business Volume continues to roll up exactly as if they were active."
              : "This restores the member under their original number, hierarchy position and history."
          }
          confirmLabel={confirmAction === "deactivate" ? "Deactivate" : "Reactivate"}
          danger={confirmAction === "deactivate"}
          busy={busy}
          onConfirm={handleConfirm}
        />
      )}
    </>
  );
}
