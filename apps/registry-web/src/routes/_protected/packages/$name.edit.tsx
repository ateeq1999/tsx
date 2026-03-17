import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { toast } from "sonner"
import { requireAuth } from "@/middleware/auth-guard"
import { packageQueryOptions, packageVersionsQueryOptions, usePackage } from "@/features/packages/hooks/use-packages"
import { useQuery } from "@tanstack/react-query"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ArrowLeft, AlertTriangle } from "lucide-react"

export const Route = createFileRoute("/_protected/packages/$name/edit")({
  beforeLoad: async () => requireAuth(),
  loader: async ({ context: { queryClient }, params: { name } }) => {
    await queryClient.prefetchQuery(packageQueryOptions(name))
    queryClient.prefetchQuery(packageVersionsQueryOptions(name))
  },
  head: ({ params: { name } }) => ({ meta: [{ title: `Edit ${name} — tsx registry` }] }),
  component: PackageEditPage,
})

const REGISTRY_URL = import.meta.env.VITE_REGISTRY_URL ?? "http://localhost:8080"

function PackageEditPage() {
  const { name } = Route.useParams()
  const { user } = Route.useRouteContext()
  const { data: pkg } = usePackage(name)
  const { data: versions } = useQuery(packageVersionsQueryOptions(name))
  const navigate = useNavigate()

  const [description, setDescription] = useState(pkg?.description ?? "")
  const [readme, setReadme] = useState("")
  const [saving, setSaving] = useState(false)
  const [readmeLoading, setReadmeLoading] = useState(false)

  if (!pkg) return null

  // Guard: only the package author can edit
  if (pkg.author !== user.name) {
    return (
      <div className="page-wrap py-16 rise-in text-center">
        <AlertTriangle className="mx-auto mb-3 size-8" style={{ color: "#f59e0b" }} />
        <p className="font-semibold" style={{ color: "var(--sea-ink)" }}>Not authorised</p>
        <p className="mt-1 text-sm" style={{ color: "var(--sea-ink-soft)" }}>Only the package author can edit this package.</p>
      </div>
    )
  }

  async function saveDescription(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      // PUT /v1/packages/:name — not yet implemented in backend; stub with toast
      toast.info("Description update sent — backend PUT /v1/packages/:name not yet implemented")
    } finally {
      setSaving(false)
    }
  }

  async function saveReadme(e: React.FormEvent) {
    e.preventDefault()
    setReadmeLoading(true)
    try {
      const res = await fetch(`${REGISTRY_URL}/v1/packages/${encodeURIComponent(name)}/readme`, {
        method: "PUT",
        headers: { "Content-Type": "text/markdown" },
        body: readme,
      })
      if (!res.ok) throw new Error(await res.text())
      toast.success("README updated")
    } catch (err) {
      toast.error(`Failed: ${err instanceof Error ? err.message : "Unknown error"}`)
    } finally {
      setReadmeLoading(false)
    }
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-6 flex items-center gap-3">
        <Link
          to="/packages"
          className="flex items-center gap-1 text-sm hover:underline"
          style={{ color: "var(--sea-ink-soft)" }}
        >
          <ArrowLeft className="size-4" /> My packages
        </Link>
      </div>

      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="font-mono text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>{name}</h1>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>v{pkg.version}</p>
        </div>
        <Link
          to="/packages/$name"
          params={{ name }}
          className="text-sm hover:underline"
          style={{ color: "var(--lagoon-deep)" }}
        >
          View public page →
        </Link>
      </div>

      <div className="max-w-2xl space-y-6">
        {/* Description */}
        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-4 font-semibold" style={{ color: "var(--sea-ink)" }}>Description</h2>
          <form onSubmit={saveDescription} className="space-y-4">
            <div>
              <Label htmlFor="desc" className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Package description</Label>
              <Input
                id="desc"
                className="mt-1"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Short description shown in search results"
              />
            </div>
            <Button size="sm" type="submit" disabled={saving}>
              {saving ? "Saving…" : "Save description"}
            </Button>
          </form>
        </div>

        {/* README */}
        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-1 font-semibold" style={{ color: "var(--sea-ink)" }}>README</h2>
          <p className="mb-4 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
            Markdown supported. Displayed on the Overview tab of the package page.
            Requires <code>PUT /v1/packages/:name/readme</code> on the backend.
          </p>
          <form onSubmit={saveReadme} className="space-y-4">
            <textarea
              className="w-full rounded-lg border px-3 py-2 font-mono text-sm"
              style={{ borderColor: "var(--line)", background: "var(--code-bg)", color: "var(--sea-ink)", minHeight: "220px", resize: "vertical" }}
              placeholder={"# My package\n\nDescribe what your package does…"}
              value={readme}
              onChange={(e) => setReadme(e.target.value)}
            />
            <Button size="sm" type="submit" disabled={readmeLoading} variant="outline">
              {readmeLoading ? "Saving…" : "Save README"}
            </Button>
          </form>
        </div>

        {/* Versions */}
        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-4 font-semibold" style={{ color: "var(--sea-ink)" }}>Versions</h2>
          {versions && versions.length > 0 ? (
            <div className="space-y-2">
              {versions.map((v) => (
                <div
                  key={v.version}
                  className="flex items-center justify-between text-sm"
                  style={{ borderBottom: "1px solid var(--line)", paddingBottom: "8px" }}
                >
                  <div>
                    <span className="font-mono font-bold" style={{ color: "var(--sea-ink)" }}>v{v.version}</span>
                    <span className="ml-3 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                      {v.download_count.toLocaleString()} downloads · {new Date(v.published_at).toLocaleDateString()}
                    </span>
                  </div>
                  <button
                    onClick={() => toast.info(`Yank v${v.version} — requires backend endpoint`)}
                    className="text-xs hover:underline"
                    style={{ color: "#f59e0b" }}
                  >
                    Yank
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>No versions found.</p>
          )}
        </div>
      </div>
    </div>
  )
}
