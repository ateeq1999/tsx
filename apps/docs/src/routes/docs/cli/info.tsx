import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/cli/info")({
  component: CliInfoPage,
})

function CliInfoPage() {
  return (
    <>
      <h1>tsx info</h1>
      <p>
        Show detailed metadata for a package: name, description, latest version, all published
        versions, author, license, download count, and the <code>provides</code> /
        <code>integrates_with</code> capability map from its manifest.
      </p>

      <h2>Usage</h2>
      <pre><code>tsx info &lt;package&gt; [flags]</code></pre>

      <h2>Examples</h2>
      <pre><code>{`# Show info for the latest version
tsx info with-auth

# Show info for a specific version
tsx info with-auth@1.2.0

# Print raw JSON
tsx info with-auth --json`}</code></pre>

      <h2>Flags</h2>
      <ul>
        <li>
          <code>--json</code> — Print the full package metadata as JSON instead of a formatted display.
        </li>
        <li>
          <code>--registry &lt;url&gt;</code> — Fetch from a specific registry instead of the default.
        </li>
      </ul>

      <h2>Output</h2>
      <pre><code>{`with-auth  v1.3.0
Better Auth integration for TanStack Start projects.

Author:    Ateeq
License:   MIT
Downloads: 4 201
Updated:   2026-03-10

provides:
  - auth
  - session

integrates_with:
  tanstack-crud: injects session guard into CRUD routes`}</code></pre>

      <h2>Reading provides and integrates_with</h2>
      <p>
        The <code>provides</code> field lists capability tokens that this package adds to your project.
        The <code>integrates_with</code> map describes how this package slots into other installed packages —
        for example, injecting an auth guard into CRUD route templates.
      </p>
      <p>
        When you run <code>tsx install</code>, tsx uses this map to automatically wire packages together
        without manual configuration.
      </p>
    </>
  )
}
