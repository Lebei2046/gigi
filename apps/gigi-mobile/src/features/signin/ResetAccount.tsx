import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from "@/components/ui/button"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { useAppDispatch } from '@/store';
import { resetAuth } from '@/store/authSlice';

export default function ResetAccount() {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const [checked, setChecked] = useState(false);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Reset Account</CardTitle>
        <CardDescription>
          You are about to reset your account.
        </CardDescription>
        <CardAction>
          <Button variant="link" onClick={() => navigate(-1)}>Cancel</Button>
        </CardAction>
      </CardHeader>

      <CardContent>
        <Alert variant="default">
          <AlertTitle>Warning</AlertTitle>
          <AlertDescription>
            Resetting your account will permanently delete all your data, including your wallet and transaction history.
            This is a destructive action, Please ensure you have backed up your recovery phrase before proceeding.
          </AlertDescription>
        </Alert>
      </CardContent>
      <CardFooter className="flex flex-col">
        <div className="mb-2">
          <input
            type="checkbox"
            id="accept-risk"
            checked={checked}
            onChange={() => setChecked(!checked)}
            className="mr-2"
          />
          <label htmlFor="accept-risk">I understand and accept the risks</label>
        </div>
        <div>
          <Button
            color="danger"
            disabled={!checked}
            onClick={() => {
              dispatch(resetAuth());
              navigate('/signup');
            }}
          >
            Reset Account
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
}