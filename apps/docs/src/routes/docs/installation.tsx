import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/installation")({
  component: InstallationPage,
})

function InstallationPage() {
  return (
    <>
      <h1>Installation</h1>
      <p>
        tsx is distributed as a single native binary. Pick the method that suits your workflow.
      </p>

      <h2>From crates.io (recommended)</h2>
      <p>
        If you have Rust installed, install directly from the official registry:
      </p>
      <pre><code>cargo install tsx-forge</code></pre>
      <p>
        Cargo will compile and install the binary into <code>~/.cargo/bin/tsx</code>.
        Make sure that directory is on your <code>PATH</code>.
      </p>

      <h2>From source</h2>
      <p>To install the latest development build from the GitHub repository:</p>
      <pre><code>cargo install --git https://github.com/ateeq1999/tsx tsx-forge</code></pre>

      <h2>Pre-built binaries</h2>
      <p>
        Compiled binaries for common platforms are published on the{" "}
        <a href="https://github.com/ateeq1999/tsx/releases" target="_blank" rel="noreferrer">
          GitHub Releases page
        </a>
        . Download the archive for your platform, extract it, and move the binary to a directory on your <code>PATH</code>.
      </p>

      <h3>Linux / macOS</h3>
      <pre><code>{`# Example for Linux x86_64
curl -L https://github.com/ateeq1999/tsx/releases/latest/download/tsx-linux-x86_64.tar.gz | tar xz
sudo mv tsx /usr/local/bin/`}</code></pre>

      <h3>Windows</h3>
      <p>
        Download <code>tsx-windows-x86_64.zip</code> from the releases page, unzip it, and add the folder to your
        system <code>PATH</code> via <em>System Properties → Environment Variables</em>.
      </p>

      <h2>Shell completions</h2>
      <p>Generate tab completions for your shell:</p>
      <pre><code>{`tsx completions bash   >> ~/.bashrc
tsx completions zsh    >> ~/.zshrc
tsx completions fish   > ~/.config/fish/completions/tsx.fish
tsx completions powershell >> $PROFILE`}</code></pre>

      <h2>Verify the install</h2>
      <pre><code>tsx --version</code></pre>
      <p>You should see the installed version number printed to stdout.</p>

      <h2>Updating</h2>
      <p>Re-run the same install command to get the latest version. For cargo installs:</p>
      <pre><code>cargo install tsx-forge --force</code></pre>
    </>
  )
}
