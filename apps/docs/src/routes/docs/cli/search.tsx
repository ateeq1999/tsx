import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/cli/search")({
  component: CliSearchPage,
})

function CliSearchPage() {
  return (
    <>
      <h1>tsx search</h1>
      <p>
        Search for packages in the registry. Results are sorted by relevance and include
        the package name, latest version, description, and download count.
      </p>

      <h2>Usage</h2>
      <pre><code>tsx search [query] [flags]</code></pre>

      <h2>Examples</h2>
      <pre><code>{`# Full-text search
tsx search auth

# Filter by language
tsx search crud --lang typescript

# Return raw JSON (useful for scripting)
tsx search form --json

# Increase result count
tsx search -- --size 50`}</code></pre>

      <h2>Flags</h2>
      <ul>
        <li>
          <code>--lang &lt;lang&gt;</code> — Filter results to packages that target a specific language.
          Accepted values: <code>typescript</code>, <code>python</code>, <code>rust</code>, <code>go</code>.
        </li>
        <li>
          <code>--size &lt;n&gt;</code> — Number of results to return per page (default: <code>20</code>, max: <code>100</code>).
        </li>
        <li>
          <code>--page &lt;n&gt;</code> — Page number for paginated results (default: <code>1</code>).
        </li>
        <li>
          <code>--json</code> — Print the full JSON response instead of a formatted table.
          Useful for piping into <code>jq</code> or other tools.
        </li>
        <li>
          <code>--registry &lt;url&gt;</code> — Search a specific registry instead of the default.
        </li>
      </ul>

      <h2>Output format</h2>
      <p>By default, tsx prints a table with the following columns:</p>
      <pre><code>{`NAME                VERSION   DOWNLOADS   DESCRIPTION
with-auth           1.3.0     4 201       Better Auth integration for TanStack Start
tanstack-crud       2.1.0     2 880       Full CRUD with TanStack Query + optimistic updates`}</code></pre>

      <h2>JSON output</h2>
      <p>
        With <code>--json</code>, the full <code>SearchResult</code> object is printed:
      </p>
      <pre><code>{`{
  "packages": [...],
  "total": 12,
  "page": 1,
  "per_page": 20
}`}</code></pre>
    </>
  )
}
