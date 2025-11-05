import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter
} from "@/components/ui/card";
import { useAppDispatch } from "@/store";
import { loadAuthData } from "@/store/authSlice";
import { useSignupContext } from "../context/SignupContext";

export default function SignupFinish() {
  const navigate = useNavigate();
  const appDispatch = useAppDispatch();
  const { state: { address, peerId, name }, saveAccountInfo } = useSignupContext();

  useEffect(() => {
    const saveInfo = async () => {
      await saveAccountInfo();
    };
    saveInfo();
  }, [saveAccountInfo]);

  const handleLogin = async () => {
    await appDispatch(loadAuthData());
    navigate('/login');
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Your account created successfully</CardTitle>
        <CardDescription>
          Below is your first Gigi wallet account
        </CardDescription>
      </CardHeader>
      <CardContent>
        <h2>Account Details</h2>
        <p>Account Name: {name}</p>
        <p>Wallet Address: {address}</p>
        <p>Peer Id: {peerId}</p>
      </CardContent>
      <CardFooter>
        <Button color="primary" onClick={handleLogin}>
          Go to login
        </Button>
      </CardFooter>
    </Card>
  )
}