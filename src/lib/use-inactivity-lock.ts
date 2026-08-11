import { useEffect, useRef } from "react";
import { useNavigate } from "react-router";

import { lockSession } from "@/lib/ipc/m8-auth";
import { useAuth } from "@/lib/auth-context";

// T-M8.3-2: default 15 minutes (D-4). Spec-wise this should read
// `session_timeout_minutes` from `settings` — but `get_settings` (US-M7.1)
// doesn't exist until S10, and there's no other command in the closed
// 40-command surface that exposes a single setting early. Hardcoded here
// until then; swap for a real read once get_settings ships, not before.
const DEFAULT_TIMEOUT_MINUTES = 15;
const ACTIVITY_EVENTS = ["mousemove", "keydown", "mousedown", "scroll", "touchstart"] as const;

/** Mounted once, inside the authenticated shell only (`AppShell`) — an
 * auth screen has nothing to time out of. */
export function useInactivityLock() {
  const { markLocked } = useAuth();
  const navigate = useNavigate();
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
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
        DEFAULT_TIMEOUT_MINUTES * 60 * 1000,
      );
    }

    ACTIVITY_EVENTS.forEach((event) => window.addEventListener(event, reset));
    reset();

    return () => {
      ACTIVITY_EVENTS.forEach((event) => window.removeEventListener(event, reset));
      if (timer.current) clearTimeout(timer.current);
    };
  }, [markLocked, navigate]);
}
