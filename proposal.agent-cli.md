# Atom Forge CLI Agent Support Proposal

**Status**: Draft  
**Target**: forge-cli v0.2.0  
**Author**: Atom Forge Team  

---

## Executive Summary

This proposal outlines a comprehensive plan to transform the Atom Forge CLI into an **agent-friendly** tool that can be programmatically controlled by AI coding agents. The primary goal is to enable AI agents (such as Claude, GPT-4, or custom agents) to autonomously generate code, scaffold projects, and manage Atom Forge applications using a structured JSON interface.

---

## Motivation

Modern AI coding agents excel at understanding natural language and generating code, but they struggle with traditional CLI tools that:

1. **Emit human-centric output** — Agents must parse unstructured text to extract meaningful data.
2. **Lack idempotent operations** — Agents cannot reliably determine if a command was successful.
3. **Provide no structured introspection** — Agents cannot discover available templates, components, or options programmatically.
4. **Require interactive input** — Agents cannot provide stdin input in most tool-use contexts.

By adding first-class JSON support, Atom Forge CLI becomes a powerful building block for AI-driven development workflows.

---

## Core Design Principles

1. **JSON-first input and output** — All commands can be invoked via JSON payload, with JSON responses.
2. **Backward compatibility** — Existing CLI arguments continue to work; JSON mode is opt-in.
3. **Structured errors** — All errors include machine-parseable codes and messages.
4. **Introspection-first** — Agents can discover capabilities before using them.
5. **Idempotent by design** — Commands report whether changes were made.

---

## Feature Specifications

### 1. JSON Input Mode

#### Activation

The CLI accepts JSON input via two mechanisms:

| Flag | Description |
|------|-------------|
| `--json` | Parse remaining arguments as JSON |
| `--stdin` | Read the entire command payload from stdin |
| `--file <path>` | Read command payload from a file |

#### Command Payload Structure

```json
{
  "version": "1.0",
  "command": "generate",
  "options": {
    "kind": "screen",
    "name": "users/list",
    "force": false,
    "dryRun": false
  }
}
```

#### Example Invocation

```bash
# Via stdin
echo '{"command": "generate", "options": {"kind": "screen", "name": "dashboard"}}' | forge --stdin

# Via file
forge --file generate-screen.json

# Via flag (single command)
forge --json generate screen dashboard
```

---

### 2. Structured JSON Output

All output is returned as JSON with a consistent envelope:

```json
{
  "success": true,
  "version": "1.0",
  "command": "generate",
  "result": {
    "kind": "screen",
    "path": "screens/dashboard.atom",
    "generated": true,
    "template": "default"
  },
  "metadata": {
    "timestamp": "2026-03-15T10:30:00Z",
    "duration_ms": 45
  }
}
```

#### Verbose Mode (`--verbose`)

Adds additional context to the response:

```json
{
  "success": true,
  "version": "1.0",
  "command": "generate",
  "result": {
    "kind": "screen",
    "path": "screens/dashboard.atom",
    "generated": true,
    "template": "default"
  },
  "metadata": {
    "timestamp": "2026-03-15T10:30:00Z",
    "duration_ms": 45,
    "warnings": ["Template 'default' is deprecated, consider using 'minimal'"]
  },
  "context": {
    "project_root": "/path/to/project",
    "forge_version": "0.1.0"
  }
}
```

---

### 3. Rich Specification for Complex Generations

Instead of simple string arguments, agents can provide full specifications:

#### Form Generation with Fields

```json
{
  "command": "generate",
  "options": {
    "kind": "form",
    "name": "user_profile",
    "spec": {
      "fields": [
        {
          "name": "email",
          "type": "string",
          "required": true,
          "validate": {
            "email": true,
            "maxLength": 255
          }
        },
        {
          "name": "age",
          "type": "i32",
          "required": false,
          "validate": {
            "min": 13,
            "max": 120
          }
        },
        {
          "name": "role",
          "type": "enum",
          "required": true,
          "validate": {
            "oneOf": ["admin", "user", "guest"]
          }
        },
        {
          "name": "bio",
          "type": "string",
          "required": false,
          "validate": {
            "minLength": 0,
            "maxLength": 500
          }
        }
      ]
    }
  }
}
```

**Generated Output** (`src/forms/user_profile.rs`):

```rust
//! UserProfile form

use forge_runtime::prelude::*;

#[forge_form]
pub struct UserProfileForm {
    #[validate(required, email, max_length = 255)]
    pub email: String,

    #[validate(min = 13, max = 120)]
    pub age: Option<i32>,

    #[validate(required, one_of = ["admin", "user", "guest"])]
    pub role: String,

    #[validate(max_length = 500)]
    pub bio: Option<String>,
}
```

#### Store Generation with State and Actions

```json
{
  "command": "generate",
  "options": {
    "kind": "store",
    "name": "cart",
    "spec": {
      "state": [
        {"name": "items", "type": "Vec<CartItem>"},
        {"name": "total", "type": "f64"},
        {"name": "is_loading", "type": "bool"}
      ],
      "actions": [
        {"name": "add_item", "params": [{"name": "item", "type": "CartItem"}]},
        {"name": "remove_item", "params": [{"name": "id", "type": "String"}]},
        {"name": "clear", "async": true},
        {"name": "sync", "async": true, "returns": "Result<(), ApiError>"}
      ],
      "computed": [
        {"name": "item_count", "type": "usize", "expression": "self.items.len()"},
        {"name": "is_empty", "type": "bool", "expression": "self.items.is_empty()"}
      ]
    }
  }
}
```

---

### 4. Dry-Run Mode

Agents can preview changes without modifying the filesystem:

```json
{
  "command": "generate",
  "options": {
    "kind": "screen",
    "name": "users/list",
    "dryRun": true
  }
}
```

**Response**:

```json
{
  "success": true,
  "dryRun": true,
  "result": {
    "wouldGenerate": true,
    "path": "screens/users/list.atom",
    "content": "{{!-- screens/users/list.atom --}}\n@vstack(class: \"p-6 gap-4\") {\n  @text(\"UsersList\", class: \"text-2xl font-bold\")\n}\n",
    "overwrites": false
  }
}
```

---

### 5. Introspection Commands

#### List Templates

```json
{
  "command": "list",
  "options": {
    "kind": "templates"
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "templates": [
      {
        "id": "default",
        "name": "Default",
        "description": "Full application with auth, DB, and routing",
        "path": "templates/default",
        "files": ["src/main.rs", "screens/", "components/", "src/stores/"]
      },
      {
        "id": "minimal",
        "name": "Minimal",
        "description": "Minimal boilerplate to get started",
        "path": "templates/minimal",
        "files": ["src/main.rs", "screens/index.atom"]
      },
      {
        "id": "with-db",
        "name": "With Database",
        "description": "Pre-configured SQLite setup",
        "path": "templates/with-db",
        "files": ["src/main.rs", "src/db/", "forge.toml"]
      }
    ]
  }
}
```

#### List Components

```json
{
  "command": "list",
  "options": {
    "kind": "components",
    "category": "inputs"
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "components": [
      {
        "name": "button",
        "category": "inputs",
        "description": "Interactive button with multiple variants",
        "props": {
          "variant": { "type": "enum", "values": ["primary", "secondary", "ghost", "destructive"], "default": "primary" },
          "size": { "type": "enum", "values": ["sm", "md", "lg"], "default": "md" },
          "disabled": { "type": "bool", "default": false },
          "onclick": { "type": "callback" }
        },
        "file": "components/button.atom"
      },
      {
        "name": "input",
        "category": "inputs",
        "description": "Text input field with validation support",
        "props": {
          "type": { "type": "enum", "values": ["text", "email", "password", "number"], "default": "text" },
          "placeholder": { "type": "string" },
          "value": { "type": "string" },
          "onchange": { "type": "callback" }
        },
        "file": "components/input.atom"
      }
    ]
  }
}
```

#### List Generators

```json
{
  "command": "list",
  "options": {
    "kind": "generators"
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "generators": [
      {
        "id": "screen",
        "description": "Scaffold a new .atom screen file",
        "options": {
          "name": { "type": "string", "required": true, "pattern": "^[a-z0-9/-]+$" },
          "template": { "type": "string", "default": "default" },
          "force": { "type": "bool", "default": false }
        }
      },
      {
        "id": "store",
        "description": "Scaffold a new #[forge_store] module",
        "options": {
          "name": { "type": "string", "required": true },
          "spec": { "type": "object", "description": "Full store specification" }
        }
      },
      {
        "id": "form",
        "description": "Scaffold a new #[forge_form] module",
        "options": {
          "name": { "type": "string", "required": true },
          "spec": { "type": "object", "description": "Full form specification with fields" }
        }
      },
      {
        "id": "component",
        "description": "Scaffold a new .atom component",
        "options": {
          "name": { "type": "string", "required": true }
        }
      },
      {
        "id": "query",
        "description": "Scaffold a new #[forge_query] module",
        "options": {
          "name": { "type": "string", "required": true }
        }
      },
      {
        "id": "migration",
        "description": "Scaffold an empty migration file",
        "options": {
          "name": { "type": "string", "required": true }
        }
      },
      {
        "id": "i18n",
        "description": "Scaffold a new locale translation file",
        "options": {
          "locale": { "type": "string", "required": true }
        }
      }
    ]
  }
}
```

---

### 6. Batch Operations

Execute multiple commands in a single invocation:

```json
{
  "command": "batch",
  "options": {
    "commands": [
      {"command": "generate", "options": {"kind": "screen", "name": "dashboard"}},
      {"command": "generate", "options": {"kind": "store", "name": "user", "spec": {"state": [{"name": "name", "type": "String"}]}}},
      {"command": "generate", "options": {"kind": "component", "name": "header"}},
      {"command": "generate", "options": {"kind": "form", "name": "login", "spec": {"fields": [{"name": "email", "type": "string", "required": true}, {"name": "password", "type": "string", "required": true}]}}}
    ]
  }
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "total": 4,
    "succeeded": 4,
    "failed": 0,
    "results": [
      {
        "index": 0,
        "success": true,
        "result": {"kind": "screen", "path": "screens/dashboard.atom"}
      },
      {
        "index": 1,
        "success": true,
        "result": {"kind": "store", "path": "src/stores/user_store.rs"}
      },
      {
        "index": 2,
        "success": true,
        "result": {"kind": "component", "path": "components/header.atom"}
      },
      {
        "index": 3,
        "success": true,
        "result": {"kind": "form", "path": "src/forms/login.rs"}
      }
    ]
  }
}
```

**Failure Handling**:

```json
{
  "success": false,
  "result": {
    "total": 3,
    "succeeded": 2,
    "failed": 1,
    "results": [
      {
        "index": 0,
        "success": true,
        "result": {"kind": "screen", "path": "screens/dashboard.atom"}
      },
      {
        "index": 1,
        "success": false,
        "error": {
          "code": "FILE_EXISTS",
          "message": "Store 'user' already exists at src/stores/user_store.rs",
          "path": "src/stores/user_store.rs"
        }
      },
      {
        "index": 2,
        "success": true,
        "result": {"kind": "component", "path": "components/header.atom"}
      }
    ]
  }
}
```

---

### 7. Structured Error Format

All errors follow a consistent structure:

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid command payload",
    "details": [
      {
        "field": "options.name",
        "message": "Name must match pattern ^[a-z0-9/-]+$"
      }
    ]
  }
}
```

#### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_PAYLOAD` | JSON payload is malformed |
| `VALIDATION_ERROR` | Payload fails schema validation |
| `UNKNOWN_COMMAND` | Command is not recognized |
| `UNKNOWN_KIND` | Generator or list kind is not recognized |
| `FILE_EXISTS` | Target file already exists (use `force: true`) |
| `DIRECTORY_NOT_FOUND` | Required parent directory does not exist |
| `PERMISSION_DENIED` | Cannot write to target location |
| `TEMPLATE_NOT_FOUND` | Specified template does not exist |
| `PROJECT_NOT_FOUND` | Not running inside an Atom Forge project |
| `INTERNAL_ERROR` | Unexpected error in CLI |

---

### 8. Dev Server JSON Events

For `dev` mode, the CLI can emit file change events as JSON:

```json
{
  "event": "file_changed",
  "timestamp": "2026-03-15T10:30:00Z",
  "data": {
    "kind": "modified",
    "path": "screens/dashboard.atom",
    "screen": "dashboard"
  }
}
```

**Event Types**:

| Event | Description |
|-------|-------------|
| `started` | Dev server has started |
| `file_changed` | A file was modified |
| `file_added` | A new file was created |
| `file_deleted` | A file was removed |
| `build_started` | Build process started |
| `build_completed` | Build completed successfully |
| `build_failed` | Build failed with errors |
| `hot_reload` | Hot reload triggered |
| `error` | Server encountered an error |
| `stopped` | Dev server stopped |

**Invocation**:

```bash
forge dev --json-events
```

---

### 9. Force Mode

Overwrite existing files without prompting:

```json
{
  "command": "generate",
  "options": {
    "kind": "screen",
    "name": "dashboard",
    "force": true
  }
}
```

---

### 10. Project Inspection

Agents can query the current project state:

```json
{
  "command": "inspect"
}
```

**Response**:

```json
{
  "success": true,
  "result": {
    "project_root": "/path/to/my-app",
    "forge_version": "0.1.0",
    "app_name": "My App",
    "app_id": "com.example.myapp",
    "structure": {
      "screens": ["index", "dashboard", "users/list"],
      "components": ["button", "input", "card"],
      "stores": ["user_store", "cart_store"],
      "forms": ["login_form"],
      "queries": ["get_users"],
      "migrations": ["20260315_create_users", "20260315_add_email"],
      "i18n": ["en", "es", "fr"]
    },
    "database": {
      "type": "sqlite",
      "url": "sqlite://app.db",
      "migrations_pending": 0
    },
    "config": {
      "window_width": 1280,
      "window_height": 800,
      "theme": "default"
    }
  }
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (v0.2.0-alpha)

- [ ] Add `--json`, `--stdin`, `--file` flags
- [ ] Implement JSON payload parsing
- [ ] Add structured JSON response envelope
- [ ] Implement basic error handling with error codes
- [ ] Add `--dry-run` flag to all generators

### Phase 2: Rich Specifications (v0.2.0-beta)

- [ ] Extend form generator to parse field specifications
- [ ] Extend store generator to parse state/actions/computed
- [ ] Add `--force` flag support
- [ ] Implement batch command execution

### Phase 3: Introspection (v0.2.0)

- [ ] Implement `list templates` command
- [ ] Implement `list components` command with metadata
- [ ] Implement `list generators` command with option schemas
- [ ] Implement `inspect` command for project state

### Phase 4: Events & Streaming (v0.3.0)

- [ ] Add `--json-events` flag to dev mode
- [ ] Implement structured event emission
- [ ] Add event subscription for external tools

---

## File Structure Changes

```
crates/forge-cli/
├── src/
│   ├── main.rs           # CLI entry point (mostly stubs)
│   ├── config.rs         # forge.toml schema
│   ├── json/
│   │   ├── mod.rs        # JSON mode orchestration
│   │   ├── payload.rs    # Command payload structures
│   │   ├── response.rs   # Response envelope
│   │   ├── error.rs      # Error types and codes
│   │   └── events.rs     # Dev server events
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── generate.rs   # Extended generate logic
│   │   ├── list.rs       # List command
│   │   ├── inspect.rs    # Inspect command
│   │   ├── batch.rs      # Batch execution
│   │   └── dev.rs        # Dev server with events
│   └── templates/         # Embedded template metadata
│       └── metadata.json
```

---

## Backward Compatibility

All changes are additive and backward compatible:

1. **CLI args work as before** — No existing commands change behavior.
2. **Human output preserved** — Without `--json` flag, CLI emits human-friendly text.
3. **Exit codes** — Zero for success, non-zero for failure (as before).
4. **Error messages** — Human-readable errors still printed to stderr in non-JSON mode.

---

## Example Agent Workflow

```python
# Example: AI agent builds a user management feature

# 1. Discover available generators
result = run_forge(["--json", "list", "--kind", "generators"])

# 2. Inspect current project
result = run_forge(["--json", "inspect"])

# 3. Generate screen, store, and form in batch
result = run_forge(["--json"], input={
    "command": "batch",
    "commands": [
        {"command": "generate", "options": {"kind": "screen", "name": "users"}},
        {"command": "generate", "options": {"kind": "store", "name": "user", "spec": {
            "state": [{"name": "users", "type": "Vec<User>"}],
            "actions": [{"name": "fetch_users", "async": true}]
        }}},
        {"command": "generate", "options": {"kind": "form", "name": "user", "spec": {
            "fields": [
                {"name": "name", "type": "string", "required": true},
                {"name": "email", "type": "string", "required": true, "validate": {"email": true}},
                {"name": "role", "type": "enum", "validate": {"oneOf": ["admin", "user"]}}
            ]
        }}}
    ]
})
```

---

## Open Questions

1. **Streaming responses** — Should batch commands stream results as they complete, or return all at once?
2. **Template customization** — Should agents be able to provide custom templates via JSON?
3. **Watchman integration** — Should the CLI integrate with file watchers like Watchman for more efficient change detection?
4. **WebSocket for dev events** — Should dev server expose a WebSocket for event streaming instead of JSON lines?

---

## Appendix A: Full JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ForgeCLI Command",
  "oneOf": [
    {"$ref": "#/definitions/GenerateCommand"},
    {"$ref": "#/definitions/ListCommand"},
    {"$ref": "#/definitions/InspectCommand"},
    {"$ref": "#/definitions/BatchCommand"}
  ],
  "definitions": {
    "GenerateCommand": {
      "type": "object",
      "required": ["command", "options"],
      "properties": {
        "version": {"type": "string"},
        "command": {"const": "generate"},
        "options": {
          "type": "object",
          "required": ["kind", "name"],
          "properties": {
            "kind": {"enum": ["screen", "store", "form", "query", "component", "migration", "i18n"]},
            "name": {"type": "string"},
            "spec": {"type": "object"},
            "dryRun": {"type": "boolean"},
            "force": {"type": "boolean"}
          }
        }
      }
    },
    "ListCommand": {
      "type": "object",
      "required": ["command", "options"],
      "properties": {
        "command": {"const": "list"},
        "options": {
          "type": "object",
          "required": ["kind"],
          "properties": {
            "kind": {"enum": ["templates", "components", "generators"]},
            "category": {"type": "string"}
          }
        }
      }
    },
    "InspectCommand": {
      "type": "object",
      "required": ["command"],
      "properties": {
        "command": {"const": "inspect"}
      }
    },
    "BatchCommand": {
      "type": "object",
      "required": ["command", "commands"],
      "properties": {
        "command": {"const": "batch"},
        "commands": {
          "type": "array",
          "items": {"oneOf": [
            {"$ref": "#/definitions/GenerateCommand"},
            {"$ref": "#/definitions/ListCommand"}
          ]}
        }
      }
    }
  }
}
```
