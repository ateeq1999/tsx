# TSX Framework Protocol

### Universal AI Agent Integration Layer for Frameworks & Packages

**Technical Proposal — Version 1.0 · March 2026**

> Framework Registry · Convention Protocol · Agent Learning Interface

---

## 1. Executive Summary

TSX should evolve from a TanStack Start-specific code generator into a **universal framework bootstrapping protocol**. Framework developers (React, Vue, Svelte, Solid, etc.) and package authors (libraries, SDKs, tools) can register their frameworks with TSX to provide AI agents with:

1. **Where** — Canonical file locations and project structure
2. **What** — Code templates for integration patterns
3. **How** — Injection points for user custom code
4. **Dependencies** — Required packages and configurations

AI agents use TSX as a **conversation partner** to learn any framework, not just generate code — but also understand conventions, patterns, and best practices.

---

## 2. Core Concepts

### 2.1 Framework Registry

Each framework registers via a `tsx-registry.json` file in the framework's docs or package:

```json
{
  "framework": "TanStack Start",
  "version": "1.0",
  "slug": "tanstack-start",
  "docs": "https://tanstack.com/start",
  "structure": {
    "routes": "routes/",
    "components": "components/",
    "server_functions": "server-functions/",
    "lib": "lib/"
  },
  "generators": [...],
  "conventions": [...],
  "injection_points": [...]
}
```

### 2.2 Convention Protocol

Frameworks define their file structure and naming conventions:

```json
{
  "conventions": {
    "routes": {
      "pattern": "routes/{path}.tsx",
      "loader": "loader",
      "component": "Component"
    },
    "components": {
      "pattern": "components/{name}/{name}.tsx",
      "index": "index.ts"
    },
    "api": {
      "pattern": "server-functions/{name}.ts"
    }
  }
}
```

### 2.3 Injection Points

Templates define where developers can add custom code:

```tsx
// <INJECT:imports>
// Your imports here
// </INJECT>

// <INJECT:state>
// Your state here
// </INJECT>

// <INJECT:handlers>
// Your handlers here
// </INJECT>
```

TSX preserves these regions when regenerating files.

---

## 3. Feature Specifications

### 3.1 Framework Discovery

Agents can discover registered frameworks:

```bash
tsx list --frameworks
```

```json
{
  "success": true,
  "frameworks": [
    {
      "slug": "tanstack-start",
      "name": "TanStack Start",
      "version": "1.0.0",
      "description": "Type-safe full-stack React framework",
      "docs": "https://tanstack.com/start",
      "category": "framework"
    },
    {
      "slug": " drizzle-orm",
      "name": "Drizzle ORM",
      "version": "0.30.0",
      "description": "Type-safe SQL with zero runtime",
      "docs": "https://orm.drizzle.team",
      "category": "orm"
    },
    {
      "slug": "better-auth",
      "name": "Better Auth",
      "version": "1.0.0",
      "description": "Authentication for modern web apps",
      "docs": "https://better-auth.com",
      "category": "auth"
    }
  ]
}
```

### 3.2 Framework Introspection

Ask questions about a framework:

```bash
tsx ask tanstack-start --question "How do I add a new route with authentication?"
```

```json
{
  "success": true,
  "answer": {
    "topic": "authentication",
    "steps": [
      {
        "action": "Create server function with auth guard",
        "code": "tsx add:server-fn --json '{...}'"
      },
      {
        "action": "Add auth guard to route",
        "code": "tsx add:auth-guard --json '{...}'"
      }
    ],
    "files_affected": ["server-functions/{name}.ts", "routes/{path}.tsx"],
    "dependencies": ["@tanstack/start", "better-auth"]
  }
}
```

### 3.3 Convention Query

Ask about where files should go:

```bash
tsx where tanstack-start --thing "api-endpoint"
```

```json
{
  "success": true,
  "location": {
    "path": "server-functions/{name}.ts",
    "pattern": "server-functions/{name}.ts",
    "example": "server-functions/users/get.ts"
  },
  "conventions": {
    "naming": "kebab-case",
    "export": "named export",
    "file_structure": "..."
  }
}
```

### 3.4 Integration Pattern Query

Ask how to integrate a package:

```bash
tsx how tanstack-start --integrate "drizzle-orm"
```

```json
{
  "success": true,
  "integration": {
    "package": "drizzle-orm",
    "install": "npm install drizzle-orm better-sqlite3",
    "setup": [
      {
        "file": "db/schema.ts",
        "template": "drizzle/schema"
      },
      {
        "file": "drizzle.config.ts",
        "template": "drizzle/config"
      }
    ],
    "patterns": [
      {
        "name": "define-table",
        "location": "db/schema/*.ts",
        "example": "..."
      },
      {
        "name": "query-builder",
        "location": "lib/db.ts",
        "example": "..."
      }
    ]
  }
}
```

### 3.5 Template Customization

Package developers can add their own templates to existing generators:

```json
{
  "extends": "tsx:add:form",
  "add": {
    "fields": [
      {
        "name": "stripeCustomerId",
        "type": "string",
        "template": "stripe/customer-id-field"
      }
    ]
  }
}
```

### 3.6 Project Template Discovery

Frameworks can publish project templates:

```bash
tsx list --templates tanstack-start
```

```json
{
  "success": true,
  "templates": [
    {
      "id": "full-stack",
      "name": "Full Stack App",
      "description": "Complete app with auth, DB, routing",
      "files": ["..."],
      "options": {
        "auth": ["none", "better-auth", "clerk"],
        "db": ["sqlite", "postgres", "mysql"]
      }
    },
    {
      "id": "api-only",
      "name": "API Only",
      "description": "TanStack Start as API backend",
      "files": ["..."]
    }
  ]
}
```

### 3.7 Learning Mode

TSX can explain its decisions to help agents learn:

```bash
tsx explain tanstack-start --template "add:schema"
```

```json
{
  "success": true,
  "explanation": {
    "template": "add:schema",
    "purpose": "Generate Drizzle table definition",
    "decisions": [
      {
        "file": "db/schema/{name}.ts",
        "reason": "Drizzle convention: schemas in db/schema/"
      },
      {
        "field": "id",
        "reason": "Every table needs a primary key; UUID is preferred for distributed systems"
      },
      {
        "field": "timestamps",
        "reason": "Best practice: track created/updated for debugging"
      }
    ],
    "learn": [
      "Why UUID over auto-increment?",
      "Why timestamps?",
      "Why relations separate?"
    ]
  }
}
```

---

## 4. Architecture Changes

### 4.1 New Directory Structure

```
crates/tsx/
  src/
    framework/           # Framework registry and discovery
      registry.rs
      loader.rs
      conventions.rs
    ask/                # AI Q&A interface
      ask.rs
      context.rs
    where/              # File location queries
      where.rs
    how/                # Integration how-tos
      how.rs
    explain/            # Learning mode
      explain.rs
  frameworks/           # Built-in framework definitions
    tanstack-start/
      registry.json
      conventions.json
      templates/
    drizzle-orm/
      registry.json
      conventions.json
    better-auth/
      registry.json
```

### 4.2 Registry File Format

```json
{
  "framework": "Framework Name",
  "version": "1.0.0",
  "slug": "framework-slug",
  "category": "framework|orm|auth|ui|tool",
  "docs": "https://docs.example.com",
  "github": "https://github.com/org/repo",
  "structure": {
    "root": ".",
    "src": "src",
    "routes": "routes",
    "components": "components",
    "lib": "lib",
    "config": "."
  },
  "generators": [
    {
      "id": "add:feature",
      "description": "...",
      "options": {...}
    }
  ],
  "conventions": {
    "files": {...},
    "naming": {...},
    "patterns": [...]
  },
  "injection_points": [
    {
      "region": "imports",
      "marker": "// <INJECT:imports>",
      "end_marker": "// </INJECT>"
    }
  ],
  "integrations": [
    {
      "package": "drizzle-orm",
      "setup": [...],
      "patterns": [...]
    }
  ],
  "questions": [
    {
      "topic": "authentication",
      "answer": "..."
    }
  ]
}
```

---

## 5. Use Cases

### 5.1 Framework Onboarding

A new developer joins a team using TanStack Start + Drizzle + Better Auth:

1. `tsx ask tanstack-start --question "How do I add a new feature?"`
2. `tsx where tanstack-start --thing "database-model"`
3. `tsx how tanstack-start --integrate "analytics"`
4. Agent learns conventions without reading docs

### 5.2 Package Integration

Maintainer of `stripe-js` wants to help agents integrate their package:

1. Create `stripe-js/registry.json` with integration patterns
2. Publish to npm
3. Agents discover via `tsx how <framework> --integrate "stripe-js"`

### 5.3 Team Conventions

Company wants to enforce internal patterns:

1. Create company-registry with custom templates
2. Agents use company-registry instead of defaults
3. All generated code follows company standards

---

## 6. Implementation Plan

### Phase 1 — Foundation

- [ ] Create `src/framework/` module
- [ ] Implement registry loader (local + npm)
- [ ] Build framework discovery command
- [ ] Create TanStack Start registry definition

### Phase 2 — Query Interface

- [ ] Implement `ask` command with Q&A
- [ ] Implement `where` command for file locations
- [ ] Implement `how` command for integrations
- [ ] Add Drizzle and Better Auth registries

### Phase 3 — Learning

- [ ] Implement `explain` command
- [ ] Add decision explanations to templates
- [ ] Build question/answer knowledge base
- [ ] Add more framework registries

### Phase 4 — Ecosystem

- [ ] Registry publishing to npm
- [ ] Framework registry website
- [ ] Template marketplace
- [ ] Community contributions

---

## 7. Success Metrics

| Metric | Target |
|---|---|
| Framework registries | 20+ popular frameworks |
| Question accuracy | >95% helpful answers |
| Agent adoption | Major AI agents use TSX protocol |
| Integration patterns | 100+ package integrations |

---

## 8. Conclusion

TSX as a Framework Protocol transforms it from a code generator into an **AI agent development partner**. Framework developers no longer need to write agent prompts — they register their conventions once, and all AI agents instantly understand how to build with their framework.

The key insight: **code generation is the side effect, learning is the product**. Agents that understand your framework are more likely to generate correct code, follow best practices, and help developers succeed.
