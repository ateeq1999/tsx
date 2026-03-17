import { createFileRoute } from "@tanstack/react-router"
import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { toast } from "sonner"
import { requireAuth } from "@/middleware/auth-guard"
import { KeyRound, Copy, Trash2, Plus, Eye, EyeOff } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"
import { createServerFn } from "@tanstack/react-start"
import { getRequestHeaders } from "@tanstack/react-start/server"
import { auth } from "@/server/auth"

// Server functions for API token management via better-auth
const listApiKeysFn = createServerFn({ method: "GET" }).handler(async () => {
  const headers = getRequestHeaders()
  // better-auth apiKey plugin — list tokens for the current user
  return await auth.api.listApiKeys({ headers }) as Array<{
    id: string
    name: string
    start: string
    createdAt: string
    expiresAt: string | null
    lastUsedAt: string | null
  }>
})

const createApiKeyFn = createServerFn({ method: "POST" })
  .inputValidator((data: { name: string }) => data)
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.createApiKey({
      body: { name: data.name },
      headers,
    }) as { key: string; id: string; name: string }
  })

const deleteApiKeyFn = createServerFn({ method: "POST" })
  .inputValidator((data: { keyId: string }) => data)
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.deleteApiKey({
      body: { keyId: data.keyId },
      headers,
    })
  })

const apiKeysQueryOptions = queryOptions({
  queryKey: ["account", "api-keys"],
  queryFn: () => listApiKeysFn(),
  staleTime: 30_000,
})

export const Route = createFileRoute("/_protected/account/api-keys")({
  beforeLoad: async () => requireAuth(),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(apiKeysQueryOptions),
  head: () => ({ meta: [{ title: "API Keys — tsx registry" }] }),
  component: ApiKeysPage,
})

function ApiKeysPage() {
  const { data: keys, isLoading } = useQuery(apiKeysQueryOptions)
  const queryClient = useQueryClient()

  const [name, setName] = useState("")
  const [creating, setCreating] = useState(false)
  const [newKey, setNewKey] = useState<string | null>(null)
  const [showKey, setShowKey] = useState(false)
  const [deleteId, setDeleteId] = useState<string | null>(null)
  const [deleting, setDeleting] = useState(false)

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault()
    if (!name.trim()) return
    setCreating(true)
    try {
      const result = await createApiKeyFn({ data: { name: name.trim() } })
      setNewKey(result.key)
      setName("")
      queryClient.invalidateQueries({ queryKey: ["account", "api-keys"] })
      toast.success("API key created")
    } catch {
      toast.error("Failed to create API key")
    } finally {
      setCreating(false)
    }
  }

  async function handleDelete() {
    if (!deleteId) return
    setDeleting(true)
    try {
      await deleteApiKeyFn({ data: { keyId: deleteId } })
      queryClient.invalidateQueries({ queryKey: ["account", "api-keys"] })
      toast.success("API key deleted")
      setDeleteId(null)
    } catch {
      toast.error("Failed to delete API key")
    } finally {
      setDeleting(false)
    }
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-2 flex items-center gap-2">
        <KeyRound className="size-5" style={{ color: "var(--lagoon)" }} />
        <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>API Keys</h1>
      </div>
      <p className="mb-8 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
        Generate bearer tokens for CLI publishing. Keep your keys secret — they have the same permissions as your account.
      </p>

      <div className="max-w-lg space-y-6">
        {/* New key revealed banner */}
        {newKey && (
          <div
            className="rounded-xl border p-4"
            style={{ borderColor: "var(--lagoon)", background: "var(--code-bg)" }}
          >
            <p className="mb-2 text-sm font-semibold" style={{ color: "var(--lagoon-deep)" }}>
              Copy your key now — it won't be shown again.
            </p>
            <div className="flex items-center gap-2">
              <code
                className="flex-1 truncate rounded px-2 py-1 text-xs"
                style={{ background: "var(--sea-ink)", color: "#d7ece8" }}
              >
                {showKey ? newKey : newKey.slice(0, 8) + "•".repeat(32)}
              </code>
              <button
                onClick={() => setShowKey((v) => !v)}
                className="shrink-0"
                style={{ color: "var(--sea-ink-soft)" }}
                title={showKey ? "Hide" : "Show"}
              >
                {showKey ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
              </button>
              <button
                onClick={() => { navigator.clipboard.writeText(newKey); toast.success("Copied!") }}
                className="shrink-0"
                style={{ color: "var(--lagoon)" }}
                title="Copy"
              >
                <Copy className="size-4" />
              </button>
            </div>
          </div>
        )}

        {/* Create form */}
        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-4 font-semibold" style={{ color: "var(--sea-ink)" }}>Create new key</h2>
          <form onSubmit={handleCreate} className="flex gap-3">
            <div className="flex-1">
              <Label htmlFor="key-name" className="sr-only">Key name</Label>
              <Input
                id="key-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. CI / local dev"
              />
            </div>
            <Button type="submit" disabled={creating || !name.trim()} size="sm">
              <Plus className="mr-1 size-4" />
              {creating ? "Creating…" : "Create"}
            </Button>
          </form>
          <p className="mt-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
            Use this token with: <code className="font-mono">tsx registry publish --api-key &lt;token&gt;</code>
          </p>
        </div>

        {/* Keys list */}
        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-4 font-semibold" style={{ color: "var(--sea-ink)" }}>Your keys</h2>

          {isLoading ? (
            <div className="space-y-3">
              {Array.from({ length: 2 }).map((_, i) => (
                <div key={i} className="h-12 animate-pulse rounded-lg" style={{ background: "var(--line)" }} />
              ))}
            </div>
          ) : keys && keys.length > 0 ? (
            <div className="space-y-2">
              {keys.map((k) => (
                <div
                  key={k.id}
                  className="flex items-center justify-between gap-4 rounded-lg px-3 py-2.5 text-sm"
                  style={{ border: "1px solid var(--line)" }}
                >
                  <div>
                    <p className="font-medium" style={{ color: "var(--sea-ink)" }}>{k.name}</p>
                    <p className="mt-0.5 text-xs font-mono" style={{ color: "var(--sea-ink-soft)" }}>
                      {k.start}••••••••
                      {k.lastUsedAt
                        ? ` · Last used ${new Date(k.lastUsedAt).toLocaleDateString()}`
                        : " · Never used"}
                    </p>
                  </div>
                  <button
                    onClick={() => setDeleteId(k.id)}
                    className="shrink-0 hover:opacity-70"
                    style={{ color: "#ef4444" }}
                    title="Delete key"
                  >
                    <Trash2 className="size-4" />
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>No API keys yet.</p>
          )}
        </div>
      </div>

      {/* Delete confirm dialog */}
      <Dialog open={!!deleteId} onOpenChange={(o) => { if (!o) setDeleteId(null) }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete API key?</DialogTitle>
            <DialogDescription>
              This key will stop working immediately. Any CI jobs or scripts using it will need to be updated.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteId(null)}>Cancel</Button>
            <Button variant="destructive" disabled={deleting} onClick={handleDelete}>
              {deleting ? "Deleting…" : "Delete key"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
