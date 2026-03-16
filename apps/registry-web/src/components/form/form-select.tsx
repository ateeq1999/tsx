import { FormErrorList } from "./form-error-list"
import { Field, FieldLabel } from "@/components/ui/field"

type Option = {
  label: string
  value: string
}

export function FormSelect({ form, name, label, options }: any) {

  return (
    <form.Field name={name}>
      {(field: any) => (
        <Field>
          <FieldLabel>{label}</FieldLabel>

          <select
            value={field.state.value ?? ""}
            onChange={(e) => field.handleChange(e.target.value)}
            className="input"
          >
            <option value="">Select</option>

            {options.map((opt: Option) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>

          <FormErrorList errors={field.state.meta.errors} />
        </Field>
      )}
    </form.Field>
  )
}