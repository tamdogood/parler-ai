"use client";

import { useId, useState } from "react";
import { ArrowRight, Check, Mail } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { HUB_API } from "@/lib/api";

type Status = "idle" | "submitting" | "success" | "error";

// A basic client-side sanity check so an obvious typo never round-trips to the hub.
// The server is the source of truth (a 400 still renders the invalid-email message).
const LOOKS_LIKE_EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

const INVALID_MSG = "That email doesn't look right.";
const UNREACHABLE_MSG = "Couldn't reach the hub. Try again in a minute.";

/**
 * The owned list. One field, one button — POSTs to the hub's `/api/waitlist`
 * (`{ ok: true }` on 200, 400 = bad email, 429 / network = try again later).
 * Design-system native; lives directly under the sessions wedge so it captures
 * the reader at the moment the payoff lands.
 */
export function EmailCapture() {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<Status>("idle");
  const [error, setError] = useState("");
  const inputId = useId();

  const submitting = status === "submitting";
  const done = status === "success";

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    const value = email.trim();
    if (submitting || done) return;

    if (!LOOKS_LIKE_EMAIL.test(value)) {
      setStatus("error");
      setError(INVALID_MSG);
      return;
    }

    setStatus("submitting");
    setError("");

    try {
      const res = await fetch(`${HUB_API}/api/waitlist`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: value }),
      });

      if (res.ok) {
        setStatus("success");
        return;
      }

      setStatus("error");
      setError(res.status === 400 ? INVALID_MSG : UNREACHABLE_MSG);
    } catch {
      setStatus("error");
      setError(UNREACHABLE_MSG);
    }
  }

  return (
    <div className="rounded-[16px] border border-graphite-rail bg-void-black p-6 sm:p-8">
      <div className="flex items-start gap-3">
        <span className="flex size-10 shrink-0 items-center justify-center rounded-[12px] border border-graphite-rail surface-lift">
          <Mail className="size-5 text-electric-blue" />
        </span>
        <div>
          <h3 className="text-[16px] font-semibold text-pure-white">Get the 3-step setup</h3>
          <p className="mt-1 max-w-xl text-[14px] leading-relaxed text-fog">
            Drop your email and we&apos;ll send the 3-step setup, plus a heads-up when the team hub is
            ready. No spam, no more than an email or two.
          </p>
        </div>
      </div>

      {done ? (
        <p
          className="mt-5 flex items-center gap-2 text-[14px] font-medium text-delivered-green"
          role="status"
          aria-live="polite"
        >
          <Check className="size-4" />
          You&apos;re on the list.
        </p>
      ) : (
        <form onSubmit={submit} className="mt-5 flex flex-col gap-3 sm:flex-row sm:items-start" noValidate>
          <div className="flex-1">
            <label htmlFor={inputId} className="sr-only">
              Email address
            </label>
            <Input
              id={inputId}
              type="email"
              inputMode="email"
              autoComplete="email"
              placeholder="you@company.com"
              value={email}
              onChange={(e) => {
                setEmail(e.target.value);
                if (status === "error") {
                  setStatus("idle");
                  setError("");
                }
              }}
              disabled={submitting}
              aria-invalid={status === "error"}
              aria-describedby={error ? `${inputId}-error` : undefined}
            />
          </div>
          <Button
            type="submit"
            variant="cta"
            size="default"
            disabled={submitting}
            className="shrink-0 sm:w-auto"
          >
            {submitting ? "Sending…" : "Notify me"}
            {!submitting && <ArrowRight className="size-4" />}
          </Button>
        </form>
      )}

      {/* Errors are announced assertively; the field is re-usable so the reader can fix and resubmit. */}
      {error && !done && (
        <p
          id={`${inputId}-error`}
          className="mt-3 text-[13px] text-bounced-red"
          role="alert"
          aria-live="assertive"
        >
          {error}
        </p>
      )}
    </div>
  );
}
