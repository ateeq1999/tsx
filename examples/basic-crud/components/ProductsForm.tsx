import { useForm } from "@tanstack/react-form";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { z } from "zod";

const ProductsSchema = z.object({
  title: z.string().min(1, "Title is required"),
  description: z.string().optional(),
  price: z.coerce.number().int().positive("Price must be positive"),
});

type ProductsFormValues = z.infer<typeof ProductsSchema>;

export function ProductsForm({ onSubmit }: { onSubmit: (values: ProductsFormValues) => void }) {
  const form = useForm({
    defaultValues: {
      title: "",
      description: "",
      price: 0,
    },
    validators: {
      onChange: ProductsSchema,
    },
  });

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        form.handleSubmit(onSubmit)();
      }}
      className="space-y-4"
    >
      <form.Field name="title">
        {(field) => (
          <div className="flex flex-col gap-1">
            <Label htmlFor={field.name}>Title</Label>
            <Input
              id={field.name}
              value={field.state.value}
              onChange={(e) => field.handleChange(e.target.value)}
              placeholder="Product title"
            />
            {field.state.meta.errors.length > 0 && (
              <p className="text-sm text-red-500">{field.state.meta.errors[0]}</p>
            )}
          </div>
        )}
      </form.Field>

      <form.Field name="description">
        {(field) => (
          <div className="flex flex-col gap-1">
            <Label htmlFor={field.name}>Description</Label>
            <Input
              id={field.name}
              value={field.state.value}
              onChange={(e) => field.handleChange(e.target.value)}
              placeholder="Optional description"
            />
          </div>
        )}
      </form.Field>

      <form.Field name="price">
        {(field) => (
          <div className="flex flex-col gap-1">
            <Label htmlFor={field.name}>Price</Label>
            <Input
              id={field.name}
              type="number"
              value={field.state.value}
              onChange={(e) => field.handleChange(Number(e.target.value))}
              placeholder="0"
            />
            {field.state.meta.errors.length > 0 && (
              <p className="text-sm text-red-500">{field.state.meta.errors[0]}</p>
            )}
          </div>
        )}
      </form.Field>

      <Button type="submit">Submit</Button>
    </form>
  );
}
