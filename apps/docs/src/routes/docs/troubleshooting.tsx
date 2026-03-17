import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/troubleshooting")({
  component: TroubleshootingPage,
})

function TroubleshootingPage() {
  return (
    <div>
      <h1>Troubleshooting</h1>
      <p>
        Common problems and how to fix them. If you don't find your issue here, open a{" "}
        <a href="https://github.com/ateeq1999/tsx/issues" target="_blank" rel="noreferrer">GitHub issue</a>.
      </p>

      <hr />

      <h2>Installation issues</h2>

      <h3>cargo install tsx fails with "no matching package"</h3>
      <p>
        The crate may not be published yet. Install from source instead:
      </p>
      <pre><code className="language-bash">{"cargo install --git https://github.com/ateeq1999/tsx tsx"}</code></pre>
      <p>
        Or download a pre-built binary from the{" "}
        <a href="https://github.com/ateeq1999/tsx/releases" target="_blank" rel="noreferrer">Releases page</a>.
      </p>

      <h3>tsx: command not found after installing</h3>
      <p>
        Cargo installs binaries to <code>~/.cargo/bin</code>. Make sure this directory is on your PATH:
      </p>
      <pre><code className="language-bash">{`# Add to ~/.bashrc, ~/.zshrc, or ~/.profile
export PATH="$HOME/.cargo/bin:$PATH"

# Reload your shell
source ~/.bashrc`}</code></pre>
      <p>On Windows (PowerShell), Cargo usually updates PATH automatically. Restart your terminal.</p>

      <h3>Permission denied on macOS/Linux</h3>
      <p>
        If <code>cargo install</code> fails with a permissions error, do <strong>not</strong> use <code>sudo</code>. Instead, ensure <code>~/.cargo</code> is owned by your user:
      </p>
      <pre><code className="language-bash">{"chown -R $(whoami) ~/.cargo"}</code></pre>

      <hr />

      <h2>Registry install issues</h2>

      <h3>tsx registry install hangs or times out</h3>
      <p>Check that the registry is reachable:</p>
      <pre><code className="language-bash">{"curl https://registry.tsx.dev/health"}</code></pre>
      <p>
        If you're using a self-hosted registry, verify the <code>TSX_REGISTRY_URL</code> environment variable is correct and the server is running.
      </p>

      <h3>Package not found</h3>
      <p>
        Search first to confirm the exact package name:
      </p>
      <pre><code className="language-bash">{"tsx registry search <query>"}</code></pre>
      <p>Package names are case-sensitive and often scoped (e.g. <code>@tsx-pkg/with-auth</code>).</p>

      <h3>Version mismatch error</h3>
      <p>
        The package requires a newer version of tsx than you have installed. Update tsx:
      </p>
      <pre><code className="language-bash">{"cargo install tsx --force"}</code></pre>
      <p>Or install a specific older package version:</p>
      <pre><code className="language-bash">{"tsx registry install <name>@<version>"}</code></pre>

      <h3>Download completes but files are not written</h3>
      <p>
        Check the target directory. By default tsx writes to <code>.tsx/packages/&lt;name&gt;/</code> in the current directory.
        Use <code>--dir</code> to override:
      </p>
      <pre><code className="language-bash">{"tsx registry install <name> --dir ./my-patterns"}</code></pre>
      <p>
        Also confirm you have write permission to the target directory.
      </p>

      <hr />

      <h2>Framework package issues</h2>

      <h3>tsx framework validate fails</h3>
      <p>
        The validator checks your <code>manifest.json</code> against the FPF schema. Common mistakes:
      </p>
      <ul>
        <li><code>version</code> must follow semver (e.g. <code>1.0.0</code>, not <code>v1.0.0</code>)</li>
        <li><code>provides</code> items must be strings</li>
        <li><code>generators[].output_paths</code> must be relative paths</li>
        <li>All referenced template files must exist in the <code>generators/</code> directory</li>
      </ul>
      <p>Run with <code>--verbose</code> for detailed validation output:</p>
      <pre><code className="language-bash">{"tsx framework validate --verbose"}</code></pre>

      <h3>tsx framework publish returns 401</h3>
      <p>
        Your API key is missing or invalid. Pass it explicitly:
      </p>
      <pre><code className="language-bash">{"tsx framework publish --api-key <your-key>"}</code></pre>
      <p>
        Get your key from the <a href="/account/api-keys">Account → API keys</a> page (requires login).
      </p>

      <h3>Generator produces wrong paths</h3>
      <p>
        Verify <code>output_paths</code> in your manifest use the correct path alias tokens. Run a preview first to see the resolved paths without writing files:
      </p>
      <pre><code className="language-bash">{"tsx framework preview <generator-id>"}</code></pre>

      <hr />

      <h2>Stack issues</h2>

      <h3>tsx stack detect shows wrong framework</h3>
      <p>
        Detection reads <code>package.json</code> and local config files. If you have multiple frameworks present (monorepo), run from the specific app directory:
      </p>
      <pre><code className="language-bash">{"cd apps/web && tsx stack detect"}</code></pre>

      <h3>tsx stack apply fails with "slot not found"</h3>
      <p>
        The package tries to inject into a slot that doesn't exist in your project. Possible causes:
      </p>
      <ul>
        <li>The target file was moved or renamed — check <code>stack.json</code> paths</li>
        <li>The package is designed for a different framework version</li>
        <li>The slot comment marker was deleted from the target file</li>
      </ul>
      <p>
        Re-add the slot marker manually:
      </p>
      <pre><code className="language-typescript">{"// @tsx-slot providers"}</code></pre>

      <hr />

      <h2>Still stuck?</h2>
      <p>
        Run any command with <code>--debug</code> for verbose logs, then{" "}
        <a href="https://github.com/ateeq1999/tsx/issues/new" target="_blank" rel="noreferrer">open a GitHub issue</a> with the output.
      </p>
    </div>
  )
}
