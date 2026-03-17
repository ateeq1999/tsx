import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { z } from "zod"
import { toast } from "sonner"
import { resetPasswordFn } from "@/server/auth/mutations"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export const Route = createFileRoute("/auth/reset-password")({
  validateSearch: z.object({ token: z.string().optional() }),
  head: () => ({ meta: [{ title: "Set new password — tsx registry" }] }),
  component: ResetPasswordPage,
})

function ResetPasswordPage() {
  const { token } = Route.useSearch()
  const navigate = useNavigate()
  const [newPassword, setNewPassword] = useState("")
  const [confirm, setConfirm] = useState("")
  const [loading, setLoading] = useState(false)

  if (!token) {
    return (
      <div className="mx-auto max-w-md py-16 text-center rise-in">
        <p className="text-2xl font-bold mb-3" style={{ color: "var(--sea-ink)" }}>Invalid link</p>
        <p className="text-sm mb-6" style={{ color: "var(--sea-ink-soft)" }}>
          This reset link is invalid or has expired.
        </p>
        <Link to="/auth/forgot-password" className="text-sm hover:underline" style={{ color: "var(--lagoon-deep)" }}>
          Request a new link
        </Link>
      </div>
    )
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (newPassword !== confirm) {
      toast.error("Passwords do not match")
      return
    }
    setLoading(true)
    try {
      await resetPasswordFn({ data: { token: token!, newPassword } })
      toast.success("Password updated — please sign in")
      navigate({ to: "/auth/login" })
    } catch {
      toast.error("Reset link expired or invalid. Request a new one.")
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="mx-auto max-w-md py-16 rise-in">
      <div className="island-shell rounded-2xl p-8">
        <h1 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Set new password</h1>
        <p className="mb-6 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          Choose a strong password — at least 8 characters.
        </p>
        <form onSubmit={handleSubmit} className="space-y-4">
          {([["newPassword", "New password", newPassword, setNewPassword], ["confirm", "Confirm password", confirm, setConfirm]] as const).map(
            ([id, label, val, set]) => (
              <div key={id}>
                <Label htmlFor={id} className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>{label}</Label>
                <Input
                  id={id}
                  type="password"
                  required
                  minLength={8}
                  className="mt-1"
                  placeholder="••••••••"
                  value={val}
                  onChange={(e) => set(e.target.value)}
                />
              </div>
            )
          )}
          <Button type="submit" disabled={loading} className="w-full">
            {loading ? "Saving…" : "Update password"}
          </Button>
        </form>
      </div>
    </div>
  )
}
