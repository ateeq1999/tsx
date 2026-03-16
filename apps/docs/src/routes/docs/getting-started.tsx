import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/getting-started")({
  component: GettingStartedPage,
})

function GettingStartedPage() {
  return (
    <>
      <h1>Getting Started</h1>
      <p>
        tsx is a CLI for installing and publishing reusable code patterns for TanStack Start projects.
        Think of it like shadcn/ui, but for full-stack patterns — routing, auth, CRUD, and more.
      </p>

      <h2>Installation</h2>
      <p>Install tsx globally via Cargo:</p>
      <pre><code>cargo install tsx-forge</code></pre>
      <p>Or download a prebuilt binary from the GitHub releases page.</p>

      <h2>Your first install</h2>
      <p>Navigate to a TanStack Start project, then run:</p>
      <pre><code>tsx install with-auth</code></pre>
      <p>
        This will scaffold Better Auth integration — auth server, client, middleware, and a
        protected dashboard route — directly into your project.
      </p>

      <h2>Search for patterns</h2>
      <pre><code>tsx search crud</code></pre>

      <h2>Get info about a package</h2>
      <pre><code>tsx info with-auth</code></pre>

      <h2>Next steps</h2>
      <ul>
        <li>Read the <a href="/docs/cli">CLI reference</a> to learn all available commands</li>
        <li>Learn about <a href="/docs/fpf">FPF (Framework Package Format)</a> to publish your own patterns</li>
        <li>Browse the <a href="https://registry.tsx.dev">public registry</a> to discover community packages</li>
      </ul>
    </>
  )
}
