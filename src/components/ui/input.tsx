import * as React from "react";
import { cn } from "../../lib/utils";

export const Input = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, ...props }, ref) => (
  <input
    className={cn(
      "h-9 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-950 outline-none transition-colors placeholder:text-slate-400 focus:border-teal-700 focus:ring-2 focus:ring-teal-700/15 disabled:cursor-not-allowed disabled:bg-slate-100 disabled:text-slate-500",
      className,
    )}
    ref={ref}
    {...props}
  />
));

Input.displayName = "Input";
