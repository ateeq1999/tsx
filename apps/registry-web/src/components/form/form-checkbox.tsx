import { FormErrorList } from "./form-error-list"
import { Field } from "@/components/ui/field"

export function FormCheckbox({ form, name, label }: any) {

  return (
    <form.Field name={name}>
      {(field: any) => (
        <Field>

          <label className="flex items-center gap-2">

            <input
              type="checkbox"
              checked={field.state.value ?? false}
              onChange={(e) => field.handleChange(e.target.checked)}
            />

            {label}

          </label>

          <FormErrorList errors={field.state.meta.errors} />

        </Field>
      )}
    </form.Field>
  )
}