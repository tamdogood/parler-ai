"use client";

import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;
const DialogClose = DialogPrimitive.Close;

function DialogOverlay({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Overlay>) {
  return (
    <DialogPrimitive.Overlay
      className={cn(
        "fixed inset-0 z-50 bg-black/70 backdrop-blur-sm data-[state=open]:animate-[fade-in_0.2s_var(--ease-out)] data-[state=closed]:animate-[fade-out_0.15s_var(--ease-out)]",
        className,
      )}
      {...props}
    />
  );
}

/** A right-side sheet (Slack-like detail panel) on black with a single hairline left border. */
function DialogContent({
  className,
  children,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Content>) {
  return (
    <DialogPrimitive.Portal>
      <DialogOverlay />
      <DialogPrimitive.Content
        className={cn(
          // A right-side drawer: enter on the iOS drawer curve, exit faster on ease-out
          // (asymmetric — deliberate in, snappy out, so dismissal never feels laggy).
          "fixed inset-y-0 right-0 z-50 flex w-full max-w-md flex-col gap-6 overflow-y-auto border-l border-graphite-rail bg-void-black p-8 outline-none data-[state=open]:animate-[slide-in-right_0.3s_var(--ease-drawer)] data-[state=closed]:animate-[slide-out-right_0.2s_var(--ease-out)]",
          className,
        )}
        {...props}
      >
        {children}
        <DialogPrimitive.Close className="press absolute right-5 top-5 rounded-[6px] p-1 text-steel transition-[transform,color] hover:text-frost focus:outline-none">
          <X className="size-4" />
          <span className="sr-only">Close</span>
        </DialogPrimitive.Close>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

/** A centered modal (used for the token-unlock dialog). */
function DialogModalContent({
  className,
  children,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Content>) {
  return (
    <DialogPrimitive.Portal>
      <DialogOverlay />
      <DialogPrimitive.Content
        className={cn(
          // A centered modal — it isn't anchored to a trigger, so it scales from center (the one
          // case where `transform-origin: center` is correct). Exit is faster than enter.
          "fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-[16px] border border-graphite-rail bg-void-black p-8 outline-none data-[state=open]:animate-[modal-in_0.2s_var(--ease-out)] data-[state=closed]:animate-[modal-out_0.15s_var(--ease-out)]",
          className,
        )}
        {...props}
      >
        {children}
        <DialogPrimitive.Close className="press absolute right-5 top-5 rounded-[6px] p-1 text-steel transition-[transform,color] hover:text-frost focus:outline-none">
          <X className="size-4" />
          <span className="sr-only">Close</span>
        </DialogPrimitive.Close>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

function DialogTitle({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      className={cn("text-[20px] font-semibold tracking-tight text-pure-white", className)}
      {...props}
    />
  );
}

function DialogDescription({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Description>) {
  return (
    <DialogPrimitive.Description
      className={cn("text-[14px] leading-relaxed text-fog", className)}
      {...props}
    />
  );
}

export {
  Dialog,
  DialogTrigger,
  DialogClose,
  DialogContent,
  DialogModalContent,
  DialogTitle,
  DialogDescription,
};
