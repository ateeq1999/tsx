import { useNavigate } from "@tanstack/react-router"

interface Props {
  packageName: string
  integratesWith: string[]
}

/** Truncate `@tsx-pkg/drizzle-pg` → `drizzle-pg`, and cap at 12 chars */
function shortLabel(name: string) {
  const base = name.includes("/") ? name.split("/").pop()! : name
  return base.length > 12 ? base.slice(0, 11) + "…" : base
}

const W = 600
const H = 320
const CX = W / 2
const CY = H / 2
const SPOKE_R = 130   // distance from center to spoke nodes
const CENTER_R = 38
const NODE_R = 30

export function DependencyGraph({ packageName, integratesWith }: Props) {
  const navigate = useNavigate()

  if (integratesWith.length === 0) {
    return (
      <div
        className="flex h-40 items-center justify-center rounded-xl text-sm"
        style={{ color: "var(--sea-ink-soft)", background: "var(--code-bg)" }}
      >
        No integration links declared for this package.
      </div>
    )
  }

  const nodes = integratesWith.map((name, i) => {
    const angle = (2 * Math.PI * i) / integratesWith.length - Math.PI / 2
    return {
      name,
      label: shortLabel(name),
      x: CX + SPOKE_R * Math.cos(angle),
      y: CY + SPOKE_R * Math.sin(angle),
    }
  })

  function goTo(name: string) {
    navigate({ to: "/packages/$name", params: { name } })
  }

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full"
      style={{ maxHeight: 320 }}
      aria-label={`Integration graph for ${packageName}`}
    >
      {/* spoke lines */}
      {nodes.map((n) => (
        <line
          key={n.name}
          x1={CX} y1={CY} x2={n.x} y2={n.y}
          stroke="var(--line)"
          strokeWidth={1.5}
          strokeDasharray="4 3"
        />
      ))}

      {/* spoke nodes */}
      {nodes.map((n) => (
        <g
          key={n.name}
          onClick={() => goTo(n.name)}
          style={{ cursor: "pointer" }}
          role="button"
          aria-label={`Go to ${n.name}`}
        >
          <circle
            cx={n.x} cy={n.y} r={NODE_R}
            fill="var(--surface-strong)"
            stroke="var(--line)"
            strokeWidth={1.5}
          />
          <text
            x={n.x} y={n.y}
            textAnchor="middle"
            dominantBaseline="middle"
            fontSize={9}
            fontFamily="monospace"
            fill="var(--sea-ink)"
          >
            {n.label}
          </text>

          {/* full name tooltip on hover via title */}
          <title>{n.name}</title>
        </g>
      ))}

      {/* center node — current package */}
      <circle
        cx={CX} cy={CY} r={CENTER_R}
        fill="var(--lagoon)"
      />
      <text
        x={CX} y={CY - 6}
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize={9}
        fontFamily="monospace"
        fontWeight="bold"
        fill="#fff"
      >
        {shortLabel(packageName)}
      </text>
      <text
        x={CX} y={CY + 7}
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize={7.5}
        fontFamily="monospace"
        fill="rgba(255,255,255,0.7)"
      >
        (this pkg)
      </text>
    </svg>
  )
}
