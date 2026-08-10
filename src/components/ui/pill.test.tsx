import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { Pill } from "./pill";

describe("Pill", () => {
  it.each(["active", "inactive", "slab", "locked", "neutral"] as const)(
    "always renders its label text for variant=%s (Colour-Plus-Label Rule)",
    (variant) => {
      render(<Pill variant={variant}>Active</Pill>);
      expect(screen.getByText("Active")).toBeInTheDocument();
    },
  );

  it("suppresses the status dot on slab and neutral, which carry no implied state", () => {
    const { container: slab } = render(<Pill variant="slab">Slab 3</Pill>);
    expect(slab.querySelector("[aria-hidden]")).not.toBeInTheDocument();

    const { container: active } = render(<Pill variant="active">Active</Pill>);
    expect(active.querySelector("[aria-hidden]")).toBeInTheDocument();
  });
});
