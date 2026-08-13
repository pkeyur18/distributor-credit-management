import { useEffect, useRef } from "react";
import { useNavigate } from "react-router";

import { lockSession } from "@/lib/ipc/m8-auth";
import { getSettings } from "@/lib/ipc/m7-settings";
import { useAuth } from "@/lib/auth-context";

// T-M8.3-2 (D-4). Used until `get_settings` resolves, and as a fallback if
// the read fails.
const DEFAULT_TIMEOUT_MINUTES = 15;
const ACTIVITY_EVENTS = ["mousemove", "keydown", "mousedown", "scroll", "touchstart"] as const;

/** Mounted once, inside the authenticated shell only (`AppShell`) — an
 * auth screen has nothing to time out of. */
export function useInactivityLock() {
  const { markLocked } = useAuth();
  const navigate = useNavigate();
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const timeoutMinutes = useRef(DEFAULT_TIMEOUT_MINUTES);

  useEffect(() => {
    let cancelled = false;

    function reset() {
      if (timer.current) clearTimeout(timer.current);
      timer.current = setTimeout(
        () => {
          lockSession()
            .catch(() => {})
            .finally(() => {
              markLocked();
              navigate("/auth/locked", { replace: true });
            });
        },
        timeoutMinutes.current * 60 * 1000,
      );
    }

    // Re-arms with the real value once loaded, since an idle user (no
    // activity event) would otherwise keep the DEFAULT_TIMEOUT_MINUTES
    // timer armed at mount forever.
    getSettings().then((s) => {
      if (cancelled) return;
      timeoutMinutes.current = s.sessionTimeoutMinutes;
      reset();
    }, () => {});

    ACTIVITY_EVENTS.forEach((event) => window.addEventListener(event, reset));
    reset();

    return () => {
      cancelled = true;
      ACTIVITY_EVENTS.forEach((event) => window.removeEventListener(event, reset));
      if (timer.current) clearTimeout(timer.current);
    };
  }, [markLocked, navigate]);
}
