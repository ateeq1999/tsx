import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/cli/stack")({
  component: CliStackPage,
})

function CliStackPage() {
  return (
    <>
      <h1>tsx stack</h1>
      <p>
        Manage the project's <code>.tsx/stack.json</code> — a lock-file-style record of every
        installed package and its pinned version. Stack commands let you reproduce, inspect, and
        modify the installed package set.
      </p>

      <h2>tsx stack init</h2>
      <p>Create a new <code>.tsx/stack.json</code> in the current project:</p>
      <pre><code>tsx stack init</code></pre>
      <p>
        If <code>stack.json</code> already exists this is a no-op. Typically you don't need
        to call this manually — <code>tsx install</code> creates it automatically on first use.
      </p>

      <h2>tsx stack show</h2>
      <p>Print the current stack (installed packages and their versions):</p>
      <pre><code>tsx stack show</code></pre>
      <pre><code>{`Installed packages:
  with-auth        1.3.0
  tanstack-crud    2.1.0`}</code></pre>

      <h2>tsx stack add</h2>
      <p>Record a package in <code>stack.json</code> without running the install (advanced use):</p>
      <pre><code>tsx stack add &lt;package&gt;@&lt;version&gt;</code></pre>

      <h2>tsx stack remove</h2>
      <p>Remove a package from the stack and delete its files from <code>.tsx/packages/</code>:</p>
      <pre><code>tsx stack remove &lt;package&gt;</code></pre>

      <h2>tsx stack apply</h2>
      <p>
        Install all packages listed in <code>stack.json</code> at their pinned versions.
        Useful after cloning a project or switching branches:
      </p>
      <pre><code>{`tsx stack apply

# Auto-install missing packages and skip already-installed ones
tsx stack apply --install`}</code></pre>

      <h3>apply flags</h3>
      <ul>
        <li>
          <code>--install</code> — Download and install any packages listed in <code>stack.json</code>
          that are not yet present in <code>.tsx/packages/</code>.
        </li>
        <li>
          <code>--force</code> — Re-install all packages, overwriting existing files.
        </li>
      </ul>

      <h2>tsx stack detect</h2>
      <p>
        Scan the project for installed packages and regenerate <code>stack.json</code>
        from what's actually on disk (useful if <code>stack.json</code> is out of sync):
      </p>
      <pre><code>tsx stack detect</code></pre>

      <h2>stack.json format</h2>
      <pre><code>{`{
  "version": 1,
  "packages": {
    "with-auth": "1.3.0",
    "tanstack-crud": "2.1.0"
  },
  "paths": {
    "@auth": ".tsx/packages/with-auth/src"
  }
}`}</code></pre>
      <p>
        The <code>paths</code> map is merged into your project's TypeScript path aliases when you
        run <code>tsx stack apply</code>, keeping imports like <code>import {"{...}"} from "@auth/client"</code> working
        out of the box.
      </p>
    </>
  )
}
