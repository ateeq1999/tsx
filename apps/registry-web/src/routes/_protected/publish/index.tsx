import { createFileRoute, Link } from "@tanstack/react-router"
import { useState, useRef, useEffect } from "react"
import { toast } from "sonner"
import { ArrowLeft, ArrowRight, Check, Upload } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"

const DRAFT_KEY = "tsx-publish-draft"

interface PublishDraft {
  step1: Step1Data
  manifestJson: string
}

export const Route = createFileRoute("/_protected/publish/")({
  component: PublishPage,
})

type Step = 1 | 2 | 3 | 4

interface Step1Data { name: string; version: string; description: string }
interface Step2Data { manifestJson: string; tarball: File | null }

function stepLabel(step: Step) {
  return ["Package info", "Files", "Preview", "Publish"][step - 1]
}

function StepIndicator({ current }: { current: Step }) {
  return (
    <div className="mb-8 flex items-center gap-0">
      {([1, 2, 3, 4] as Step[]).map((s, i) => (
        <div key={s} className="flex items-center">
          <div
            className="flex size-8 items-center justify-center rounded-full text-xs font-bold"
            style={{
              background: s < current ? "var(--lagoon)" : s === current ? "var(--lagoon)" : "var(--line)",
              color: s <= current ? "#fff" : "var(--sea-ink-soft)",
            }}
          >
            {s < current ? <Check className="size-4" /> : s}
          </div>
          <span
            className="ml-2 text-xs hidden sm:inline"
            style={{ color: s === current ? "var(--sea-ink)" : "var(--sea-ink-soft)" }}
          >
            {stepLabel(s)}
          </span>
          {i < 3 && (
            <div className="mx-3 h-px w-8 sm:w-12" style={{ background: "var(--line)" }} />
          )}
        </div>
      ))}
    </div>
  )
}

function Step1({ onNext, initialData }: { onNext: (d: Step1Data) => void; initialData?: Step1Data }) {
  const [data, setData] = useState<Step1Data>(initialData ?? { name: "", version: "", description: "" })
  const [errors, setErrors] = useState<Partial<Step1Data>>({})

  function validate() {
    const e: Partial<Step1Data> = {}
    if (!data.name.match(/^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/))
      e.name = "Use lowercase letters, numbers and hyphens only."
    if (!data.version.match(/^\d+\.\d+\.\d+(-[\w.]+)?(\+[\w.]+)?$/))
      e.version = "Must be a valid semver (e.g. 1.0.0 or 1.0.0-beta.1)."
    if (data.description.length < 10)
      e.description = "Description must be at least 10 characters."
    return e
  }

  function submit(e: React.FormEvent) {
    e.preventDefault()
    const errs = validate()
    if (Object.keys(errs).length) { setErrors(errs); return }
    onNext(data)
  }

  return (
    <form onSubmit={submit} className="space-y-5">
      <div>
        <label className="mb-1.5 block text-sm font-medium" style={{ color: "var(--sea-ink)" }}>
          Package name
        </label>
        <Input
          placeholder="my-pattern"
          value={data.name}
          onChange={(e) => setData((d) => ({ ...d, name: e.target.value }))}
        />
        {errors.name && <p className="mt-1 text-xs text-red-500">{errors.name}</p>}
        <p className="mt-1 text-xs" style={{ color: "var(--sea-ink-soft)" }}>
          Lowercase, no spaces. Will be published as <code>@tsx-pkg/{data.name || "name"}</code>.
        </p>
      </div>

      <div>
        <label className="mb-1.5 block text-sm font-medium" style={{ color: "var(--sea-ink)" }}>
          Version
        </label>
        <Input
          placeholder="1.0.0"
          value={data.version}
          onChange={(e) => setData((d) => ({ ...d, version: e.target.value }))}
        />
        {errors.version && <p className="mt-1 text-xs text-red-500">{errors.version}</p>}
      </div>

      <div>
        <label className="mb-1.5 block text-sm font-medium" style={{ color: "var(--sea-ink)" }}>
          Description
        </label>
        <Textarea
          placeholder="A reusable pattern that adds..."
          rows={3}
          value={data.description}
          onChange={(e) => setData((d) => ({ ...d, description: e.target.value }))}
        />
        {errors.description && <p className="mt-1 text-xs text-red-500">{errors.description}</p>}
      </div>

      <div className="flex justify-end">
        <Button type="submit">
          Next <ArrowRight className="ml-1 size-4" />
        </Button>
      </div>
    </form>
  )
}

function Step2({ onNext, onBack, initialManifest }: { onNext: (d: Step2Data) => void; onBack: () => void; initialManifest?: string }) {
  const [manifest, setManifest] = useState(initialManifest ?? "")
  const [tarball, setTarball] = useState<File | null>(null)
  const [manifestError, setManifestError] = useState("")
  const fileRef = useRef<HTMLInputElement>(null)

  function validateManifest(val: string) {
    try { JSON.parse(val); setManifestError("") } catch { setManifestError("Invalid JSON.") }
  }

  function submit(e: React.FormEvent) {
    e.preventDefault()
    if (manifestError || !manifest) { setManifestError("Paste a valid manifest.json"); return }
    if (!tarball) { toast.error("Select a .tar.gz tarball"); return }
    onNext({ manifestJson: manifest, tarball })
  }

  return (
    <form onSubmit={submit} className="space-y-5">
      <div>
        <label className="mb-1.5 block text-sm font-medium" style={{ color: "var(--sea-ink)" }}>
          manifest.json
        </label>
        <Textarea
          rows={12}
          className="font-mono text-xs"
          placeholder={'{\n  "id": "my-pattern",\n  "name": "My Pattern",\n  ...\n}'}
          value={manifest}
          onChange={(e) => { setManifest(e.target.value); validateManifest(e.target.value) }}
        />
        {manifestError && <p className="mt-1 text-xs text-red-500">{manifestError}</p>}
      </div>

      <div>
        <label className="mb-1.5 block text-sm font-medium" style={{ color: "var(--sea-ink)" }}>
          Tarball (.tar.gz)
        </label>
        <div
          className="flex cursor-pointer flex-col items-center gap-2 rounded-xl border-2 border-dashed py-8 transition-colors hover:border-[var(--lagoon)]"
          style={{ borderColor: tarball ? "var(--lagoon)" : "var(--line)" }}
          onClick={() => fileRef.current?.click()}
        >
          <Upload className="size-6" style={{ color: tarball ? "var(--lagoon)" : "var(--sea-ink-soft)" }} />
          <span className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            {tarball ? tarball.name : "Click to select a .tar.gz file"}
          </span>
        </div>
        <input
          ref={fileRef}
          type="file"
          accept=".tar.gz,.tgz"
          className="hidden"
          onChange={(e) => setTarball(e.target.files?.[0] ?? null)}
        />
      </div>

      <div className="flex justify-between">
        <Button type="button" variant="outline" onClick={onBack}>
          <ArrowLeft className="mr-1 size-4" /> Back
        </Button>
        <Button type="submit">
          Next <ArrowRight className="ml-1 size-4" />
        </Button>
      </div>
    </form>
  )
}

function Step3({
  step1,
  step2,
  onNext,
  onBack,
}: {
  step1: Step1Data
  step2: Step2Data
  onNext: () => void
  onBack: () => void
}) {
  let parsed: Record<string, unknown> = {}
  try { parsed = JSON.parse(step2.manifestJson) } catch {}

  const fields = [
    { label: "id", value: parsed.id as string },
    { label: "lang", value: parsed.lang as string },
    { label: "runtime", value: parsed.runtime as string },
    { label: "tsx_min", value: parsed.tsx_min as string },
    { label: "provides", value: Array.isArray(parsed.provides) ? (parsed.provides as string[]).join(", ") : "—" },
  ]

  return (
    <div className="space-y-5">
      <div className="island-shell rounded-xl p-5">
        <p className="island-kicker mb-3">Package</p>
        <p className="font-mono text-lg font-bold" style={{ color: "var(--sea-ink)" }}>
          @tsx-pkg/{step1.name} <span className="text-sm font-normal" style={{ color: "var(--sea-ink-soft)" }}>v{step1.version}</span>
        </p>
        <p className="mt-1 text-sm" style={{ color: "var(--sea-ink-soft)" }}>{step1.description}</p>
      </div>

      <div className="island-shell rounded-xl p-5">
        <p className="island-kicker mb-3">Manifest fields</p>
        <div className="space-y-2 text-sm">
          {fields.map(({ label, value }) => (
            <div key={label} className="flex gap-3">
              <span className="w-24 shrink-0 font-mono text-xs" style={{ color: "var(--sea-ink-soft)" }}>{label}</span>
              <span style={{ color: "var(--sea-ink)" }}>{value || "—"}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="island-shell rounded-xl p-5">
        <p className="island-kicker mb-2">Tarball</p>
        <p className="font-mono text-sm" style={{ color: "var(--sea-ink)" }}>{step2.tarball?.name}</p>
        <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>
          {step2.tarball ? `${(step2.tarball.size / 1024).toFixed(1)} KB` : ""}
        </p>
      </div>

      <div className="flex justify-between">
        <Button type="button" variant="outline" onClick={onBack}>
          <ArrowLeft className="mr-1 size-4" /> Back
        </Button>
        <Button onClick={onNext}>
          Publish <ArrowRight className="ml-1 size-4" />
        </Button>
      </div>
    </div>
  )
}

function Step4({ step1, step2, onBack, onPublished }: { step1: Step1Data; step2: Step2Data; onBack: () => void; onPublished: () => void }) {
  const [loading, setLoading] = useState(false)
  const [done, setDone] = useState(false)

  async function publish() {
    setLoading(true)
    try {
      const form = new FormData()
      form.append("name", step1.name)
      form.append("version", step1.version)
      form.append("description", step1.description)
      form.append("manifest", new Blob([step2.manifestJson], { type: "application/json" }), "manifest.json")
      if (step2.tarball) form.append("tarball", step2.tarball)

      const apiKey = import.meta.env.VITE_REGISTRY_API_KEY ?? ""
      const base = import.meta.env.VITE_REGISTRY_URL ?? "http://localhost:8080"
      const res = await fetch(`${base}/v1/packages/publish`, {
        method: "POST",
        headers: { Authorization: `Bearer ${apiKey}` },
        body: form,
      })

      if (!res.ok) {
        const msg = await res.text()
        throw new Error(msg || `HTTP ${res.status}`)
      }

      setDone(true)
      onPublished()
      toast.success("Package published!")
    } catch (err: unknown) {
      toast.error(err instanceof Error ? err.message : "Publish failed")
    } finally {
      setLoading(false)
    }
  }

  if (done) {
    return (
      <div className="space-y-6 text-center">
        <div className="mx-auto flex size-16 items-center justify-center rounded-full" style={{ background: "var(--lagoon)" }}>
          <Check className="size-8 text-white" />
        </div>
        <div>
          <h2 className="text-xl font-bold" style={{ color: "var(--sea-ink)" }}>Published!</h2>
          <p className="mt-1 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Your package is live on the registry.
          </p>
        </div>
        <div className="island-shell mx-auto max-w-sm rounded-xl p-4">
          <p className="island-kicker mb-2">Install command</p>
          <code className="text-sm" style={{ color: "var(--sea-ink)" }}>tsx install {step1.name}</code>
        </div>
        <div className="flex justify-center gap-3">
          <Button asChild>
            <Link to="/packages/$name" params={{ name: step1.name }}>View package</Link>
          </Button>
          <Button variant="outline" asChild>
            <Link to="/dashboard">Dashboard</Link>
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-5">
      <div className="island-shell rounded-xl p-5">
        <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          You're about to publish{" "}
          <strong style={{ color: "var(--sea-ink)" }}>@tsx-pkg/{step1.name} v{step1.version}</strong>{" "}
          to the registry. This action cannot be undone.
        </p>
      </div>

      <div className="flex justify-between">
        <Button type="button" variant="outline" onClick={onBack} disabled={loading}>
          <ArrowLeft className="mr-1 size-4" /> Back
        </Button>
        <Button onClick={publish} disabled={loading}>
          {loading ? (
            <span className="flex items-center gap-2">
              <span className="size-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
              Publishing…
            </span>
          ) : (
            "Confirm & publish"
          )}
        </Button>
      </div>
    </div>
  )
}

function PublishPage() {
  const [step, setStep] = useState<Step>(1)

  // Load draft from localStorage on mount
  const [step1Data, setStep1Data] = useState<Step1Data>(() => {
    try {
      const raw = localStorage.getItem(DRAFT_KEY)
      if (raw) return (JSON.parse(raw) as PublishDraft).step1
    } catch {}
    return { name: "", version: "", description: "" }
  })
  const [step2Data, setStep2Data] = useState<Step2Data>(() => {
    try {
      const raw = localStorage.getItem(DRAFT_KEY)
      if (raw) return { manifestJson: (JSON.parse(raw) as PublishDraft).manifestJson, tarball: null }
    } catch {}
    return { manifestJson: "", tarball: null }
  })

  // Persist text fields to localStorage whenever they change
  useEffect(() => {
    try {
      const draft: PublishDraft = { step1: step1Data, manifestJson: step2Data.manifestJson }
      localStorage.setItem(DRAFT_KEY, JSON.stringify(draft))
    } catch {}
  }, [step1Data, step2Data.manifestJson])

  function clearDraft() {
    try { localStorage.removeItem(DRAFT_KEY) } catch {}
  }

  return (
    <div className="page-wrap py-12 rise-in">
      <div className="mb-8">
        <h1 className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Publish a package</h1>
        <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
          Share a reusable pattern with the community.
        </p>
      </div>

      <div className="mx-auto max-w-xl">
        <StepIndicator current={step} />

        <div className="island-shell rounded-xl p-6">
          <h2 className="mb-5 font-semibold" style={{ color: "var(--sea-ink)" }}>
            Step {step} — {stepLabel(step)}
          </h2>

          {step === 1 && (
            <Step1
              initialData={step1Data}
              onNext={(d) => { setStep1Data(d); setStep(2) }}
            />
          )}
          {step === 2 && (
            <Step2
              initialManifest={step2Data.manifestJson}
              onNext={(d) => { setStep2Data(d); setStep(3) }}
              onBack={() => setStep(1)}
            />
          )}
          {step === 3 && (
            <Step3
              step1={step1Data}
              step2={step2Data}
              onNext={() => setStep(4)}
              onBack={() => setStep(2)}
            />
          )}
          {step === 4 && (
            <Step4
              step1={step1Data}
              step2={step2Data}
              onBack={() => setStep(3)}
              onPublished={clearDraft}
            />
          )}
        </div>
      </div>
    </div>
  )
}
