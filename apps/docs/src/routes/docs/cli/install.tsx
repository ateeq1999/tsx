import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/cli/install")({
  component: CliInstallPage,
})

function CliInstallPage() {
  return (
    <>
      <h1>tsx install</h1>
      <p>
        Install a framework package from the registry into the current project.
        tsx copies the package's generators, templates, and manifest into{" "}
        <code>.tsx/packages/&lt;name&gt;/</code> and wires up any slot injections defined in
        the manifest's <code>integrates_with</code> map.
      </p>

      <h2>Usage</h2>
      <pre><code>tsx install &lt;package&gt; [flags]</code></pre>

      <h2>Examples</h2>
      <pre><code>{`# Install the latest version of a package
tsx install with-auth

# Install a specific version
tsx install with-auth@1.2.0

# Install from a self-hosted registry
tsx install with-auth --registry https://registry.my-org.com

# Preview what would be installed without writing files
tsx install with-auth --dry-run`}</code></pre>

      <h2>Flags</h2>
      <ul>
        <li>
          <code>--registry &lt;url&gt;</code> — Override the default registry URL.
          Equivalent to setting <code>TSX_REGISTRY_URL</code> for this invocation.
        </li>
        <li>
          <code>--dir &lt;path&gt;</code> — Install into a specific directory instead of{" "}
          <code>.tsx/packages/</code> in the current working directory.
        </li>
        <li>
          <code>--force</code> — Overwrite existing package files if the package is already installed.
        </li>
        <li>
          <code>--dry-run</code> — Print what files would be created or modified without writing anything to disk.
        </li>
        <li>
          <code>--offline</code> — Use a previously cached tarball and skip the network request.
        </li>
      </ul>

      <h2>What gets installed</h2>
      <p>After a successful install, tsx writes the following under <code>.tsx/packages/&lt;name&gt;/</code>:</p>
      <ul>
        <li><code>manifest.json</code> — The package manifest pinned to the installed version.</li>
        <li><code>generators/</code> — Forge generator scripts.</li>
        <li><code>templates/</code> — Handlebars or plain-text templates used by generators.</li>
      </ul>
      <p>
        tsx also updates <code>.tsx/stack.json</code> to record the installed package and its version,
        enabling reproducible installs via <code>tsx stack apply</code>.
      </p>

      <h2>Version pinning</h2>
      <p>
        When you install a package, the exact version is written to <code>.tsx/stack.json</code>.
        Running <code>tsx stack apply</code> later will install exactly those versions, even if
        newer versions have been published.
      </p>

      <h2>Environment variables</h2>
      <ul>
        <li><code>TSX_REGISTRY_URL</code> — Default registry base URL (overridden by <code>--registry</code>).</li>
      </ul>
    </>
  )
}
