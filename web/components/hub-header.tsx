import { Globe, Lock } from "lucide-react";
import type { HubSummary } from "@/lib/types";
import { Skeleton } from "@/components/ui/skeleton";

export function HubHeader({ hub }: { hub: HubSummary | null }) {
  if (!hub) {
    return (
      <div className="flex items-center gap-4">
        <Skeleton className="size-11 rounded-[12px]" />
        <div className="space-y-2">
          <Skeleton className="h-5 w-40" />
          <Skeleton className="h-3 w-24" />
        </div>
      </div>
    );
  }
  const isPublic = hub.mode === "public";
  return (
    <div className="flex flex-wrap items-center gap-x-5 gap-y-3">
      <span className="flex size-11 items-center justify-center rounded-[12px] border border-graphite-rail surface-lift">
        {isPublic ? (
          <Globe className="size-5 text-opened-blue" strokeWidth={1.75} />
        ) : (
          <Lock className="size-5 text-complained-yellow" strokeWidth={1.75} />
        )}
      </span>
      <div>
        <div className="flex items-center gap-2.5">
          <h2 className="text-[22px] font-semibold tracking-tight text-pure-white">{hub.name}</h2>
          <span className="rounded-[6px] border border-graphite-rail px-1.5 py-0.5 text-[11px] uppercase tracking-wide text-fog">
            {hub.mode} hub
          </span>
        </div>
        <p className="mt-0.5 text-[13px] text-steel">
          <span className="text-fog">{hub.agents}</span> agent{hub.agents === 1 ? "" : "s"}
          {" · "}
          <span className="text-fog">{hub.publicAgents}</span> public
          {" · "}
          protocol <span className="font-mono text-fog">v{hub.protocolVersion}</span>
        </p>
      </div>
    </div>
  );
}
