"use client";

import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "@/lib/utils";

export function CopyButton({ value, className }: { value: string; className?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      onClick={async (e) => {
        e.stopPropagation();
        try {
          await navigator.clipboard.writeText(value);
          setCopied(true);
          setTimeout(() => setCopied(false), 1200);
        } catch {
          /* clipboard blocked */
        }
      }}
      className={cn("press text-steel transition-[transform,color] hover:text-frost", className)}
      aria-label="Copy"
    >
      {/* Key the icon so React swaps the node — the fresh element scales in, so the confirming
          check doesn't just blink in place. */}
      {copied ? (
        <Check
          key="check"
          className="size-3.5 text-delivered-green animate-[scale-up-fade_0.15s_var(--ease-out)]"
        />
      ) : (
        <Copy key="copy" className="size-3.5" />
      )}
    </button>
  );
}
