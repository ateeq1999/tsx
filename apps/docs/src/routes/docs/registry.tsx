import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/registry")({
  component: RegistryPage,
})

function RegistryPage() {
  return (
    <>
      <h1>Registry</h1>
      <p>
        tsx ships with a built-in registry server you can self-host. It exposes a REST API
        that the CLI uses for search, install, info, and publish operations.
      </p>

      <h2>Self-hosting</h2>
      <p>The registry server is a single Rust binary included in the tsx workspace:</p>
      <pre><code>cd crates/registry-server
cargo run --release</code></pre>
      <p>It listens on <code>0.0.0.0:8080</code> by default.</p>

      <h3>Environment variables</h3>
      <ul>
        <li><code>DATABASE_URL</code> — PostgreSQL connection string.</li>
        <li><code>STORAGE_PATH</code> — Directory for package tarballs (default: <code>./data</code>).</li>
        <li><code>PORT</code> — Port to listen on (default: <code>8080</code>).</li>
        <li><code>REGISTRY_API_KEY</code> — Required API key for publish operations.</li>
      </ul>

      <h2>Pointing the CLI at your registry</h2>
      <p>Set the <code>TSX_REGISTRY_URL</code> environment variable:</p>
      <pre><code>export TSX_REGISTRY_URL=https://registry.your-org.com
tsx install my-pattern</code></pre>

      <h2>API Reference</h2>

      <h3><code>GET /v1/search?q=&size=20</code></h3>
      <p>Search for packages. Returns a paginated list with total count.</p>

      <h3><code>GET /v1/packages/:name</code></h3>
      <p>Get metadata for a specific package.</p>

      <h3><code>GET /v1/packages/:name/versions</code></h3>
      <p>List all versions of a package.</p>

      <h3><code>GET /v1/packages/:name/:version/tarball</code></h3>
      <p>Download a package version as a <code>.tar.gz</code> archive.</p>

      <h3><code>POST /v1/packages/publish</code></h3>
      <p>
        Publish a new package version. Requires <code>Authorization: Bearer &lt;api-key&gt;</code>.
        Rate limited to 10 requests per minute per IP.
      </p>

      <h3><code>GET /v1/stats</code></h3>
      <p>Registry-wide statistics: total packages, downloads, versions, weekly activity.</p>
    </>
  )
}
