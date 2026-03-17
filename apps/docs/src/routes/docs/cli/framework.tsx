import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/cli/framework")({
  component: CliFrameworkPage,
})

function CliFrameworkPage() {
  return (
    <>
      <h1>tsx framework</h1>
      <p>
        Commands for authoring, validating, and publishing Framework Package Format (FPF) packages.
        Use these when you want to create a reusable pattern that others can install via{" "}
        <code>tsx install</code>.
      </p>

      <h2>tsx framework init</h2>
      <p>Scaffold a new FPF package in the current directory:</p>
      <pre><code>tsx framework init</code></pre>
      <p>This creates the following structure:</p>
      <pre><code>{`.
├── manifest.json       # Package metadata and generator definitions
├── generators/         # Generator scripts (Rhai or shell)
│   └── default.rhai
└── templates/          # File templates used by generators
    └── example.ts.hbs`}</code></pre>

      <h2>tsx framework validate</h2>
      <p>Validate the <code>manifest.json</code> in the current directory against the FPF schema:</p>
      <pre><code>tsx framework validate</code></pre>
      <p>
        Exits with code <code>0</code> on success, <code>1</code> on validation errors.
        Reports missing required fields, invalid semver, unknown capability tokens, and malformed
        <code>output_paths</code> patterns.
      </p>

      <h2>tsx framework preview</h2>
      <p>Run a generator locally and print the output files to stdout without writing them:</p>
      <pre><code>{`tsx framework preview --generator default
tsx framework preview --generator crud --input '{"model":"Post"}'`}</code></pre>

      <h2>tsx framework add</h2>
      <p>Add a new generator scaffold to an existing package:</p>
      <pre><code>tsx framework add &lt;generator-name&gt;</code></pre>

      <h2>tsx framework publish</h2>
      <p>Package and publish to the registry:</p>
      <pre><code>{`# Publish to the default registry
tsx framework publish --api-key sk_live_...

# Publish to a self-hosted registry
tsx framework publish --registry https://registry.my-org.com --api-key sk_...

# Validate and package without uploading
tsx framework publish --dry-run`}</code></pre>

      <h3>Publish flags</h3>
      <ul>
        <li><code>--registry &lt;url&gt;</code> — Target registry (default: <code>TSX_REGISTRY_URL</code> or public registry).</li>
        <li><code>--api-key &lt;key&gt;</code> — API key for authentication. Generate one in the registry dashboard.</li>
        <li><code>--dry-run</code> — Validate and create the tarball without uploading.</li>
      </ul>

      <h2>Full authoring workflow</h2>
      <ol>
        <li>Run <code>tsx framework init</code> to scaffold the package.</li>
        <li>Edit <code>manifest.json</code> — set <code>id</code>, <code>name</code>, <code>provides</code>, and define your generators.</li>
        <li>Write your generator scripts and templates.</li>
        <li>Run <code>tsx framework validate</code> to check for errors.</li>
        <li>Run <code>tsx framework preview</code> to inspect the output.</li>
        <li>Run <code>tsx framework publish --api-key &lt;key&gt;</code> to publish.</li>
      </ol>

      <p>
        See the <a href="/docs/fpf">FPF Format</a> docs for the full manifest specification.
      </p>
    </>
  )
}
