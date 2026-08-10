import { useState, type ReactNode } from "react";
import { Shield } from "lucide-react";

import { useTheme } from "@/lib/use-theme";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/ui/pill";
import { Input, InputHint } from "@/components/ui/input";
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableWrap,
} from "@/components/ui/table";
import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "@/components/ui/dialog";
import { ToastProvider, Toaster, useToast } from "@/components/ui/toast";
import { AlertNote } from "@/components/ui/alert-note";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { PinDots } from "@/components/ui/pin-input";
import { AuthBrandMark } from "@/components/auth-brand-mark";
import { StructureTreeNode } from "@/components/structure-tree-node";
import { BarListChart } from "@/components/bar-list-chart";
import { ImpactRow, ImpactSummary, ImpactValue } from "@/components/impact-summary";
import { RestoreOptionList } from "@/components/restore-option-list";
import { AddMemberModal } from "@/components/add-member-modal";
import type { AddMemberOutcome } from "@/lib/ipc/m1-members";
import type { Member } from "@/lib/ipc/entities";

// Sprint 3 (US-UI.3/US-UI.4) DoD item 13 verification aid: every component
// this sprint built, in every documented variant, in one place — so the
// light/dark walkthrough doesn't wait for a real screen to consume them.
// Dev-only; never routed, never shipped (gated at the call site by
// import.meta.env.DEV).

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mb-10">
      <h2 className="text-title mb-3">{title}</h2>
      <div className="flex flex-wrap items-start gap-4">{children}</div>
    </section>
  );
}

function ToastTriggers() {
  const toast = useToast();
  return (
    <>
      <Button onClick={() => toast.add({ title: "Business Volume saved" })}>Default toast</Button>
      <Button onClick={() => toast.add({ title: "Month closed", type: "success" })}>
        Success toast
      </Button>
      <Button onClick={() => toast.add({ title: "Restore refused — checksum mismatch", type: "danger" })}>
        Danger toast
      </Button>
      <Toaster />
    </>
  );
}

function ModalTriggers() {
  const [dismissable, setDismissable] = useState(false);
  const [nonDismissable, setNonDismissable] = useState(false);
  return (
    <>
      <Button variant="secondary" onClick={() => setDismissable(true)}>
        Dismissable modal
      </Button>
      <Button variant="secondary" onClick={() => setNonDismissable(true)}>
        Non-dismissable modal
      </Button>
      <Modal open={dismissable} onOpenChange={setDismissable}>
        <ModalHeader title="Restore from a backup" />
        <ModalBody>Escape and backdrop-click both close this one.</ModalBody>
        <ModalFooter>
          <ModalCancel />
          <Button variant="primary">Restore</Button>
        </ModalFooter>
      </Modal>
      <Modal open={nonDismissable} onOpenChange={setNonDismissable} dismissable={false} wide>
        <ModalHeader title="Add member" />
        <ModalBody>Only Cancel or ✕ close this one — Escape and backdrop-click are refused.</ModalBody>
        <ModalFooter>
          <ModalCancel />
          <Button variant="commit">Close month</Button>
        </ModalFooter>
      </Modal>
    </>
  );
}

const MOCK_MEMBER: Member = {
  id: 284913,
  name: "Asha Patel",
  phone: "9876543210",
  email: null,
  address: "1 Main Street",
  introducerMemberId: 100001,
  level: 2,
  isActive: false,
  joiningDate: "2026-01-01",
  consentGiven: true,
  consentDate: "2026-01-01",
  createdAt: "2026-01-01",
};

// Sprint 4 (US-M1.1): no login/DB wiring exists yet (S5), so the real
// `addMember` IPC call can't round-trip here — each trigger below injects a
// mocked `onSubmit` via the modal's own dependency-injection prop instead,
// covering the three outcomes T-M1.1-3/4 define.
function AddMemberModalTriggers() {
  const [mode, setMode] = useState<"created" | "reactivation" | "conflict" | null>(null);
  return (
    <>
      <Button variant="secondary" onClick={() => setMode("created")}>
        Add member — success
      </Button>
      <Button variant="secondary" onClick={() => setMode("reactivation")}>
        Add member — reactivation offer
      </Button>
      <Button variant="secondary" onClick={() => setMode("conflict")}>
        Add member — phone conflict
      </Button>
      <AddMemberModal
        open={mode !== null}
        onOpenChange={(open) => !open && setMode(null)}
        onSubmit={async (): Promise<AddMemberOutcome> => {
          if (mode === "reactivation") {
            return { status: "reactivation_offer", existingMember: MOCK_MEMBER };
          }
          if (mode === "conflict") {
            throw {
              kind: "conflict",
              message: "This phone number is already in use by Rahul Shah (#512004).",
            };
          }
          return { status: "created", member: { ...MOCK_MEMBER, isActive: true }, warnings: [] };
        }}
      />
    </>
  );
}

export function ComponentGallery() {
  const [theme, setTheme] = useTheme();
  const [mode, setMode] = useState<"pin" | "password">("pin");
  const [restorePoint, setRestorePoint] = useState<string | null>("2026-06");

  return (
    <ToastProvider>
      <div className="mx-auto max-w-5xl p-8">
        <div className="mb-8 flex items-center justify-between">
          <h1 className="text-headline">Component gallery (dev only)</h1>
          <SegmentedControl
            value={theme === "dark" ? "dark" : "light"}
            onValueChange={setTheme}
            options={[
              { value: "light", label: "Light" },
              { value: "dark", label: "Dark" },
            ]}
          />
        </div>

        <Section title="Buttons">
          <Button variant="primary">Primary</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="danger">Danger</Button>
          <Button variant="commit">Close month</Button>
          <Button variant="primary" size="sm">
            Small
          </Button>
          <Button variant="primary" disabled>
            Disabled
          </Button>
        </Section>

        <Section title="Pills">
          <Pill variant="active">Active</Pill>
          <Pill variant="inactive">Inactive</Pill>
          <Pill variant="slab">8%</Pill>
          <Pill variant="locked">Locked</Pill>
          <Pill variant="neutral">Neutral</Pill>
        </Section>

        <Section title="Inputs">
          <div className="w-56">
            <Input placeholder="Business Volume" />
          </div>
          <div className="w-56">
            <Input placeholder="Amount" aria-invalid />
            <InputHint error>Enter an amount greater than zero.</InputHint>
          </div>
          <div className="w-56">
            <Input placeholder="Disabled" disabled />
          </div>
        </Section>

        <Section title="Cards">
          <Card className="w-64">
            <CardHeader>
              <CardTitle>Member Directory</CardTitle>
            </CardHeader>
            <CardDescription>500 members, 4 levels deep.</CardDescription>
          </Card>
          <Card variant="stat" className="w-40">
            <div className="text-label">Total members</div>
            <div className="text-numeric num">4,182</div>
          </Card>
        </Section>

        <Section title="Table">
          <TableWrap className="w-full">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Number</TableHead>
                  <TableHead numeric>Business Volume</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow clickable>
                  <TableCell primary>Asha Patel</TableCell>
                  <TableCell className="mono">284913</TableCell>
                  <TableCell numeric>12,450.00</TableCell>
                </TableRow>
                <TableRow clickable>
                  <TableCell primary>Rahul Shah</TableCell>
                  <TableCell className="mono">512004</TableCell>
                  <TableCell numeric>0.00</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TableWrap>
        </Section>

        <Section title="Modal">
          <ModalTriggers />
        </Section>

        <Section title="Add member modal">
          <AddMemberModalTriggers />
        </Section>

        <Section title="Toast">
          <ToastTriggers />
        </Section>

        <Section title="Alert notes">
          <div className="w-96">
            <AlertNote variant="warn">
              <b>June is re-worked as soon as you save.</b> Closed months are never affected.
            </AlertNote>
          </div>
          <div className="w-96">
            <AlertNote variant="danger">
              Enter an amount greater than zero and a date within the active period.
            </AlertNote>
          </div>
        </Section>

        <Section title="Segmented control">
          <SegmentedControl
            value={mode}
            onValueChange={setMode}
            options={[
              { value: "pin", label: "PIN" },
              { value: "password", label: "Password" },
            ]}
          />
        </Section>

        <Section title="PIN dots">
          <PinDots filledCount={3} />
          <PinDots filledCount={6} error />
        </Section>

        <Section title="Auth brand mark">
          <AuthBrandMark size={40} tone="accent">
            <Shield className="size-5" />
          </AuthBrandMark>
          <AuthBrandMark size={52} tone="success">
            <Shield className="size-6" />
          </AuthBrandMark>
          <AuthBrandMark size={52} tone="danger">
            <Shield className="size-6" />
          </AuthBrandMark>
        </Section>

        <Section title="Structure tree node">
          <StructureTreeNode name="Top Member" memberNumber="100001" ownBusinessVolume="500" root legCount={3} />
          <StructureTreeNode name="Asha Patel" memberNumber="284913" ownBusinessVolume="12,450" legCount={2} open />
          <StructureTreeNode name="Leaf Member" memberNumber="391027" ownBusinessVolume="0" legCount={0} />
          <StructureTreeNode name="Rahul Shah" memberNumber="512004" ownBusinessVolume="0" legCount={0} inactive />
          <StructureTreeNode
            name="Read-only (Full Hierarchy)"
            memberNumber="284913"
            ownBusinessVolume="12,450"
            legCount={2}
            interactive={false}
          />
        </Section>

        <Section title="Bar-list chart">
          <div className="w-72">
            <BarListChart
              size="lg"
              rows={[
                { id: 1, label: "8%", value: 42, fraction: 1 },
                { id: 2, label: "12%", value: 17, fraction: 0.4 },
                { id: 3, label: "18%", value: 6, fraction: 0.14 },
              ]}
            />
          </div>
        </Section>

        <Section title="Impact summary">
          <div className="w-96">
            <ImpactSummary>
              <ImpactRow label="Rewards this month">
                <ImpactValue before="980.00" after="1,120.00" changed />
              </ImpactRow>
              <ImpactRow label="Members earning royalty">
                <ImpactValue before={3} after={3} changed={false} />
              </ImpactRow>
            </ImpactSummary>
          </div>
        </Section>

        <Section title="Restore option list">
          <div className="w-96">
            <RestoreOptionList
              value={restorePoint}
              onValueChange={setRestorePoint}
              options={[
                { value: "2026-06", primary: "June 2026", provenance: "Version 1" },
                { value: "2026-05", primary: "May 2026 (corrected)", provenance: "Version 2" },
              ]}
            />
          </div>
        </Section>
      </div>
    </ToastProvider>
  );
}
