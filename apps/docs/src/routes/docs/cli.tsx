import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/cli")({
  component: CliPage,
})

const commands = [
  {
    cmd: "tsx install <package>",
    desc: "Install a pattern from the registry into your project.",
    flags: [
      { flag: "--dry-run", desc: "Preview what files would be created without writing anything." },
      { flag: "--force", desc: "Overwrite existing files." },
      { flag: "--target <dir>", desc: "Install into a specific directory (default: current dir)." },
    ],
  },
  {
    cmd: "tsx search [query]",
    desc: "Search for packages in the registry.",
    flags: [
      { flag: "--size <n>", desc: "Number of results to return (default: 20)." },
    ],
  },
  {
    cmd: "tsx info <package>",
    desc: "Show metadata for a package: version, description, tags, author.",
    flags: [],
  },
  {
    cmd: "tsx framework init",
    desc: "Scaffold a new FPF package in the current directory.",
    flags: [],
  },
  {
    cmd: "tsx framework validate",
    desc: "Validate the stack.json manifest in the current directory.",
    flags: [],
  },
  {
    cmd: "tsx framework publish [--registry <url>]",
    desc: "Publish the current FPF package to the registry.",
    flags: [
      { flag: "--registry <url>", desc: "Target a self-hosted registry instead of the default." },
      { flag: "--api-key <key>", desc: "API key for authenticated publish." },
      { flag: "--dry-run", desc: "Validate and package without uploading." },
    ],
  },
  {
    cmd: "tsx stack apply",
    desc: "Apply a stack profile (.tsx/stack.json) to the current project.",
    flags: [
      { flag: "--profile <name>", desc: "Named profile to apply (default: 'default')." },
    ],
  },
]

function CliPage() {
  return (
    <>
      <h1>CLI Reference</h1>
      <p>
        Complete reference for all tsx commands, flags, and environment variables.
      </p>

      <h2>Environment variables</h2>
      <ul>
        <li>
          <code>TSX_REGISTRY_URL</code> — Override the default registry URL.
          Set this to point to a self-hosted registry.
        </li>
      </ul>

      {commands.map((c) => (
        <div key={c.cmd} style={{ marginBottom: "2rem" }}>
          <h3><code>{c.cmd}</code></h3>
          <p>{c.desc}</p>
          {c.flags.length > 0 && (
            <ul>
              {c.flags.map((f) => (
                <li key={f.flag}>
                  <code>{f.flag}</code> — {f.desc}
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </>
  )
}
