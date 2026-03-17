import { createAPIFileRoute } from "@tanstack/react-start/api"
import { registryApi } from "@/lib/api"

export const APIRoute = createAPIFileRoute("/api/sitemap.xml")({
  GET: async () => {
    const base = import.meta.env.VITE_SITE_URL ?? "https://registry.tsx.dev"

    let packageUrls = ""
    try {
      const result = await registryApi.search("", 1, 200)
      packageUrls = result.packages
        .map(
          (pkg) =>
            `  <url><loc>${base}/packages/${encodeURIComponent(pkg.name)}</loc><changefreq>weekly</changefreq></url>`
        )
        .join("\n")
    } catch {
      // registry unavailable at build time — skip package URLs
    }

    const staticUrls = ["/", "/browse"].map(
      (path) =>
        `  <url><loc>${base}${path}</loc><changefreq>daily</changefreq></url>`
    ).join("\n")

    const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${staticUrls}
${packageUrls}
</urlset>`

    return new Response(xml, {
      headers: { "Content-Type": "application/xml; charset=utf-8" },
    })
  },
})
