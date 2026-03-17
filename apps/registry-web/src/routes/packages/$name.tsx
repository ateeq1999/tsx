import { createFileRoute, notFound, Link } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { marked } from "marked"
import { Check, Clock, Copy, Download, Tag, User } from "lucide-react"
import { useState, useEffect, useRef } from "react"
import hljs from "highlight.js/lib/core"
import hljsTs from "highlight.js/lib/languages/typescript"
import hljsJs from "highlight.js/lib/languages/javascript"
import hljsBash from "highlight.js/lib/languages/bash"
import hljsJson from "highlight.js/lib/languages/json"
import hljsRust from "highlight.js/lib/languages/rust"
import "highlight.js/styles/github.css"

hljs.registerLanguage("typescript", hljsTs)
hljs.registerLanguage("javascript", hljsJs)
hljs.registerLanguage("bash", hljsBash)
hljs.registerLanguage("json", hljsJson)
hljs.registerLanguage("rust", hljsRust)
import {
  packageQueryOptions,
  packageVersionsQueryOptions,
  packageReadmeQueryOptions,
  usePackage,
} from "@/features/packages/hooks/use-packages"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"

export const Route = createFileRoute("/packages/$name")({
  loader: async ({ context: { queryClient }, params: { name } }) => {
    try {
      await queryClient.ensureQueryData(packageQueryOptions(name))
    } catch {
      throw notFound()
    }
    queryClient.prefetchQuery(packageVersionsQueryOptions(name))
    queryClient.prefetchQuery(packageReadmeQueryOptions(name))
  },
  head: ({ params: { name } }) => ({
    meta: [
      { title: `${name} — tsx registry` },
      { name: "description", content: `Install ${name} with tsx. Browse package details, versions, and documentation.` },
      { property: "og:title", content: `${name} — tsx registry` },
      { property: "og:description", content: `Install ${name} with one command: tsx install ${name}` },
    ],
  }),
  notFoundComponent: () => (
    <div className="page-wrap py-24 text-center" style={{ color: "var(--sea-ink-soft)" }}>
      Package not found.
    </div>
  ),
  component: PackageDetailPage,
})

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  function copy() {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }
  return (
    <button
      onClick={copy}
      className="flex items-center gap-1 text-xs opacity-60 hover:opacity-100 transition-opacity"
      style={{ color: "var(--lagoon-deep)" }}
    >
      {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
      {copied ? "copied" : "copy"}
    </button>
  )
}

function PackageDetailPage() {
  const { name } = Route.useParams()
  const { data: pkg } = usePackage(name)
  const { data: versions } = useQuery(packageVersionsQueryOptions(name))
  const { data: readme } = useQuery(packageReadmeQueryOptions(name))
  const readmeRef = useRef<HTMLDivElement>(null)

  const readmeHtml = readme ? marked.parse(readme) as string : null

  useEffect(() => {
    const el = readmeRef.current
    if (!el || !readmeHtml) return
    el.querySelectorAll("pre code").forEach((block) => {
      if (block.getAttribute("data-highlighted")) return
      hljs.highlightElement(block as HTMLElement)
    })
  }, [readmeHtml])

  if (!pkg) return null

  return (
    <div className="page-wrap py-12 rise-in">
      {/* Header */}
      <div className="mb-8">
        <div className="mb-2 flex items-center gap-3">
          <h1 className="font-mono text-3xl font-bold" style={{ color: "var(--sea-ink)" }}>
            {pkg.name}
          </h1>
          <Badge variant="secondary">v{pkg.version}</Badge>
        </div>
        <p className="mb-4 text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>
          {pkg.description}
        </p>
        <div className="flex flex-wrap gap-1">
          {pkg.tags.map((tag) => (
            <span key={tag} className="pkg-tag">{tag}</span>
          ))}
        </div>
      </div>

      {/* Install command */}
      <div className="island-shell mb-8 rounded-xl p-4">
        <p className="island-kicker mb-2">Install</p>
        <div className="flex items-center gap-3 rounded-lg bg-black/5 px-4 py-2 dark:bg-white/5">
          <span style={{ color: "var(--lagoon)" }} className="font-bold">$</span>
          <code className="flex-1 text-sm" style={{ color: "var(--sea-ink)" }}>
            tsx install {pkg.name}
          </code>
          <CopyButton text={`tsx install ${pkg.name}`} />
        </div>
      </div>

      {/* Tabs */}
      <div className="grid gap-6 lg:grid-cols-[1fr_280px]">
        <Tabs defaultValue={readmeHtml ? "overview" : "versions"}>
          <TabsList className="mb-4">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="versions">Versions</TabsTrigger>
          </TabsList>

          <TabsContent value="overview">
            {readmeHtml ? (
              <div
                ref={readmeRef}
                className="island-shell rounded-xl p-6 doc-content"
                dangerouslySetInnerHTML={{ __html: readmeHtml }}
              />
            ) : (
              <div
                className="island-shell rounded-xl p-8 text-center text-sm"
                style={{ color: "var(--sea-ink-soft)" }}
              >
                No README available for this package.
              </div>
            )}
          </TabsContent>

          <TabsContent value="versions">
            <div className="island-shell rounded-xl p-6">
              <h2 className="mb-4 font-bold" style={{ color: "var(--sea-ink)" }}>Versions</h2>
              {versions ? (
                <div className="space-y-2">
                  {versions.map((v) => (
                    <div
                      key={v.version}
                      className="flex items-center justify-between text-sm"
                      style={{ borderBottom: "1px solid var(--line)", paddingBottom: "8px" }}
                    >
                      <span className="font-mono font-bold" style={{ color: "var(--sea-ink)" }}>
                        v{v.version}
                      </span>
                      <div className="flex items-center gap-4" style={{ color: "var(--sea-ink-soft)" }}>
                        <span>{v.download_count.toLocaleString()} downloads</span>
                        <span>{new Date(v.published_at).toLocaleDateString()}</span>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="space-y-2">
                  {Array.from({ length: 4 }).map((_, i) => (
                    <div key={i} className="h-8 animate-pulse rounded" style={{ background: "var(--line)" }} />
                  ))}
                </div>
              )}
            </div>
          </TabsContent>
        </Tabs>

        {/* Meta sidebar */}
        <div className="space-y-4">
          <div className="island-shell rounded-xl p-4">
            <h2 className="island-kicker mb-4">Details</h2>
            <div className="space-y-3 text-sm">
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <User className="size-4" />
                <span>{pkg.author}</span>
              </div>
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <Tag className="size-4" />
                <span>{pkg.license}</span>
              </div>
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <Download className="size-4" />
                <span>{pkg.download_count.toLocaleString()} installs</span>
              </div>
              <div className="flex items-center gap-2" style={{ color: "var(--sea-ink-soft)" }}>
                <Clock className="size-4" />
                <span>Updated {new Date(pkg.updated_at).toLocaleDateString()}</span>
              </div>
            </div>
          </div>

          <div className="island-shell rounded-xl p-4">
            <h2 className="island-kicker mb-2">Requires tsx</h2>
            <p className="font-mono text-sm" style={{ color: "var(--sea-ink)" }}>&gt;= {pkg.tsx_min}</p>
          </div>

          {pkg.provides && pkg.provides.length > 0 && (
            <div className="island-shell rounded-xl p-4">
              <h2 className="island-kicker mb-3">Provides</h2>
              <div className="flex flex-wrap gap-1.5">
                {pkg.provides.map((cap) => (
                  <Link
                    key={cap}
                    to="/browse"
                    search={{ q: cap, page: 1, lang: "", sort: "relevant" }}
                    className="rounded-full px-2.5 py-0.5 text-xs font-medium hover:no-underline"
                    style={{ background: "var(--lagoon)", color: "#fff" }}
                  >
                    {cap}
                  </Link>
                ))}
              </div>
            </div>
          )}

          {pkg.integrates_with && pkg.integrates_with.length > 0 && (
            <div className="island-shell rounded-xl p-4">
              <h2 className="island-kicker mb-3">Integrates with</h2>
              <div className="flex flex-wrap gap-1.5">
                {pkg.integrates_with.map((dep) => (
                  <Link
                    key={dep}
                    to="/packages/$name"
                    params={{ name: dep }}
                    className="rounded-full px-2.5 py-0.5 text-xs font-medium hover:no-underline"
                    style={{ background: "var(--line)", color: "var(--sea-ink)" }}
                  >
                    {dep}
                  </Link>
                ))}
              </div>
            </div>
          )}

          {pkg.lang && (
            <div className="island-shell rounded-xl p-4">
              <h2 className="island-kicker mb-2">Language</h2>
              <p className="text-sm capitalize" style={{ color: "var(--sea-ink)" }}>{pkg.lang}</p>
              {pkg.runtime && (
                <p className="mt-1 text-xs capitalize" style={{ color: "var(--sea-ink-soft)" }}>Runtime: {pkg.runtime}</p>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
