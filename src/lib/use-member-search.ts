import { useEffect, useState } from "react";

import { searchMembers } from "@/lib/ipc/m1-members";
import type { SearchResult } from "@/lib/ipc/entities";

// T-M1.4-1: the one hook behind every search box in the console — Home,
// Structure, Business Volume Entry, Correction panel, and the Add-Member
// reference lookup (`activeOnly`, Rule-30). A screen wiring its own fetch
// instead of this hook is a defect, not a variation.
export function useMemberSearch(query: string, activeOnly = false) {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);

  const trimmed = query.trim();

  useEffect(() => {
    if (!trimmed) return;
    let cancelled = false;
    const timer = setTimeout(() => {
      if (cancelled) return;
      setLoading(true);
      searchMembers(trimmed, activeOnly)
        .then((found) => {
          if (!cancelled) setResults(found);
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [trimmed, activeOnly]);

  // Derived, not effect-cleared: an empty query has no results by
  // definition (V4.1), so there's nothing to reset when it empties out —
  // `results` simply isn't looked at until a real query fills it again.
  return { results: trimmed ? results : [], loading: trimmed ? loading : false };
}
