"use client";

import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

/**
 * Reveals its children with a subtle slide-up-fade the first time they scroll into view.
 *
 * The animation is a progressive enhancement, never a gate on visibility: the content is
 * rendered visible, and this only *hides* an element (to animate it in on scroll) when it's
 * genuinely below the fold and there's an IntersectionObserver to bring it back. So no-JS,
 * a stalled observer, prefers-reduced-motion, or a screenshot tool that never scrolls all
 * leave the content on screen rather than stranded at opacity:0.
 */
export function Reveal({
  children,
  className,
  delay = 0,
  as: Tag = "div",
}: {
  children: React.ReactNode;
  className?: string;
  delay?: number;
  as?: "div" | "li" | "section";
}) {
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    // Already visible (on screen or above it), or no observer to reveal with → leave it be.
    if (typeof IntersectionObserver === "undefined") return;
    if (el.getBoundingClientRect().top < window.innerHeight) return;

    // Below the fold: hide it now (off-screen, so no visible flash) and animate it in on scroll.
    el.dataset.reveal = "pending";
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) {
            delete el.dataset.reveal;
            io.disconnect();
          }
        }
      },
      { threshold: 0.12, rootMargin: "0px 0px -8% 0px" },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <Tag
      // @ts-expect-error — ref typing across the union of tags is fine at runtime
      ref={ref}
      className={cn("reveal", className)}
      style={{ transitionDelay: `${delay}ms` }}
    >
      {children}
    </Tag>
  );
}
