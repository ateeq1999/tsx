import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/registry/api")({
  component: RegistryApiPage,
})

function RegistryApiPage() {
  return (
    <>
      <h1>API Reference</h1>
      <p>
        The registry server exposes a REST API used by the tsx CLI and the web dashboard.
        All endpoints return JSON unless noted otherwise.
      </p>

      <h2>Base URL</h2>
      <pre><code>https://registry.tsx.dev</code></pre>
      <p>
        For self-hosted registries, replace with your own domain. The CLI reads{" "}
        <code>TSX_REGISTRY_URL</code> for the base URL.
      </p>

      <h2>Authentication</h2>
      <p>
        Only the publish endpoint requires authentication. Pass your API key as a Bearer token:
      </p>
      <pre><code>Authorization: Bearer sk_live_YOUR_KEY_HERE</code></pre>

      <hr />

      <h2><code>GET /health</code></h2>
      <p>Server health check. Returns 200 when the server and database are reachable.</p>
      <pre><code>{`curl https://registry.tsx.dev/health

200 OK
{ "status": "ok" }`}</code></pre>

      <hr />

      <h2><code>GET /v1/stats</code></h2>
      <p>Registry-wide statistics.</p>
      <pre><code>{`curl https://registry.tsx.dev/v1/stats

{
  "total_packages": 42,
  "total_downloads": 18204,
  "total_versions": 97,
  "packages_this_week": 5
}`}</code></pre>

      <hr />

      <h2><code>GET /v1/search</code></h2>
      <p>Full-text search across package names and descriptions.</p>
      <h3>Query parameters</h3>
      <ul>
        <li><code>q</code> — Search query (required).</li>
        <li><code>lang</code> — Filter by language: <code>typescript</code>, <code>python</code>, etc.</li>
        <li><code>page</code> — Page number (default: <code>1</code>).</li>
        <li><code>size</code> — Results per page (default: <code>20</code>, max: <code>100</code>).</li>
      </ul>
      <pre><code>{`curl "https://registry.tsx.dev/v1/search?q=auth&size=5"

{
  "packages": [
    {
      "name": "with-auth",
      "version": "1.3.0",
      "description": "Better Auth integration for TanStack Start.",
      "author": "ateeq1999",
      "license": "MIT",
      "tags": ["auth", "session"],
      "tsx_min": "0.4.0",
      "download_count": 4201,
      "created_at": "2025-11-01T00:00:00Z",
      "updated_at": "2026-01-15T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 5
}`}</code></pre>

      <hr />

      <h2><code>GET /v1/packages</code></h2>
      <p>List packages. Supports sorting.</p>
      <h3>Query parameters</h3>
      <ul>
        <li><code>sort</code> — <code>recent</code> (default) or <code>downloads</code>.</li>
        <li><code>limit</code> — Number of results (default: <code>20</code>).</li>
      </ul>
      <pre><code>{`curl "https://registry.tsx.dev/v1/packages?sort=recent&limit=10"`}</code></pre>

      <hr />

      <h2><code>GET /v1/packages/:name</code></h2>
      <p>Get metadata for a specific package (latest version).</p>
      <pre><code>{`curl https://registry.tsx.dev/v1/packages/with-auth`}</code></pre>
      <p>Returns the same <code>Package</code> object as the search results. Returns 404 if not found.</p>

      <hr />

      <h2><code>GET /v1/packages/:name/versions</code></h2>
      <p>List all published versions of a package.</p>
      <pre><code>{`curl https://registry.tsx.dev/v1/packages/with-auth/versions

[
  { "version": "1.3.0", "published_at": "2026-01-15T00:00:00Z", "download_count": 2100 },
  { "version": "1.2.0", "published_at": "2025-12-01T00:00:00Z", "download_count": 1800 },
  { "version": "1.0.0", "published_at": "2025-11-01T00:00:00Z", "download_count": 301 }
]`}</code></pre>

      <hr />

      <h2><code>GET /v1/packages/:name/:version/tarball</code></h2>
      <p>Download a specific version as a <code>.tar.gz</code> archive.</p>
      <pre><code>{`curl -L https://registry.tsx.dev/v1/packages/with-auth/1.3.0/tarball -o with-auth-1.3.0.tar.gz`}</code></pre>
      <p>Returns the raw binary tarball with <code>Content-Type: application/gzip</code>.</p>

      <hr />

      <h2><code>POST /v1/packages/publish</code></h2>
      <p>Publish a new package or version. Requires authentication.</p>
      <h3>Request</h3>
      <p>Multipart form with the following fields:</p>
      <ul>
        <li><code>manifest</code> — The <code>manifest.json</code> file (JSON).</li>
        <li><code>tarball</code> — The package archive (<code>.tar.gz</code>).</li>
      </ul>
      <pre><code>{`curl -X POST https://registry.tsx.dev/v1/packages/publish \\
  -H "Authorization: Bearer sk_live_..." \\
  -F "manifest=@manifest.json" \\
  -F "tarball=@my-pattern-1.0.0.tar.gz"

201 Created
{ "name": "my-pattern", "version": "1.0.0" }`}</code></pre>
      <h3>Error codes</h3>
      <ul>
        <li><code>400</code> — Invalid manifest, bad semver, or missing fields.</li>
        <li><code>401</code> — Missing or invalid API key.</li>
        <li><code>409</code> — Version already published.</li>
        <li><code>429</code> — Rate limit exceeded (10 requests per minute per IP).</li>
      </ul>
    </>
  )
}
