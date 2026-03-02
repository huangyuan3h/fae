import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default:
          "bg-sky-500 text-slate-900 shadow-[0_8px_24px_-12px_rgba(56,189,248,0.85)] hover:bg-sky-400 focus-visible:ring-sky-300",
        secondary:
          "bg-slate-800 text-slate-100 hover:bg-slate-700 focus-visible:ring-slate-400",
        outline:
          "border border-slate-700 bg-slate-900/60 text-slate-100 hover:border-slate-500 hover:bg-slate-800/70 focus-visible:ring-slate-500",
        ghost:
          "text-slate-200 hover:bg-slate-800/70 hover:text-slate-100 focus-visible:ring-slate-500",
        destructive:
          "bg-rose-500 text-white hover:bg-rose-400 focus-visible:ring-rose-300"
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-lg px-3",
        lg: "h-11 rounded-xl px-8",
        icon: "h-10 w-10"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "default"
    }
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size }), className)}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };
