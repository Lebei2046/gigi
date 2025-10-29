import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardAction,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { login } from '@/store/authSlice';
import { useAppDispatch, useAppSelector } from '@/store';

export default function Unlock() {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { error } = useAppSelector((state: { auth: any; }) => state.auth);
  const [password, setPassword] = useState('');

  return (
    <Card>
      <CardHeader>
        <CardTitle>Unlock Account</CardTitle>
        <CardDescription>
          Enter your password to continue
        </CardDescription>
        <CardAction>
          <Button
            variant="link"
            onClick={() => navigate('/reset')}
          >
            Forgot password?
          </Button>
        </CardAction>
      </CardHeader>
      <CardFooter className="flex flex-col">
        <div className="mb-2">
          <Input
            type="password"
            placeholder="Enter your password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          {error && <p>{error}</p>}
        </div>
        <div>
          <Button
            color="primary"
            onClick={() => {
              dispatch(login({ password }));
              navigate('/');
            }}
          >
            Unlock
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
}