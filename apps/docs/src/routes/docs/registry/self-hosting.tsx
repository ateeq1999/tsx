import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/registry/self-hosting")({
  component: SelfHostingPage,
})

function SelfHostingPage() {
  return (
    <>
      <h1>Self-hosting the registry</h1>
      <p>
        The tsx registry server is a single Rust binary with no external database dependencies —
        it uses SQLite for metadata and stores package tarballs on the local filesystem.
        You can run it on any Linux server, a Fly.io machine, or inside Docker.
      </p>

      <h2>Option A — Binary</h2>
      <p>Download the <code>registry-server</code> binary from the GitHub Releases page and run it:</p>
      <pre><code>{`./registry-server`}</code></pre>
      <p>
        The server listens on <code>0.0.0.0:8080</code> by default and creates
        <code>./data/registry.db</code> and <code>./data/packages/</code> on first run.
      </p>

      <h2>Option B — Docker</h2>
      <p>A multi-stage Dockerfile is included in <code>crates/registry-server/</code>:</p>
      <pre><code>{`cd crates/registry-server
docker build -t tsx-registry .
docker run -p 8080:8080 -v "$(pwd)/data:/data" tsx-registry`}</code></pre>

      <h2>Option C — Fly.io</h2>
      <p>A <code>fly.toml</code> is included at the repo root. Deploy with:</p>
      <pre><code>{`fly launch   # first time
fly deploy   # subsequent deploys`}</code></pre>
      <p>
        The <code>fly.toml</code> mounts a persistent volume at <code>/data</code> for the SQLite
        database and tarballs. Make sure you create the volume before the first deploy:
      </p>
      <pre><code>fly volumes create registry_data --size 10</code></pre>

      <h2>Environment variables</h2>
      <ul>
        <li>
          <code>PORT</code> — Port to listen on (default: <code>8080</code>).
        </li>
        <li>
          <code>DATA_DIR</code> — Directory for SQLite database and tarball storage (default: <code>./data</code>).
          On Fly.io, set this to <code>/data</code>.
        </li>
        <li>
          <code>TSX_REGISTRY_API_KEY</code> — Required secret for <code>POST /v1/packages/publish</code>.
          Set this to a long random string. Generate one with:
          <pre><code>openssl rand -hex 32</code></pre>
        </li>
      </ul>

      <h2>Pointing the CLI at your registry</h2>
      <pre><code>{`# Per-project (add to .env or shell profile)
export TSX_REGISTRY_URL=https://registry.my-org.com

# Per-command override
tsx install my-pattern --registry https://registry.my-org.com`}</code></pre>

      <h2>Backup strategy</h2>
      <p>
        SQLite uses WAL mode. To back up the database safely while the server is running:
      </p>
      <pre><code>sqlite3 data/registry.db ".backup data/registry.db.bak"</code></pre>
      <p>
        For tarballs, back up the entire <code>data/packages/</code> directory.
        On Fly.io, use <code>fly ssh console</code> + <code>tar</code> or configure a Tigris/S3
        bucket as the storage backend (future feature).
      </p>

      <h2>Health check</h2>
      <pre><code>curl https://registry.my-org.com/health</code></pre>
      <p>Returns <code>200 OK</code> with body <code>{`{"status":"ok"}`}</code> when the server is healthy.</p>
    </>
  )
}
