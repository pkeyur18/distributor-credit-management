import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./table";

describe("Table", () => {
  it("right-aligns numeric cells and headers, left-aligns everything else", () => {
    render(
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead numeric>Total Business Volume</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>Asha Patel</TableCell>
            <TableCell numeric>1,250.50</TableCell>
          </TableRow>
        </TableBody>
      </Table>,
    );
    expect(screen.getByText("Total Business Volume")).toHaveClass("text-right");
    expect(screen.getByText("Name")).toHaveClass("text-left");
    expect(screen.getByText("1,250.50")).toHaveClass("text-right", "num");
  });

  it("marks a row clickable with a hover affordance", () => {
    render(
      <Table>
        <TableBody>
          <TableRow clickable data-testid="row">
            <TableCell>Asha Patel</TableCell>
          </TableRow>
        </TableBody>
      </Table>,
    );
    expect(screen.getByTestId("row")).toHaveClass("cursor-pointer");
  });
});
