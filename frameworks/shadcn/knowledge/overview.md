# shadcn/ui — Overview

shadcn/ui is a collection of accessible, customizable React components built on Radix UI and Tailwind CSS.
This package generates component files you own directly in your project.

## Key commands

| Command | What it generates |
|---|---|
| `add:ui-button` | `components/ui/button.tsx` |
| `add:ui-input` | `components/ui/input.tsx` |
| `add:ui-form` | `components/{{name}}Form.tsx` — TanStack Form + shadcn inputs |
| `add:ui-data-table` | `components/{{name}}Table.tsx` + `components/{{name}}Columns.tsx` |
| `add:ui-dialog` | `components/{{name}}Dialog.tsx` |

## Slot integration

When `shadcn` is in the stack, `tanstack-start` generators replace raw HTML elements
with shadcn Input/Button components via the `ui_imports` slot.
