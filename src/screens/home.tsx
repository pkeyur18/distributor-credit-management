import { useState } from "react";
import { PlusCircle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { AddMemberModal } from "@/components/add-member-modal";

// US-M1.1 (T-M1.1-7): the "Add member" entry point. Search, stat cards and
// the slab-distribution charts (US-M1.4/M4.4) land in later sprints as
// those stories ship — this screen only carries what S4 delivers.
export function Home() {
  const [addMemberOpen, setAddMemberOpen] = useState(false);

  return (
    <>
      <div className="flex items-center justify-between">
        <h1 className="text-headline">Home</h1>
        <Button variant="primary" onClick={() => setAddMemberOpen(true)}>
          <PlusCircle className="size-4" />
          Add member
        </Button>
      </div>
      <AddMemberModal open={addMemberOpen} onOpenChange={setAddMemberOpen} />
    </>
  );
}
