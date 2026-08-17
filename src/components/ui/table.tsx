import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// 07-design-system.md §Layout/§6 — ~40px row density, uppercase muted
// column headers (the Label typography role), 1px hairline separators.

function TableWrap({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="table-wrap"
      className={cn("overflow-x-auto rounded-lg border border-border bg-surface", className)}
      {...props}
    />
  );
}

function Table({ className, ...props }: ComponentProps<"table">) {
  return (
    <table
      data-slot="table"
      className={cn("w-full border-collapse text-[13px]", className)}
      {...props}
    />
  );
}

function TableHeader({ className, ...props }: ComponentProps<"thead">) {
  return <thead data-slot="table-header" className={className} {...props} />;
}

function TableBody({ className, ...props }: ComponentProps<"tbody">) {
  return <tbody data-slot="table-body" className={className} {...props} />;
}

function TableRow({
  className,
  clickable,
  ...props
}: ComponentProps<"tr"> & { clickable?: boolean }) {
  return (
    <tr
      data-slot="table-row"
      className={cn(
        "[&:not(:last-child)>td]:border-b [&:not(:last-child)>td]:border-border",
        clickable && "cursor-pointer hover:bg-bg",
        className,
      )}
      {...props}
    />
  );
}

function TableHead({
  className,
  numeric,
  ...props
}: ComponentProps<"th"> & { numeric?: boolean }) {
  return (
    <th
      data-slot="table-head"
      className={cn(
        "text-label border-b border-border px-3.5 py-[9px] whitespace-nowrap",
        numeric ? "text-right" : "text-left",
        className,
      )}
      {...props}
    />
  );
}

function TableCell({
  className,
  numeric,
  primary,
  ...props
}: ComponentProps<"td"> & { numeric?: boolean; primary?: boolean }) {
  return (
    <td
      data-slot="table-cell"
      className={cn(
        "h-10 px-3.5 align-middle",
        numeric ? "num text-right" : "text-left",
        primary && "font-[550]",
        className,
      )}
      {...props}
    />
  );
}

export { TableWrap, Table, TableHeader, TableBody, TableRow, TableHead, TableCell };
