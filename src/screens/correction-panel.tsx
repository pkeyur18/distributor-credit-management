import { useState } from "react";
import { ArrowLeft } from "lucide-react";
import { Link } from "react-router";

import { Button } from "@/components/ui/button";
import { Input, InputHint } from "@/components/ui/input";
import { editEntry } from "@/lib/ipc/m2-entries";
import type { BusinessVolumeEntry as Entry } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useToast } from "@/components/ui/toast";
import { centsToDisplay, displayToCents } from "@/lib/utils";

// US-M2.2 (§5.5), Rule-39 — `edit_entry` is the sole correction mechanism,
// live period or closed month alike. Looked up **by Entry ID**, not by
// searching a member's history: no command in the closed 40-command
// surface lists a member's past entries yet (get_member_detail is S8,
// get_audit_log is S14), and there's no read command to preview an entry's
// current amount/date before editing either — this is a direct correction
// (new amount + new date go straight to `edit_entry`), not a browse-then-
// edit flow. An operator working from a paper record already knows what
// the corrected figure should be; ID lookup is what today's API supports.
export function CorrectionPanel() {
  const [entryId, setEntryId] = useState("");
  const [amountInput, setAmountInput] = useState("");
  const [date, setDate] = useState("");
  const [amountError, setAmountError] = useState<string | null>(null);
  const [dateError, setDateError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [lastResult, setLastResult] = useState<Entry | null>(null);
  const toast = useToast();

  const id = Number.parseInt(entryId, 10);
  const cents = displayToCents(amountInput);
  const canSave = Number.isInteger(id) && id > 0 && cents !== null && !!date && !saving;

  async function handleSave() {
    if (!canSave || cents === null) return;
    setSaving(true);
    setAmountError(null);
    setDateError(null);
    try {
      const updated = await editEntry({ id, amount: cents, entryDate: date });
      setLastResult(updated);
      toast.add({
        title: `Entry #${updated.id} corrected to ${centsToDisplay(updated.amount)}`,
        type: "success",
      });
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      if (presented.field === "entryDate") {
        setDateError(presented.message);
      } else {
        setAmountError(presented.message);
      }
    } finally {
      setSaving(false);
    }
  }

  return (
    <>
      <h1 className="text-headline">Correct a closed month</h1>

      <div className="mt-3.5 max-w-md rounded-sm border border-border bg-bg px-3.5 py-3 text-caption">
        Editing a record recalculates the affected chain and writes a new snapshot version — the
        original record is never overwritten.
      </div>

      <div className="mt-3.5 max-w-md rounded-lg border border-border bg-surface p-4.5">
        <label htmlFor="correction-entry-id" className="text-label mb-1 block">
          Entry ID *
        </label>
        <Input
          id="correction-entry-id"
          type="number"
          min={1}
          value={entryId}
          onChange={(e) => {
            setEntryId(e.target.value);
            setAmountError(null);
            setDateError(null);
          }}
        />

        <div className="mt-3.5">
          <label htmlFor="correction-date" className="text-label mb-1 block">
            Date *
          </label>
          <Input
            id="correction-date"
            type="date"
            value={date}
            aria-invalid={!!dateError}
            onChange={(e) => {
              setDate(e.target.value);
              setDateError(null);
            }}
          />
          <InputHint error={!!dateError}>
            {dateError ?? "Must stay within the entry's own month (RQ-21)"}
          </InputHint>
        </div>

        <div className="mt-3.5">
          <label htmlFor="correction-amount" className="text-label mb-1 block">
            Business Volume *
          </label>
          <Input
            id="correction-amount"
            className="num"
            type="text"
            inputMode="decimal"
            placeholder="0.00"
            value={amountInput}
            aria-invalid={!!amountError}
            onChange={(e) => {
              setAmountInput(e.target.value);
              setAmountError(null);
            }}
          />
          <InputHint error={!!amountError}>{amountError}</InputHint>
        </div>

        <Button variant="primary" className="mt-3.5 w-full" disabled={!canSave} onClick={handleSave}>
          Save correction
        </Button>
      </div>

      {lastResult && (
        <div className="mt-3.5 max-w-md text-caption">
          Entry #{lastResult.id} now reads {centsToDisplay(lastResult.amount)} on{" "}
          {lastResult.entryDate}.
        </div>
      )}

      <p className="mt-3.5 max-w-md text-caption">
        <Link to="/entry" className="inline-flex items-center gap-1 text-accent">
          <ArrowLeft className="size-3.5" /> Back to volume entry
        </Link>
      </p>
    </>
  );
}
