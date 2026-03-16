# shadcn/ui Conventions

## File Structure

```
src/
├── components/
│   ├── ui/              # shadcn components (button.tsx, input.tsx)
│   └── {{name}}Form.tsx # Your custom forms
└── lib/
    └── utils.ts         # cn() utility function
```

## Naming Conventions

- UI component file: `components/ui/{{name}}.tsx`
- Custom form: `components/{{name}}Form.tsx`
- Utility: `lib/utils.ts` exports `cn()` function

## Component Usage

```tsx
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"

export function MyForm() {
  return (
    <form>
      <Label htmlFor="email">Email</Label>
      <Input id="email" type="email" />
      <Button type="submit">Submit</Button>
    </form>
  )
}
```

## Variants with cva()

```typescript
import { cva } from "class-variance-authority"

const buttonVariants = cva(
  "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline: "border border-input bg-background hover:bg-accent",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-10 w-10",
      },
    },
  }
)
```
