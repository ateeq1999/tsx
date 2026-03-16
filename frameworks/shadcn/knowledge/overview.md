# shadcn/ui Overview

shadcn/ui is a collection of re-usable components built with Radix UI and Tailwind CSS. Components are copied into your project rather than installed as a package, giving you full control over customization.

## Key Concepts

- **Components**: Copy/paste into `components/ui/`
- **Radix UI**: Headless primitives for accessibility
- **Tailwind CSS**: Styling via utility classes
- **Variants**: Component customization via `cva()`

## With TanStack Start

shadcn/ui integrates with TanStack Start through:
- Form components replace raw HTML inputs
- Table components for data display
- Dialog components for modals

## Installation

```bash
npx shadcn@latest add button input label
```

Components are added to `components/ui/`.
