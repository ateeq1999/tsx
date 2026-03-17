import { createFileRoute, Link } from "@tanstack/react-router"
import { useState } from "react"
import { toast } from "sonner"
import { forgotPasswordFn } from "@/server/auth/mutations"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export const Route = createFileRoute("/auth/forgot-password")({
  head: () => ({ meta: [{ title: "Reset password — tsx registry" }] }),
  component: ForgotPasswordPage,
})

function ForgotPasswordPage() {
  const [email, setEmail] = useState("")
  const [sent, setSent] = useState(false)
  const [loading, setLoading] = useState(false)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    try {
      await forgotPasswordFn({ data: { email } })
      setSent(true)
    } catch {
      toast.error("Failed to send reset email. Check the address and try again.")
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="mx-auto max-w-md py-16 rise-in">
      <div className="island-shell rounded-2xl p-8">
        <h1 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Reset password</h1>

        {sent ? (
          <div>
            <p className="mb-4 text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>
              If <strong>{email}</strong> has an account, a reset link has been sent. Check your inbox.
            </p>
            <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>
              Didn't receive it? Check spam, or{" "}
              <button className="hover:underline" style={{ color: "var(--lagoon-deep)" }} onClick={() => setSent(false)}>
                try again
              </button>.
            </p>
          </div>
        ) : (
          <>
            <p className="mb-6 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
              Enter your email address and we'll send you a link to reset your password.
            </p>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <Label htmlFor="email" className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Email</Label>
                <Input
                  id="email"
                  type="email"
                  required
                  className="mt-1"
                  placeholder="you@example.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
              </div>
              <Button type="submit" disabled={loading} className="w-full">
                {loading ? "Sending…" : "Send reset link"}
              </Button>
            </form>
          </>
        )}

        <p className="mt-6 text-center text-xs" style={{ color: "var(--sea-ink-soft)" }}>
          Remember it?{" "}
          <Link to="/auth/login" className="hover:underline" style={{ color: "var(--lagoon-deep)" }}>Sign in</Link>
        </p>
      </div>
    </div>
  )
}
