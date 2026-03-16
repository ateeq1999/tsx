import { FormErrorList } from "./form-error-list"
import { Textarea } from "@/components/ui/textarea"
import { Field, FieldLabel } from "@/components/ui/field"

export function FormTextarea({ form, name, label, placeholder }: any) {

  return (
    <form.Field name={name}>
      {(field: any) => (
        <Field>
          <FieldLabel>{label}</FieldLabel>

          <Textarea
            placeholder={placeholder}
            value={field.state.value ?? ""}
            onChange={(e) => field.handleChange(e.target.value)}
          />

          <FormErrorList errors={field.state.meta.errors} />
        </Field>
      )}
    </form.Field>
  )
}