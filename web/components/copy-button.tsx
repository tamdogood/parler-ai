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
      className={cn("text-steel transition-colors hover:text-frost", className)}
      aria-label="Copy"
    >
      {copied ? <Check className="size-3.5 text-delivered-green" /> : <Copy className="size-3.5" />}
    </button>
  );
}
