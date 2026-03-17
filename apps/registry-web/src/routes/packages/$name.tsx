import { createFileRoute, notFound, Link } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { marked } from "marked"
import DOMPurify from "dompurify"
import { Check, Clock, Copy, Download, Tag, User, TrendingUp } from "lucide-react"
import { useState, useEffect, useRef } from "react"
import hljs from "highlight.js/lib/core"
import hljsTs from "highlight.js/lib/languages/typescript"
import hljsJs from "highlight.js/lib/languages/javascript"
import hljsBash from "highlight.js/lib/languages/bash"
import hljsJson from "highlight.js/lib/languages/json"
import hljsRust from "highlight.js/lib/languages/rust"
import "highlight.js/styles/github.css"
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from "recharts"

hljs.registerLanguage("typescript", hljsTs)
hljs.registerLanguage("javascript", hljsJs)
hljs.registerLanguage("bash", hljsBash)
hljs.registerLanguage("json", hljsJson)
hljs.registerLanguage("rust", hljsRust)
import {
  packageQueryOptions,
  packageVersionsQueryOptions,
  packageReadmeQueryOptions,
  packageDownloadStatsQueryOptions,
  usePackage,
} from "@/features/packages/hooks/use-packages"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { DependencyGraph } from "@/components/DependencyGraph"

export const Route = createFileRoute("/packages/$name")({
  loader: async ({ context: { queryClient }, params: { name } }) => {
    try {
      await queryClient.ensureQueryData(packageQueryOptions(name))
    } catch {
      throw notFound()
    }
    queryClient.prefetchQuery(packageVersionsQueryOptions(name))
    queryClient.prefetchQuery(packageReadmeQueryOptions(name))
    queryClient.prefetchQuery(packageDownloadStatsQueryOptions(name))
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
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => () => { if (timerRef.current) clearTimeout(timerRef.current) }, [])

  function copy() {
    navigator.clipboard.writeText(text)
    setCopied(true)
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = setTimeout(() => setCopied(false), 2000)
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
  const { data: dlStats } = useQuery(packageDownloadStatsQueryOptions(name))
  const readmeRef = useRef<HTMLDivElement>(null)

  const readmeHtml = readme ? DOMPurify.sanitize(marked.parse(readme) as string) : null
  const trendData = dlStats?.map((d) => ({
    day: new Date(d.date).toLocaleDateString("en-US", { month: "short", day: "numeric" }),
    downloads: d.downloads,
  })) ?? []

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
            <TabsTrigger value="downloads">
              <TrendingUp className="mr-1 size-3.5" />
              Downloads
            </TabsTrigger>
            {(pkg.integrates_with?.length ?? 0) > 0 && (
              <TabsTrigger value="graph">Graph</TabsTrigger>
            )}
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
              <h2 className="mb-4 font-bold" style={{ color: "var(--sea-ink)" }}>Version history</h2>
              {versions ? (
                <div className="space-y-0">
                  {versions.map((v, idx) => {
                    const isLatest = idx === 0
                    const prev = versions[idx + 1]
                    const dlDelta = prev ? v.download_count - prev.download_count : null
                    return (
                      <div
                        key={v.version}
                        className="flex items-start gap-4 py-4 text-sm"
                        style={{ borderBottom: "1px solid var(--line)" }}
                      >
                        <div className="flex flex-col items-center pt-0.5">
                          <div
                            className="size-2.5 rounded-full"
                            style={{ background: isLatest ? "var(--lagoon)" : "var(--line)" }}
                          />
                          {idx < versions.length - 1 && (
                            <div className="mt-1 w-px flex-1" style={{ background: "var(--line)", minHeight: "24px" }} />
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="font-mono font-bold" style={{ color: "var(--sea-ink)" }}>
                              v{v.version}
                            </span>
                            {isLatest && (
                              <span
                                className="rounded-full px-2 py-0.5 text-[10px] font-semibold"
                                style={{ background: "var(--lagoon)", color: "#fff" }}
                              >
                                latest
                              </span>
                            )}
                          </div>
                          <p className="mt-0.5 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
                            Published {new Date(v.published_at).toLocaleDateString()}
                            {" · "}
                            {v.download_count.toLocaleString()} downloads
                            {dlDelta !== null && dlDelta > 0 && (
                              <span style={{ color: "var(--lagoon-deep)" }}> (+{dlDelta.toLocaleString()} over prev)</span>
                            )}
                          </p>
                        </div>
                      </div>
                    )
                  })}
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

          <TabsContent value="downloads">
            <div className="island-shell rounded-xl p-6">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="font-bold" style={{ color: "var(--sea-ink)" }}>Download trend</h2>
                <span className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Last 30 days</span>
              </div>
              {trendData.length === 0 ? (
                <div className="flex h-[200px] items-center justify-center text-sm" style={{ color: "var(--sea-ink-soft)" }}>
                  No download data yet.
                </div>
              ) : (
                <ResponsiveContainer width="100%" height={200}>
                  <BarChart data={trendData} margin={{ top: 4, right: 4, left: -20, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--line)" vertical={false} />
                    <XAxis dataKey="day" tick={{ fontSize: 11, fill: "var(--sea-ink-soft)" }} axisLine={false} tickLine={false} />
                    <YAxis tick={{ fontSize: 11, fill: "var(--sea-ink-soft)" }} axisLine={false} tickLine={false} />
                    <Tooltip
                      contentStyle={{ background: "var(--surface-strong)", border: "1px solid var(--line)", borderRadius: "8px", fontSize: 12 }}
                      labelStyle={{ color: "var(--sea-ink)", fontWeight: 600 }}
                      itemStyle={{ color: "var(--lagoon-deep)" }}
                    />
                    <Bar dataKey="downloads" fill="var(--lagoon)" radius={[4, 4, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              )}
            </div>
          </TabsContent>

          <TabsContent value="graph">
            <div className="island-shell rounded-xl p-6">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="font-bold" style={{ color: "var(--sea-ink)" }}>Integration map</h2>
                <span className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>Click a node to navigate</span>
              </div>
              <DependencyGraph
                packageName={pkg.name}
                integratesWith={pkg.integrates_with ?? []}
              />
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
