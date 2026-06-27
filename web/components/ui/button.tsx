import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-[6px] text-[14px] font-medium transition-colors duration-150 ease-out focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-electric-blue disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        // The single chromatic signal: an electric-blue ring (never a filled fill).
        primary:
          "border border-electric-blue bg-transparent text-pure-white hover:bg-electric-blue/10",
        outline:
          "border border-graphite-rail bg-transparent text-frost hover:border-smoke hover:text-pure-white",
        ghost: "text-frost/90 hover:text-pure-white hover:bg-white/[0.06]",
        subtle: "text-fog hover:text-frost",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 px-3 text-[13px]",
        icon: "h-9 w-9",
      },
    },
    defaultVariants: { variant: "outline", size: "default" },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => (
    <button ref={ref} className={cn(buttonVariants({ variant, size, className }))} {...props} />
  ),
);
Button.displayName = "Button";

export { buttonVariants };
