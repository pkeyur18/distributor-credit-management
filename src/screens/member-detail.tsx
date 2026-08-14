import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";

import { Button } from "@/components/ui/button";
import { Pill } from "@/components/ui/pill";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableWrap,
} from "@/components/ui/table";
import { EmptyState } from "@/components/empty-state";
import { LoadingState } from "@/components/loading-state";
import { MemberModal } from "@/components/member-modal";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { MonthSwitcher } from "@/components/month-switcher";
import { useToast } from "@/components/ui/toast";
import { getMemberDetail } from "@/lib/ipc/m4-search";
import { deactivateMember, reactivateMember } from "@/lib/ipc/m1-members";
import { getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import type { MemberDetail as MemberDetailData } from "@/lib/ipc/m4-search";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { centsToDisplay } from "@/lib/utils";
import { Breadcrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";

// US-M4.1 (§5.2). Own-Business-Volume reward first (Rule-46/CR-4), then
// every direct leg's differential term, then royalty — Rule-11's note that
// differential and royalty never both pay on the same leg falls out of the
// numbers themselves, not from special-casing a row here. No inline
// entry-edit table: the prototype's version duplicates the Correction
// Panel Sprint 7 already shipped, so it's left out (T-M4.1-1..6's scope).
export function MemberDetail() {
  const { memberId } = useParams<{ memberId: string }>();
  const navigate = useNavigate();
  const toast = useToast();
  const id = Number(memberId);

  const [detail, setDetail] = useState<MemberDetailData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editOpen, setEditOpen] = useState(false);
  const [confirmAction, setConfirmAction] = useState<"deactivate" | "reactivate" | null>(null);
  const [busy, setBusy] = useState(false);

  const backTarget = useBackTarget();
  useRouteLabel(detail?.member.name);

  // T-M2.5-3: this figure screen defaults to the oldest recordable month,
  // switchable when more than one is outstanding (CR-2).
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const viewMonth = selectedMonth ?? lockStatus?.recordablePeriodMonths[0];

  useEffect(() => {
    getPeriodLockStatus().then(setLockStatus);
  }, []);

  useEffect(() => {
    let cancelled = false;
    getMemberDetail(id, viewMonth)
      .then((d) => {
        if (!cancelled) {
          setDetail(d);
          setError(null);
        }
      })
      .catch((raw) => {
        if (!cancelled) setError(toErrorPresentation(raw).message);
      });
    return () => {
      cancelled = true;
    };
  }, [id, viewMonth]);

  async function handleConfirm() {
    if (!detail || !confirmAction) return;
    setBusy(true);
    try {
      if (confirmAction === "deactivate") {
        await deactivateMember(detail.member.id);
        toast.add({ title: "Member deactivated", type: "success" });
      } else {
        await reactivateMember(detail.member.id);
        toast.add({ title: "Member reactivated", type: "success" });
      }
      setDetail({
        ...detail,
        member: { ...detail.member, isActive: confirmAction === "reactivate" },
      });
      setConfirmAction(null);
    } finally {
      setBusy(false);
    }
  }

  if (error) return <EmptyState title="Member not found" description={error} />;
  if (!detail) return <LoadingState />;

  const { member, rewards } = detail;
  const isRoot = member.introducerMemberId == null;

  return (
    <>
      <Breadcrumb
        backLabel={backTarget.label}
        onBack={backTarget.go}
        crumbs={[{ label: "Home", to: "/" }, { label: member.name }]}
      />
      <div className="rounded-lg border border-border bg-surface p-4.5">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <div className="flex items-center gap-1.5 text-title">
              {member.name}
              {!member.isActive && <Pill variant="inactive">Inactive</Pill>}
              {isRoot && <Pill variant="neutral">Root member</Pill>}
            </div>
            <div className="mono mt-0.5 text-caption text-muted-text">
              Member #{member.id} · {member.phone} · Joined {member.joiningDate}
            </div>
          </div>
          <div className="flex gap-2">
            <Button variant="secondary" size="sm" onClick={() => setEditOpen(true)}>
              Edit member
            </Button>
            {member.isActive ? (
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
            <Button
              variant="primary"
              size="sm"
              onClick={() => navigate(`/entry?member=${member.id}`)}
            >
              Record volume
            </Button>
          </div>
        </div>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
        <StatCard
          label="Business Volume"
          value={centsToDisplay(rewards.ownReward.ownBusinessVolume)}
        />
        <StatCard
          label="Total Business Volume"
          value={centsToDisplay(detail.totalBusinessVolume)}
        />
        <StatCard label="Slab" value={`${detail.slabPct}%`} />
        <StatCard label="Rewards this period" value={centsToDisplay(rewards.rewardsTotal)} />
      </div>

      {lockStatus && (
        <MonthSwitcher
          months={lockStatus.recordablePeriodMonths}
          value={viewMonth ?? lockStatus.recordablePeriodMonths[0]}
          onChange={setSelectedMonth}
        />
      )}

      <div className="mt-4 grid gap-4 lg:grid-cols-[1.4fr_1fr]">
        <div className="rounded-lg border border-border bg-surface p-4.5">
          <div className="text-title-sm">Rewards detail</div>
          <p className="text-caption mt-0.5 mb-3">
            Own Business Volume, differential and royalty — differential and royalty never pay on
            the same leg.
          </p>
          <TableWrap>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Leg</TableHead>
                  <TableHead numeric>Total Business Volume</TableHead>
                  <TableHead>Slab</TableHead>
                  <TableHead numeric>Gap</TableHead>
                  <TableHead numeric>Amount</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell colSpan={3} primary>
                    Own Business Volume{" "}
                    <span className="font-normal text-muted-text">
                      — {centsToDisplay(rewards.ownReward.ownBusinessVolume)} at{" "}
                      {rewards.ownReward.ownSlabPct}%
                    </span>
                  </TableCell>
                  <TableCell />
                  <TableCell numeric primary>
                    {centsToDisplay(rewards.ownReward.amount)}
                  </TableCell>
                </TableRow>
                {rewards.differentials.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-muted-text">
                      No direct legs — differential and royalty are earned on the gap to each direct
                      leg's slab
                    </TableCell>
                    <TableCell numeric className="text-muted-text">
                      0.00
                    </TableCell>
                  </TableRow>
                ) : (
                  <>
                    {rewards.differentials.map((line) => (
                      <TableRow
                        key={line.childId}
                        clickable
                        onClick={() => navigate(`/member/${line.childId}`)}
                      >
                        <TableCell primary>{line.childName}</TableCell>
                        <TableCell numeric>
                          {centsToDisplay(line.childTotalBusinessVolume)}
                        </TableCell>
                        <TableCell>
                          <Pill variant="slab">{line.childSlabPct}%</Pill>
                        </TableCell>
                        <TableCell numeric className="text-muted-text">
                          {line.ownSlabPct}% − {line.childSlabPct}%
                        </TableCell>
                        <TableCell numeric primary>
                          {centsToDisplay(line.amount)}
                        </TableCell>
                      </TableRow>
                    ))}
                    {rewards.royalty && (
                      <TableRow>
                        <TableCell colSpan={3} primary>
                          Royalty{" "}
                          <span className="font-normal text-muted-text">
                            — {rewards.royalty.qualifyingChildren} of {rewards.differentials.length}{" "}
                            legs qualifying (top slab)
                          </span>
                        </TableCell>
                        <TableCell />
                        <TableCell numeric primary>
                          {centsToDisplay(rewards.royalty.amount)}
                        </TableCell>
                      </TableRow>
                    )}
                  </>
                )}
                <TableRow className="bg-bg font-[650]">
                  <TableCell colSpan={4}>Rewards total</TableCell>
                  <TableCell numeric>{centsToDisplay(rewards.rewardsTotal)}</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TableWrap>
        </div>

        <div className="rounded-lg border border-border bg-surface p-4.5">
          <div className="text-title-sm">Details</div>
          <div className="mt-3 flex flex-col gap-2.25 text-body">
            <DetailRow label="Address" value={member.address} />
            <DetailRow label="Email" value={member.email ?? "Not provided"} />
            <DetailRow
              label="Introduced by"
              value={
                isRoot ? (
                  "None — root member"
                ) : (
                  <button
                    type="button"
                    className="text-accent"
                    onClick={() => navigate(`/member/${member.introducerMemberId}`)}
                  >
                    #{member.introducerMemberId}
                  </button>
                )
              }
            />
            <DetailRow label="Direct legs" value={String(detail.legCount)} />
            <DetailRow label="Consent captured" value={member.consentDate} />
          </div>
          <hr className="my-3.5 border-border" />
          <Button
            variant="secondary"
            className="w-full"
            onClick={() => navigate(`/structure/${member.id}`)}
          >
            View in structure
          </Button>
        </div>
      </div>

      {detail.directChildren.length > 0 && (
        <div className="mt-4 rounded-lg border border-border bg-surface p-4.5">
          <div className="text-title-sm mb-3">Direct legs ({detail.directChildren.length})</div>
          <TableWrap>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Member #</TableHead>
                  <TableHead numeric>Total Business Volume</TableHead>
                  <TableHead>Slab</TableHead>
                  <TableHead>Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {detail.directChildren.map((child) => (
                  <TableRow
                    key={child.memberId}
                    clickable
                    onClick={() => navigate(`/member/${child.memberId}`)}
                  >
                    <TableCell primary>{child.name}</TableCell>
                    <TableCell className="mono">{child.memberId}</TableCell>
                    <TableCell numeric>{centsToDisplay(child.totalBusinessVolume)}</TableCell>
                    <TableCell>
                      <Pill variant="slab">{child.slabPct}%</Pill>
                    </TableCell>
                    <TableCell>
                      <Pill variant={child.isActive ? "active" : "inactive"}>
                        {child.isActive ? "Active" : "Inactive"}
                      </Pill>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableWrap>
        </div>
      )}

      <MemberModal
        open={editOpen}
        onOpenChange={setEditOpen}
        mode="edit"
        member={member}
        onSaved={(m) =>
          setDetail({
            ...detail,
            member: {
              ...detail.member,
              name: m.name,
              phone: m.phone,
              email: m.email,
              address: m.address,
            },
          })
        }
      />

      {confirmAction && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && setConfirmAction(null)}
          title={
            confirmAction === "deactivate"
              ? `Deactivate ${member.name}?`
              : `Reactivate ${member.name}?`
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

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-border bg-surface p-3.5">
      <div className="text-label text-muted-text">{label}</div>
      <div className="num mt-1 text-numeric-lg">{value}</div>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div>
      <span className="text-muted-text">{label}</span>
      <div>{value}</div>
    </div>
  );
}
