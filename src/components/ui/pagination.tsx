import { useState } from "react";

import { Button } from "@/components/ui/button";

export const PAGE_SIZES = [10, 25, 50] as const;

interface PaginationControls {
  totalItems: number;
  rangeStart: number;
  pageSize: number;
  setPageSize: (size: number) => void;
  setPage: (page: number) => void;
  currentPage: number;
  totalPages: number;
  goPrev: () => void;
  goNext: () => void;
}

// Shared by every screen that paginates a plain client-side row list
// (audit log, correction panel, volume entry) — same slice math and same
// controls, previously re-implemented three times with identical markup.
export function usePagination<T>(
  items: T[],
  initialPageSize: number = PAGE_SIZES[0],
): PaginationControls & { pageItems: T[] } {
  const [page, setPage] = useState(0);
  const [pageSize, setPageSizeState] = useState<number>(initialPageSize);

  const totalPages = Math.max(1, Math.ceil(items.length / pageSize));
  const currentPage = Math.min(page, totalPages - 1);
  const rangeStart = currentPage * pageSize;
  const pageItems = items.slice(rangeStart, rangeStart + pageSize);

  return {
    pageItems,
    totalItems: items.length,
    rangeStart,
    pageSize,
    setPageSize: (size) => {
      setPageSizeState(size);
      setPage(0);
    },
    setPage,
    currentPage,
    totalPages,
    goPrev: () => setPage((p) => p - 1),
    goNext: () => setPage((p) => p + 1),
  };
}

export function TablePagination({
  idPrefix,
  pagination,
}: {
  idPrefix: string;
  pagination: PaginationControls;
}) {
  const { totalItems, rangeStart, pageSize, setPageSize, currentPage, totalPages, goPrev, goNext } =
    pagination;

  return (
    <div className="mt-2.5 flex items-center justify-between">
      <span className="text-caption">
        Showing {rangeStart + 1}–{Math.min(rangeStart + pageSize, totalItems)} of {totalItems}
      </span>
      <div className="flex items-center gap-2">
        <label htmlFor={`${idPrefix}-page-size`} className="text-caption">
          Rows per page
        </label>
        <select
          id={`${idPrefix}-page-size`}
          className="h-7.5 w-auto rounded-sm border border-border bg-surface px-2 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
          value={pageSize}
          onChange={(e) => setPageSize(Number(e.target.value))}
        >
          {PAGE_SIZES.map((size) => (
            <option key={size} value={size}>
              {size}
            </option>
          ))}
        </select>
        <Button variant="secondary" size="sm" disabled={currentPage === 0} onClick={goPrev}>
          Prev
        </Button>
        <Button
          variant="secondary"
          size="sm"
          disabled={currentPage >= totalPages - 1}
          onClick={goNext}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
