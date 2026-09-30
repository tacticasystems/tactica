import { Brand } from "../components/shared";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { useState, useSyncExternalStore } from "react";
import type { SubmitEvent } from "react";
import { Link, Navigate, useNavigate } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import { client, sessionSnapshot, subscribeSession } from "../lib/api";
import { Button } from "../components/ui/button";

export function AuthPage({ register = false }: { register?: boolean }) {
  const session = useSyncExternalStore(subscribeSession, sessionSnapshot);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const navigate = useNavigate();
  if (session) return <Navigate to="/units" replace />;
  const submit = async (event: SubmitEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const username = String(form.get("username") ?? "").trim();
    const password = String(form.get("password") ?? "");
    setError("");
    setPending(true);
    try {
      if (register)
        await client.register({
          username,
          password,
          email: String(form.get("email") ?? "").trim(),
          password_confirm: String(form.get("password_confirm") ?? ""),
        });
      await client.login(username, password);
      await navigate({ to: "/units" });
    } catch (error) {
      setError(error instanceof Error ? error.message : "Sign-in failed. Try again.");
    } finally {
      setPending(false);
    }
  };
  return (
    <main className="auth-page">
      <div className="auth-brand">
        <Brand />
      </div>
      <div className="auth-form">
        <h1>{register ? "Create your account" : "Welcome back"}</h1>
        <p>
          {register ? "Your place in the unit starts here." : "Sign in to your unit workspace."}
        </p>
        <form onSubmit={(event) => void submit(event)} aria-busy={pending}>
          <Label htmlFor="username">Username</Label>
          <Input
            id="username"
            name="username"
            autoComplete="username"
            required
            maxLength={100}
            autoFocus
          />
          {register && (
            <>
              <Label htmlFor="email">Email</Label>
              <Input id="email" name="email" type="email" autoComplete="email" required />
            </>
          )}
          <Label htmlFor="password">Password</Label>
          <Input
            id="password"
            name="password"
            type="password"
            autoComplete={register ? "new-password" : "current-password"}
            required
          />
          {register && (
            <>
              <Label htmlFor="password_confirm">Confirm password</Label>
              <Input
                id="password_confirm"
                name="password_confirm"
                type="password"
                autoComplete="new-password"
                required
              />
            </>
          )}
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
          <Button type="submit" disabled={pending}>
            {pending
              ? register
                ? "Creating account…"
                : "Signing in…"
              : register
                ? "Create account"
                : "Sign in"}
            <ArrowRight size={16} />
          </Button>
        </form>
        <p className="auth-link">
          {register ? "Already have an account?" : "New to Tactica?"}{" "}
          <Link to={register ? "/login" : "/register"}>
            {register ? "Sign in" : "Create an account"}
          </Link>
        </p>
        <div className="auth-preview">
          <span>Take a look around first.</span>
          <Link to="/units/$unitId/personnel" params={{ unitId: "preview" }}>
            Explore the 9 Rifles preview <ArrowRight size={15} />
          </Link>
        </div>
      </div>
    </main>
  );
}
