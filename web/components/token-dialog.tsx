"use client";

import { useState } from "react";
import { KeyRound } from "lucide-react";
import { Dialog, DialogModalContent, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { setDirectoryToken } from "@/lib/api";

export function TokenDialog({
  open,
  onOpenChange,
  onSaved,
  hasToken,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  onSaved: () => void;
  hasToken: boolean;
}) {
  const [value, setValue] = useState("");

  function save() {
    const t = value.trim();
    if (!t) return;
    setDirectoryToken(t);
    setValue("");
    onSaved();
    onOpenChange(false);
  }

  function clear() {
    setDirectoryToken(null);
    setValue("");
    onSaved();
    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogModalContent>
        <div className="flex items-center gap-2.5">
          <span className="flex size-9 items-center justify-center rounded-[10px] border border-graphite-rail surface-lift">
            <KeyRound className="size-4 text-electric-blue" />
          </span>
          <div>
            <DialogTitle>Unlock the hub directory</DialogTitle>
          </div>
        </div>
        <DialogDescription className="mt-3">
          A private hub&apos;s full directory (including private agents) is gated. Paste a directory
          token — any member can mint one with{" "}
          <code className="rounded-[4px] border border-graphite-rail px-1 py-0.5 font-mono text-[12px] text-resend-violet">
            parler token
          </code>
          .
        </DialogDescription>

        <Input
          className="mt-5 font-mono"
          placeholder="e.g. VDXNMKGDQFQAHHUN9M9JXQLE…"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && save()}
          autoFocus
        />

        <div className="mt-5 flex items-center justify-between">
          {hasToken ? (
            <Button variant="subtle" size="sm" onClick={clear}>
              Remove saved token
            </Button>
          ) : (
            <span />
          )}
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button variant="primary" size="sm" onClick={save}>
              Unlock
            </Button>
          </div>
        </div>
      </DialogModalContent>
    </Dialog>
  );
}
