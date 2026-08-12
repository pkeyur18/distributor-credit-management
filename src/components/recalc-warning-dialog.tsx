import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "@/components/ui/dialog";
import { AlertNote } from "@/components/ui/alert-note";
import { Button } from "@/components/ui/button";
import { ImpactRow, ImpactSummary, ImpactValue } from "@/components/impact-summary";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, TableWrap } from "@/components/ui/table";
import { centsToDisplay } from "@/lib/utils";
import type { SettingsImpactPreview } from "@/lib/ipc/m3-calc";

// RQ-18/V7.6, T-M7.3-3/-4 — fires only on a Slab table or Royalty save.
// Mirrors the approved prototype's `confirmSettingsRecalc` modal: a
// `.modal-warn` note naming the open month, the Rewards before/after total,
// a royalty-earner count only for a royalty change, and the affected
// members themselves (capped, "and N more").
const MOVERS_SHOWN = 4;

interface RecalcWarningDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  kind: "slab" | "royalty";
  monthName: string;
  preview: SettingsImpactPreview | null;
  busy?: boolean;
  onConfirm: () => void;
}

function RecalcWarningDialog({
  open,
  onOpenChange,
  kind,
  monthName,
  preview,
  busy,
  onConfirm,
}: RecalcWarningDialogProps) {
  const rewardsChanged = preview ? preview.rewardsBefore !== preview.rewardsAfter : false;
  const movers = preview?.affectedMembers ?? [];
  const shown = movers.slice(0, MOVERS_SHOWN);
  const title = kind === "slab" ? "Slab table changes" : "Royalty changes";
  const confirmLabel = `Save and re-work ${monthName.split(" ")[0]}`;

  return (
    <Modal open={open} onOpenChange={onOpenChange}>
      <ModalHeader title={title} />
      <ModalBody>
        <AlertNote variant="warn">
          <strong>{monthName} is re-worked as soon as you save.</strong> Closed months are never
          affected — their permanent records stay exactly as they are.
        </AlertNote>
        {preview && (
          <>
            <ImpactSummary className="mt-3">
              <ImpactRow label="Total Rewards">
                <ImpactValue
                  before={centsToDisplay(preview.rewardsBefore)}
                  after={centsToDisplay(preview.rewardsAfter)}
                  changed={rewardsChanged}
                />
              </ImpactRow>
              {kind === "royalty" && (
                <ImpactRow label="Members earning royalty">
                  <ImpactValue
                    before={preview.royaltyEarnerCountBefore}
                    after={preview.royaltyEarnerCountAfter}
                    changed={preview.royaltyEarnerCountBefore !== preview.royaltyEarnerCountAfter}
                  />
                </ImpactRow>
              )}
            </ImpactSummary>
            {movers.length > 0 ? (
              <TableWrap className="mt-3">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Member</TableHead>
                      <TableHead numeric>{kind === "slab" ? "Slab" : "Royalty"}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {shown.map((m) => (
                      <TableRow key={m.memberId}>
                        <TableCell primary>{m.memberName}</TableCell>
                        <TableCell numeric>
                          {kind === "slab"
                            ? `${m.slabPctBefore}% → ${m.slabPctAfter}%`
                            : m.royaltyAfter > 0
                              ? "Starts"
                              : "Stops"}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableWrap>
            ) : (
              <p className="text-body text-muted-text mt-3">
                No member&apos;s figures change under these settings.
              </p>
            )}
            {movers.length > MOVERS_SHOWN && (
              <p className="text-label text-muted-text mt-1.5">
                and {movers.length - MOVERS_SHOWN} more
              </p>
            )}
          </>
        )}
      </ModalBody>
      <ModalFooter>
        <ModalCancel />
        <Button disabled={!preview || busy} onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </ModalFooter>
    </Modal>
  );
}

export { RecalcWarningDialog };
