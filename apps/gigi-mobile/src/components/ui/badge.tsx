import * as React from 'react'

import { cn } from '@/lib/utils'

function Badge({ className, ...props }: React.ComponentProps<'span'>) {
  return (
    <span
      data-slot="badge"
      className={cn(
        'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
        'bg-primary text-primary-foreground hover:bg-primary/90',
        className
      )}
      {...props}
    />
  )
}

export { Badge }