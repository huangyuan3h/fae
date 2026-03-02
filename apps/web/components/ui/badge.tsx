import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-1 text-xs font-semibold transition",
  {
    variants: {
      variant: {
        default: "border-sky-300/30 bg-sky-400/15 text-sky-200",
        secondary: "border-slate-600 bg-slate-800 text-slate-200",
        success: "border-emerald-300/30 bg-emerald-400/15 text-emerald-200",
        danger: "border-rose-300/30 bg-rose-400/15 text-rose-200"
      }
    },
    defaultVariants: {
      variant: "default"
    }
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { Badge, badgeVariants };
