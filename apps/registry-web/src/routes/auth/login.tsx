import { createFileRoute, Link } from "@tanstack/react-router"
import { LoginForm } from "@/components/auth/login-form"

export const Route = createFileRoute("/auth/login")({
  head: () => ({ meta: [{ title: "Sign in — tsx registry" }] }),
  component: LoginPage,
})

function LoginPage() {
  return (
    <div className="mx-auto max-w-md py-16 rise-in">
      <div className="island-shell rounded-2xl p-8">
        <h1 className="mb-1 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Sign in</h1>
        <p className="mb-6 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          New here?{" "}
          <Link to="/auth/register" className="hover:underline" style={{ color: "var(--lagoon-deep)" }}>
            Create an account
          </Link>
        </p>
        <LoginForm />
        <p className="mt-4 text-center text-xs" style={{ color: "var(--sea-ink-soft)" }}>
          <Link to="/auth/forgot-password" className="hover:underline" style={{ color: "var(--lagoon-deep)" }}>
            Forgot your password?
          </Link>
        </p>
      </div>
    </div>
  )
}
