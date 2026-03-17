import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import { useEffect, useState } from "react"
import { z } from "zod"
import { toast } from "sonner"
import { createServerFn } from "@tanstack/react-start"
import { getRequestHeaders } from "@tanstack/react-start/server"
import { auth } from "@/lib/auth"
import { MailCheck, AlertTriangle, Loader2 } from "lucide-react"
import { Button } from "@/components/ui/button"

const verifyEmailFn = createServerFn({ method: "POST" })
  .inputValidator(z.object({ token: z.string().min(1) }))
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.verifyEmail({ query: { token: data.token }, headers })
  })

const resendVerificationFn = createServerFn({ method: "POST" })
  .inputValidator(z.object({ email: z.string().email() }))
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.sendVerificationEmail({
      body: { email: data.email, callbackURL: "/auth/verify-email" },
      headers,
    })
  })

export const Route = createFileRoute("/auth/verify-email")({
  validateSearch: z.object({ token: z.string().optional() }),
  head: () => ({ meta: [{ title: "Verify email — tsx registry" }] }),
  component: VerifyEmailPage,
})

function VerifyEmailPage() {
  const { token } = Route.useSearch()
  const navigate = useNavigate()
  const [status, setStatus] = useState<"idle" | "verifying" | "success" | "error">(
    token ? "verifying" : "idle"
  )
  const [resendEmail, setResendEmail] = useState("")
  const [resending, setResending] = useState(false)

  useEffect(() => {
    if (!token) return
    setStatus("verifying")
    verifyEmailFn({ data: { token } })
      .then(() => {
        setStatus("success")
        toast.success("Email verified! You can now publish packages.")
        setTimeout(() => navigate({ to: "/dashboard" }), 2500)
      })
      .catch(() => {
        setStatus("error")
        toast.error("Verification failed — the link may have expired.")
      })
  }, [token])

  async function handleResend(e: React.FormEvent) {
    e.preventDefault()
    if (!resendEmail) return
    setResending(true)
    try {
      await resendVerificationFn({ data: { email: resendEmail } })
      toast.success("Verification email sent — check your inbox.")
    } catch {
      toast.error("Failed to resend verification email.")
    } finally {
      setResending(false)
    }
  }

  if (status === "verifying") {
    return (
      <div className="mx-auto max-w-md py-20 rise-in text-center">
        <Loader2 className="mx-auto mb-4 size-10 animate-spin" style={{ color: "var(--lagoon)" }} />
        <p className="font-semibold" style={{ color: "var(--sea-ink)" }}>Verifying your email…</p>
      </div>
    )
  }

  if (status === "success") {
    return (
      <div className="mx-auto max-w-md py-20 rise-in text-center">
        <MailCheck className="mx-auto mb-4 size-12" style={{ color: "var(--lagoon)" }} />
        <h1 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Email verified!</h1>
        <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          Redirecting you to the dashboard…
        </p>
      </div>
    )
  }

  if (status === "error") {
    return (
      <div className="mx-auto max-w-md py-20 rise-in text-center">
        <AlertTriangle className="mx-auto mb-4 size-10" style={{ color: "#f59e0b" }} />
        <h1 className="mb-2 text-xl font-bold" style={{ color: "var(--sea-ink)" }}>Link expired or invalid</h1>
        <p className="mb-6 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          Verification links expire after 24 hours. Request a new one below.
        </p>
        <form onSubmit={handleResend} className="flex gap-2">
          <input
            type="email"
            value={resendEmail}
            onChange={(e) => setResendEmail(e.target.value)}
            placeholder="your@email.com"
            className="flex-1 rounded-lg border px-3 py-2 text-sm"
            style={{ borderColor: "var(--line)", background: "var(--code-bg)", color: "var(--sea-ink)" }}
          />
          <Button type="submit" disabled={resending} size="sm">
            {resending ? "Sending…" : "Resend"}
          </Button>
        </form>
      </div>
    )
  }

  // idle — no token, just info page
  return (
    <div className="mx-auto max-w-md py-16 rise-in">
      <div className="island-shell rounded-2xl p-8 text-center">
        <MailCheck className="mx-auto mb-4 size-12" style={{ color: "var(--lagoon)" }} />
        <h1 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Check your inbox</h1>
        <p className="mb-6 text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>
          We sent a verification link to your email address. Click it to activate your account.
          Publishing is disabled until your email is verified.
        </p>

        <p className="mb-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>Didn't receive it? Resend below:</p>

        <form onSubmit={handleResend} className="space-y-3">
          <input
            type="email"
            value={resendEmail}
            onChange={(e) => setResendEmail(e.target.value)}
            placeholder="your@email.com"
            className="w-full rounded-lg border px-3 py-2 text-sm"
            style={{ borderColor: "var(--line)", background: "var(--code-bg)", color: "var(--sea-ink)" }}
          />
          <Button type="submit" disabled={resending} className="w-full">
            {resending ? "Sending…" : "Resend verification email"}
          </Button>
        </form>

        <p className="mt-6 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
          <Link to="/auth/login" className="hover:underline" style={{ color: "var(--lagoon-deep)" }}>
            Back to sign in
          </Link>
        </p>
      </div>
    </div>
  )
}
