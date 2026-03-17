import { createFileRoute, Link } from "@tanstack/react-router"
import { RegisterForm } from "@/components/auth/register-form"

export const Route = createFileRoute("/auth/register")({
  head: () => ({ meta: [{ title: "Create account — tsx registry" }] }),
  component: RegisterPage,
})

function RegisterPage() {
  return (
    <div className="mx-auto max-w-md py-16 rise-in">
      <div className="island-shell rounded-2xl p-8">
        <h1 className="mb-1 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Create account</h1>
        <p className="mb-6 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          Already have one?{" "}
          <Link to="/auth/login" className="hover:underline" style={{ color: "var(--lagoon-deep)" }}>
            Sign in
          </Link>
        </p>
        <RegisterForm />
      </div>
    </div>
  )
}
