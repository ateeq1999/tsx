import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/fpf/manifest")({
  component: FpfManifestPage,
})

function FpfManifestPage() {
  return (
    <>
      <h1>manifest.json reference</h1>
      <p>
        Every FPF package must have a <code>manifest.json</code> at its root. This file is the
        single source of truth for everything tsx needs to install, run, and integrate the package.
      </p>

      <h2>Full example</h2>
      <pre><code>{`{
  "id": "with-auth",
  "name": "Better Auth",
  "description": "Full Better Auth integration for TanStack Start.",
  "version": "1.3.0",
  "tsx_min": "0.4.0",
  "lang": "typescript",
  "runtime": "node",
  "license": "MIT",
  "author": "ateeq1999",

  "provides": ["auth", "session"],

  "integrates_with": {
    "tanstack-crud": {
      "slot": "route.beforeLoad",
      "inject": "templates/auth-guard-slot.hbs"
    }
  },

  "generators": [
    {
      "id": "setup",
      "command": "generators/setup.rhai",
      "description": "Scaffold the full auth integration.",
      "schema": {},
      "output_paths": [
        "src/lib/auth.ts",
        "src/lib/auth-client.ts",
        "src/middleware/auth-guard.ts"
      ]
    },
    {
      "id": "add-route",
      "command": "generators/add-route.rhai",
      "description": "Add a new protected route.",
      "schema": {
        "name": { "type": "string", "description": "Route name (kebab-case)" }
      },
      "output_paths": [
        "src/routes/_protected/{{name}}/index.tsx"
      ]
    }
  ],

  "style": {
    "quotes": "double",
    "indent": 2,
    "semicolons": false
  },

  "paths": {
    "@auth": "src/lib"
  }
}`}</code></pre>

      <h2>Top-level fields</h2>
      <ul>
        <li><code>id</code> <em>(string, required)</em> — Unique identifier used as the install name. Lowercase, hyphens allowed.</li>
        <li><code>name</code> <em>(string, required)</em> — Human-readable display name.</li>
        <li><code>description</code> <em>(string, required)</em> — One-line description shown in search results.</li>
        <li><code>version</code> <em>(string, required)</em> — Semver version string (<code>MAJOR.MINOR.PATCH</code>).</li>
        <li><code>tsx_min</code> <em>(string, required)</em> — Minimum tsx CLI version required to install this package.</li>
        <li><code>lang</code> <em>(string, required)</em> — Primary language: <code>typescript</code>, <code>python</code>, <code>rust</code>, <code>go</code>.</li>
        <li><code>runtime</code> <em>(string, optional)</em> — Target runtime: <code>node</code>, <code>bun</code>, <code>deno</code>.</li>
        <li><code>license</code> <em>(string, optional)</em> — SPDX license identifier (e.g. <code>MIT</code>).</li>
        <li><code>author</code> <em>(string, optional)</em> — Author name or username.</li>
      </ul>

      <h2>provides[]</h2>
      <p>
        An array of capability tokens this package adds to the project. Used by other packages to
        discover integration points and by the registry for filtering.
      </p>
      <pre><code>{`"provides": ["auth", "session", "email"]`}</code></pre>
      <p>Common tokens: <code>auth</code>, <code>crud</code>, <code>forms</code>, <code>i18n</code>, <code>payments</code>, <code>storage</code>, <code>email</code>, <code>analytics</code>.</p>

      <h2>integrates_with{"{}"}</h2>
      <p>
        A map from other package IDs to slot injection descriptors. When both packages are installed,
        tsx threads the injection automatically at generator time.
      </p>
      <pre><code>{`"integrates_with": {
  "tanstack-crud": {
    "slot": "route.beforeLoad",
    "inject": "templates/auth-guard-slot.hbs"
  }
}`}</code></pre>
      <ul>
        <li><code>slot</code> — Named injection point defined by the target package's generator.</li>
        <li><code>inject</code> — Path to the Handlebars template to inject at that slot, relative to the package root.</li>
      </ul>

      <h2>generators[]</h2>
      <p>Each entry describes one callable generator:</p>
      <ul>
        <li><code>id</code> — Generator name used in <code>tsx run &lt;pkg&gt;:&lt;id&gt;</code>.</li>
        <li><code>command</code> — Path to the generator script (Rhai <code>.rhai</code> or shell <code>.sh</code>), relative to the package root.</li>
        <li><code>description</code> — Shown in <code>tsx info</code>.</li>
        <li><code>schema</code> — JSON Schema for the <code>--input</code> object. Used for validation and IDE autocomplete.</li>
        <li><code>output_paths</code> — List of files the generator will create. Supports Handlebars expressions using input schema keys (e.g. <code>{"{{name}}"}</code>).</li>
      </ul>

      <h2>style{"{}"}</h2>
      <p>Code style hints applied when rendering templates:</p>
      <ul>
        <li><code>quotes</code> — <code>"single"</code> or <code>"double"</code>.</li>
        <li><code>indent</code> — Number of spaces (default: <code>2</code>).</li>
        <li><code>semicolons</code> — <code>true</code> or <code>false</code>.</li>
      </ul>

      <h2>paths{"{}"}</h2>
      <p>
        TypeScript path alias map merged into the project's <code>tsconfig.json</code> paths when
        <code>tsx stack apply</code> is run:
      </p>
      <pre><code>{`"paths": {
  "@auth": "src/lib",
  "@auth/*": "src/lib/*"
}`}</code></pre>
    </>
  )
}
