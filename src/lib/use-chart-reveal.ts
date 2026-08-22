import { useEffect, useRef, useState } from "react";

function prefersReducedMotion(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** Reveals once, the first time the element scrolls into view — drives the
 * slab charts' grow-in animation (US-M4.4 chart redesign). Falls back to
 * already-revealed when IntersectionObserver isn't available (jsdom tests). */
export function useRevealOnView<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [revealed, setRevealed] = useState(false);

  useEffect(() => {
    if (revealed) return;
    const el = ref.current;
    if (!el || typeof IntersectionObserver === "undefined") {
      setRevealed(true);
      return;
    }
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setRevealed(true);
          observer.disconnect();
        }
      },
      { threshold: 0.3 },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [revealed]);

  return { ref, revealed };
}

/** Milliseconds elapsed since `active` became true, ticking every animation
 * frame up to `totalMs`. Jumps straight to `totalMs` under reduced motion
 * (or when requestAnimationFrame isn't available). */
export function useElapsedSinceActive(active: boolean, totalMs: number) {
  // Reduced motion (or no rAF, e.g. jsdom) skips the animation loop entirely
  // — this is derived at render time, never via setState-in-effect, so
  // there's nothing to synchronize for that path.
  const skipAnimation = prefersReducedMotion() || typeof requestAnimationFrame === "undefined";
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!active || skipAnimation) return;
    let raf = 0;
    const start = performance.now();
    const tick = (now: number) => {
      const e = now - start;
      if (e >= totalMs) {
        setElapsed(totalMs);
        return;
      }
      setElapsed(e);
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [active, totalMs, skipAnimation]);

  if (skipAnimation) return active ? totalMs : 0;
  return elapsed;
}

/** Ease-out-cubic progress (0-1) for row `index`, staggered `stagger`ms apart
 * over `duration`ms each — the same curve/timing as the approved redesign. */
export function rowProgress(elapsedMs: number, index: number, stagger: number, duration: number) {
  const t = Math.min(1, Math.max(0, (elapsedMs - index * stagger) / duration));
  return 1 - (1 - t) ** 3;
}
