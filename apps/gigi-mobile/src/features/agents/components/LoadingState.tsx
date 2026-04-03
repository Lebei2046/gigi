import React from 'react'
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Loader2 } from 'lucide-react'

const LoadingState: React.FC = () => {
  return (
    <Card className="h-full flex items-center justify-center">
      <CardHeader className="text-center">
        <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-muted">
          <Loader2 className="h-6 w-6 text-muted-foreground animate-spin" />
        </div>
        <CardTitle className="mt-4">Discovering Agents</CardTitle>
        <CardDescription>
          Searching for agents on the network...
        </CardDescription>
      </CardHeader>
    </Card>
  )
}

export default LoadingState
