import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter,
} from '@/components/ui/card'
import { ReactComponent as Terms } from '@/assets/TermsOfUse.md'
import AgreeToContinue from './AgreeToContinue'

export default function TermsOfUse() {
  const STEP: number = 0

  return (
    <Card>
      <CardHeader>
        <CardTitle>Terms of Use Agreement</CardTitle>
        <CardDescription>
          Please read the following terms and conditions carefully, check the
          box to agree and continue.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ScrollArea>
          <Terms />
        </ScrollArea>
      </CardContent>
      <CardFooter>
        <AgreeToContinue
          id="termsOfUseAgreement"
          label="I agree to the Terms of Use Agreement"
          step={STEP}
        />
      </CardFooter>
    </Card>
  )
}
