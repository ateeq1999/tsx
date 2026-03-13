# TSX - TanStack Start Code Generation CLI

A Rust CLI tool for generating boilerplate code for TanStack Start projects. TSX reduces AI agent token overhead by automatically generating Drizzle schemas, TanStack Query hooks, forms, tables, server functions, and more through a composable template system.

## Features

- **Scaffold Complete CRUD Features** - Generate full resource modules with one command
- **Drizzle Schema Generation** - Type-safe database table definitions
- **Server Functions** - Typed TanStack Start server functions
- **TanStack Query Hooks** - Suspense-ready query and mutation hooks
- **Form Components** - React Hook Form integration
- **Data Tables** - React Table with pagination
- **Auth Configuration** - Better Auth setup
- **Route Pages** - Index and detail pages
- **Prettier Integration** - Auto-formatted output
- **Dry-Run Mode** - Preview changes before writing files

## Installation

### Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/your-repo/tsx/releases):

```bash
# Windows
curl -LO https://github.com/your-repo/tsx/releases/latest/download/tsx-x86_64-pc-windows-msvc.exe
mv tsx-x86_64-pc-windows-msvc.exe tsx.exe

# Linux
curl -LO https://github.com/your-repo/tsx/releases/latest/download/tsx-x86_64-unknown-linux-gnu
chmod +x tsx-x86_64-unknown-linux-gnu
sudo mv tsx-x86_64-unknown-linux-gnu /usr/local/bin/tsx

# macOS
brew install tsx
```

### Build from Source

```bash
git clone https://github.com/your-repo/tsx.git
cd tsx
cargo build --release
cp target/release/tsx.exe /usr/local/bin/  # or .exe on Windows
```

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `tsx init` | Bootstrap a new TanStack Start project |
| `tsx add:feature` | Scaffold a complete CRUD feature module |
| `tsx add:schema` | Generate a Drizzle schema table definition |
| `tsx add:server-fn` | Generate a typed server function |
| `tsx add:query` | Generate a TanStack Query hook |
| `tsx add:form` | Generate a TanStack Form component |
| `tsx add:table` | Generate a TanStack Table component |
| `tsx add:page` | Add a new route page |
| `tsx add:auth` | Configure Better Auth |
| `tsx add:auth-guard` | Wrap a route with a session guard |
| `tsx add:migration` | Run drizzle-kit generate + migrate |
| `tsx add:seed` | Generate a database seed file |

### Global Flags

| Flag | Description |
|------|-------------|
| `--overwrite` | Overwrite existing files without prompting |
| `--dry-run` | Print what would be written without creating files |

### Examples

#### Add a Schema

```bash
tsx add:schema --json '{
  "name": "products",
  "fields": [
    {"name": "title", "type": "string"},
    {"name": "price", "type": "number"},
    {"name": "description", "type": "text"}
  ],
  "timestamps": true
}'
```

#### Add a Server Function

```bash
tsx add:server-fn --json '{
  "name": "getProducts",
  "table": "products",
  "operation": "list",
  "auth": false
}'
```

#### Add a Complete Feature

```bash
tsx add:feature --json '{
  "name": "posts",
  "fields": [
    {"name": "title", "type": "string"},
    {"name": "content", "type": "text"},
    {"name": "published", "type": "boolean"}
  ],
  "operations": ["list", "create", "update", "delete"],
  "auth": true
}'
```

This creates:
- `db/schema/posts.ts` - Drizzle table schema
- `server-functions/postslist.ts` - List server function
- `server-functions/postscreate.ts` - Create server function
- `server-functions/postsupdate.ts` - Update server function
- `server-functions/postsdelete.ts` - Delete server function
- `components/posts/posts-table.tsx` - Data table component
- `routes/posts.tsx` - Index page
- `routes/posts/$id.tsx` - Detail page

#### Dry Run Mode

```bash
tsx add:feature --dry-run --json '{
  "name": "users",
  "fields": [{"name": "email", "type": "string"}],
  "operations": ["list"],
  "auth": false
}'
```

Output shows what files would be created without writing them.

#### Add Auth

```bash
tsx add:auth --json '{
  "providers": ["github", "google"],
  "session_fields": [],
  "email_verification": true
}'
```

#### Protect a Route

```bash
tsx add:auth-guard --json '{
  "route_path": "/dashboard",
  "redirect_to": "/login"
}'
```

## Project Structure

TSX expects your project to have a standard TanStack Start layout:

```
my-project/
├── package.json
├── db/
│   ├── schema/          # Generated Drizzle schemas
│   └── seeds/          # Database seed files
├── server-functions/    # TanStack Start server functions
├── queries/            # TanStack Query hooks
├── components/         # React components
│   └── posts/
│       ├── posts-form.tsx
│       └── posts-table.tsx
├── routes/             # TanStack Start routes
│   ├── posts.tsx
│   └── posts/
│       └── $id.tsx
└── lib/
    └── auth.ts         # Better Auth configuration
```

## Field Types

### Drizzle Schema Types

| Type | Description |
|------|-------------|
| `string` | VARCHAR column |
| `text` | TEXT column |
| `number` | INTEGER column |
| `boolean` | BOOLEAN column |
| `date` | DATE/TIMESTAMP column |
| `json` | JSON column |
| `uuid` | UUID column |
| `email` | VARCHAR with email validation |
| `url` | VARCHAR with URL validation |
| `reference` | Foreign key relation |

### Form Field Types

| Type | Description |
|------|-------------|
| `input` | Text input |
| `select` | Dropdown select |
| `switch` | Toggle switch |
| `datepicker` | Date picker |
| `textarea` | Multi-line text |

## Configuration

### Templates

Templates are embedded in the binary by default. To use custom templates, place a `templates/` directory alongside the executable:

```
tsx.exe
templates/
├── atoms/
│   ├── drizzle/
│   ├── zod/
│   ├── form/
│   └── query/
├── molecules/
├── layouts/
└── features/
```

### Prettier

TSX uses Prettier for code formatting if available. If Prettier is not installed, files are written without formatting.

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run E2E tests
cargo test --test e2e
```

### Binary Size

The release binary is optimized for minimal size:
- `opt-level = 3`
- `lto = true`
- `codegen-units = 1`
- `strip = true`

Current size: ~2.4MB

## Architecture

TSX uses a three-tier template system:

1. **Atoms** - Smallest reusable units (column definitions, field rules, query keys)
2. **Molecules** - Composed atoms (table bodies, schema blocks, handler functions)
3. **Layouts** - File wrappers (component, route, base layouts)
4. **Features** - Complete file templates (schema, server_fn, query, form, table, page)

Templates are rendered using MiniJinja2 with custom filters for case conversion (snake_case, pascal_case, camel_case, kebab_case).

Import hoisting is handled via thread-local collectors that gather imports during rendering and output them at the top of generated files.

## License

MIT
