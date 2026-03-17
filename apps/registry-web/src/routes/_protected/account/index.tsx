import { createFileRoute, Link, useRouter, useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { useForm } from "@tanstack/react-form"
import { toast } from "sonner"
import { User, Lock, KeyRound, Trash2 } from "lucide-react"
import { requireAuth } from "@/middleware/auth-guard"
import { updateProfileFn, changePasswordFn, deleteAccountFn } from "@/server/auth/mutations"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"

export const Route = createFileRoute("/_protected/account/")({
  beforeLoad: async () => requireAuth(),
  head: () => ({ meta: [{ title: "Account — tsx registry" }] }),
  component: AccountPage,
})

function AccountPage() {
  const { user } = Route.useRouteContext()
  const router = useRouter()
  const navigate = useNavigate()
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [deleting, setDeleting] = useState(false)

  const profileForm = useForm({
    defaultValues: { name: user.name ?? "" },
    onSubmit: async ({ value }) => {
      try {
        await updateProfileFn({ data: value })
        toast.success("Profile updated")
        router.invalidate()
      } catch {
        toast.error("Failed to update profile")
      }
    },
  })

  const passwordForm = useForm({
    defaultValues: { currentPassword: "", newPassword: "", confirmPassword: "" },
    onSubmit: async ({ value }) => {
      if (value.newPassword !== value.confirmPassword) {
        toast.error("Passwords do not match")
        return
      }
      try {
        await changePasswordFn({
          data: { currentPassword: value.currentPassword, newPassword: value.newPassword },
        })
        toast.success("Password changed")
        passwordForm.reset()
      } catch {
        toast.error("Current password is incorrect")
      }
    },
  })

  return (
    <div className="page-wrap py-12 rise-in">
      <h1 className="mb-8 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Account Settings</h1>

      <div className="max-w-lg space-y-6">
        {/* Profile */}
        <div className="island-shell rounded-xl p-6">
          <div className="mb-4 flex items-center gap-2">
            <User className="size-4" style={{ color: "var(--lagoon)" }} />
            <h2 className="font-semibold" style={{ color: "var(--sea-ink)" }}>Profile</h2>
          </div>

          <div className="mb-4">
            <Label className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Email</Label>
            <p className="mt-1 text-sm font-medium" style={{ color: "var(--sea-ink)" }}>{user.email}</p>
            <p className="mt-0.5 text-xs" style={{ color: "var(--sea-ink-soft)" }}>Email cannot be changed here.</p>
          </div>

          <form
            onSubmit={(e) => { e.preventDefault(); profileForm.handleSubmit() }}
            className="space-y-4"
          >
            <div>
              <Label htmlFor="name" className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Display name</Label>
              <profileForm.Field name="name">
                {(field) => (
                  <Input
                    id="name"
                    className="mt-1"
                    value={field.state.value}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Your name"
                  />
                )}
              </profileForm.Field>
            </div>
            <Button type="submit" disabled={profileForm.state.isSubmitting} size="sm">
              {profileForm.state.isSubmitting ? "Saving…" : "Save name"}
            </Button>
          </form>
        </div>

        {/* Password */}
        <div className="island-shell rounded-xl p-6">
          <div className="mb-4 flex items-center gap-2">
            <Lock className="size-4" style={{ color: "var(--lagoon)" }} />
            <h2 className="font-semibold" style={{ color: "var(--sea-ink)" }}>Change Password</h2>
          </div>

          <form
            onSubmit={(e) => { e.preventDefault(); passwordForm.handleSubmit() }}
            className="space-y-4"
          >
            {(["currentPassword", "newPassword", "confirmPassword"] as const).map((name) => (
              <div key={name}>
                <Label htmlFor={name} className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                  {name === "currentPassword" ? "Current password" : name === "newPassword" ? "New password" : "Confirm new password"}
                </Label>
                <passwordForm.Field name={name}>
                  {(field) => (
                    <Input
                      id={name}
                      type="password"
                      className="mt-1"
                      value={field.state.value}
                      onChange={(e) => field.handleChange(e.target.value)}
                      placeholder="••••••••"
                    />
                  )}
                </passwordForm.Field>
              </div>
            ))}
            <Button type="submit" disabled={passwordForm.state.isSubmitting} size="sm" variant="outline">
              {passwordForm.state.isSubmitting ? "Changing…" : "Change password"}
            </Button>
          </form>
        </div>

        {/* Sessions link */}
        <div className="island-shell rounded-xl p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-semibold text-sm" style={{ color: "var(--sea-ink)" }}>Active sessions</p>
              <p className="text-xs mt-0.5" style={{ color: "var(--sea-ink-soft)" }}>View and revoke devices signed in to your account.</p>
            </div>
            <Button asChild size="sm" variant="outline">
              <Link to="/account/sessions">Manage</Link>
            </Button>
          </div>
        </div>

        {/* API key link */}
        <div className="island-shell rounded-xl p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-semibold text-sm" style={{ color: "var(--sea-ink)" }}>API keys</p>
              <p className="text-xs mt-0.5" style={{ color: "var(--sea-ink-soft)" }}>Generate tokens for CLI publishing.</p>
            </div>
            <Button asChild size="sm" variant="outline">
              <Link to="/account/api-keys">Manage</Link>
            </Button>
          </div>
        </div>

        {/* API Key hint */}
        <div className="island-shell rounded-xl p-6">
          <div className="mb-3 flex items-center gap-2">
            <KeyRound className="size-4" style={{ color: "var(--lagoon)" }} />
            <h2 className="font-semibold" style={{ color: "var(--sea-ink)" }}>CLI Publishing</h2>
          </div>
          <p className="mb-3 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            To publish packages from the CLI, use the web publish flow or pass your session token via the API key flag:
          </p>
          <div className="rounded-lg bg-black/5 px-4 py-2 dark:bg-white/5">
            <code className="text-xs" style={{ color: "var(--sea-ink)" }}>
              tsx framework publish --registry https://registry.tsx.dev --api-key &lt;token&gt;
            </code>
          </div>
          <p className="mt-2 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
            Full API key management (generate / revoke tokens) is coming soon.
          </p>
        </div>
        {/* Danger zone */}
        <div className="rounded-xl border p-6" style={{ borderColor: "#fca5a5" }}>
          <div className="mb-3 flex items-center gap-2">
            <Trash2 className="size-4" style={{ color: "#ef4444" }} />
            <h2 className="font-semibold" style={{ color: "#ef4444" }}>Danger zone</h2>
          </div>
          <p className="mb-4 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Permanently delete your account and all associated data. This cannot be undone.
          </p>
          <Button variant="destructive" size="sm" onClick={() => setDeleteOpen(true)}>
            Delete account
          </Button>
        </div>
      </div>

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete account?</DialogTitle>
            <DialogDescription>
              This will permanently delete <strong>{user.email}</strong> and all your data.
              This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>Cancel</Button>
            <Button
              variant="destructive"
              disabled={deleting}
              onClick={async () => {
                setDeleting(true)
                try {
                  await deleteAccountFn()
                  toast.success("Account deleted")
                  navigate({ to: "/" })
                } catch {
                  toast.error("Failed to delete account")
                  setDeleting(false)
                }
              }}
            >
              {deleting ? "Deleting…" : "Yes, delete my account"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
