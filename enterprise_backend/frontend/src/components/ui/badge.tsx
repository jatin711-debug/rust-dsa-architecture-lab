import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const badgeVariants = cva(
  'inline-flex items-center gap-1 font-mono text-[10px] font-semibold uppercase tracking-wider px-2.5 py-0.5 rounded-full transition-all border',
  {
    variants: {
      variant: {
        default:
          'bg-indigo-500/10 text-indigo-400 border-indigo-500/20',
        emerald:
          'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        cyan:
          'bg-cyan-500/10 text-cyan-400 border-cyan-500/20',
        amber:
          'bg-amber-500/10 text-amber-300 border-amber-500/20',
        rose:
          'bg-rose-500/10 text-rose-400 border-rose-500/20',
        purple:
          'bg-purple-500/10 text-purple-400 border-purple-500/20',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
