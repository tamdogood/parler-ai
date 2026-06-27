"use client";

import { Globe, Lock } from "lucide-react";
import type { DirectoryEntry } from "@/lib/types";
import { relativeTime } from "@/lib/utils";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { StatusDot, StatusLabel } from "@/components/status-dot";
import { VerifiedBadge } from "@/components/verified-badge";
import { CopyButton } from "@/components/copy-button";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="text-[11px] uppercase tracking-wide text-steel">{label}</div>
      <div className="mt-1 text-[14px] text-frost">{children}</div>
    </div>
  );
}

export function AgentDetail({
  entry,
  onClose,
}: {
  entry: DirectoryEntry | null;
  onClose: () => void;
}) {
  return (
    <Dialog open={!!entry} onOpenChange={(o) => !o && onClose()}>
      {entry && (
        <DialogContent>
          {/* Header */}
          <div className="flex items-start gap-3">
            <StatusDot status={entry.status} className="mt-2" />
            <div className="min-w-0 flex-1">
              <DialogTitle>{entry.card.name}</DialogTitle>
              <DialogDescription>
                {entry.card.role ?? "agent"} · {entry.hub}
              </DialogDescription>
            </div>
            <VerifiedBadge verified={entry.verified} withLabel />
          </div>

          {/* id */}
          <div className="flex items-center gap-2 rounded-[10px] border border-graphite-rail bg-white/[0.02] px-3 py-2">
            <span className="truncate font-mono text-[12px] text-resend-violet">{entry.card.id}</span>
            <CopyButton value={entry.card.id} className="ml-auto" />
          </div>

          {entry.card.description && (
            <p className="text-[14px] leading-relaxed text-fog">{entry.card.description}</p>
          )}

          {/* meta grid */}
          <div className="grid grid-cols-2 gap-5">
            <Field label="Status">
              <StatusLabel status={entry.status} />
            </Field>
            <Field label="Visibility">
              <span className="inline-flex items-center gap-1.5">
                {entry.visibility === "public" ? (
                  <Globe className="size-3.5 text-opened-blue" />
                ) : (
                  <Lock className="size-3.5 text-complained-yellow" />
                )}
                {entry.visibility}
              </span>
            </Field>
            <Field label="First seen">{relativeTime(entry.firstSeen)}</Field>
            <Field label="Last seen">{relativeTime(entry.lastSeen)}</Field>
          </div>

          {entry.activity && (
            <Field label="Now">
              <span className="text-fog">{entry.activity}</span>
            </Field>
          )}

          {/* skills */}
          {entry.card.skills && entry.card.skills.length > 0 && (
            <div>
              <div className="text-[11px] uppercase tracking-wide text-steel">Skills</div>
              <ul className="mt-2 space-y-2">
                {entry.card.skills.map((s) => (
                  <li
                    key={s.id}
                    className="rounded-[8px] border border-graphite-rail px-3 py-2 text-[13px]"
                  >
                    <div className="font-medium text-frost">{s.name}</div>
                    {s.description && <div className="mt-0.5 text-fog">{s.description}</div>}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* tags */}
          {entry.card.tags && entry.card.tags.length > 0 && (
            <div>
              <div className="text-[11px] uppercase tracking-wide text-steel">Tags</div>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {entry.card.tags.map((t) => (
                  <Badge key={t}>{t}</Badge>
                ))}
              </div>
            </div>
          )}

          {/* signature */}
          <div className="mt-auto rounded-[10px] border border-graphite-rail p-3">
            <div className="flex items-center justify-between">
              <span className="text-[11px] uppercase tracking-wide text-steel">Signature</span>
              <VerifiedBadge verified={entry.verified} withLabel />
            </div>
            <div className="mt-2 break-all font-mono text-[11px] leading-relaxed text-steel">
              {entry.sig ? `${entry.sig.slice(0, 44)}…` : "— unsigned —"}
            </div>
            <p className="mt-2 text-[11px] leading-relaxed text-steel">
              {entry.verified
                ? "Signed by the agent's own nkey over the canonical card. The hub can't forge it."
                : "No valid signature — treat this listing with caution."}
            </p>
          </div>
        </DialogContent>
      )}
    </Dialog>
  );
}
