import { FormErrorList } from "./form-error-list"
import { Input } from "@/components/ui/input"
import { Field, FieldLabel } from "@/components/ui/field"

type Props = {
    form: any
    name: string
    label: string
    type?: string
    placeholder?: string
}

export function FormField({
    form,
    name,
    label,
    type = "text",
    placeholder,
}: Props) {

    return (
        <form.Field name={name}>
            {(field: any) => (
                <Field>
                    <FieldLabel>{label}</FieldLabel>

                    <Input
                        type={type}
                        placeholder={placeholder}
                        value={field.state.value ?? ""}
                        onChange={(e) => field.handleChange(e.target.value)}
                        disabled={form.state.isSubmitting}
                    />

                    <FormErrorList errors={field.state.meta.errors} />
                </Field>
            )}
        </form.Field>
    )
}

export function FormFile({ form, name, label }: any) {

  return (
    <form.Field name={name}>
      {(field: any) => (
        <Field>

          <FieldLabel>{label}</FieldLabel>

          <input
            type="file"
            onChange={(e) => field.handleChange(e.target.files?.[0])}
          />

          <FormErrorList errors={field.state.meta.errors} />

        </Field>
      )}
    </form.Field>
  )
}

export function FormArray({ form, name, label }: any) {

  return (
    <form.Field name={name} mode="array">
      {(field: any) => (

        <div>

          <label>{label}</label>

          {field.state.value?.map((item: string, i: number) => (

            <div key={i} className="flex gap-2">

              <input
                value={item}
                onChange={(e) =>
                  field.handleChange(
                    field.state.value.map((v: string, idx: number) =>
                      idx === i ? e.target.value : v
                    )
                  )
                }
              />

              <button
                type="button"
                onClick={() =>
                  field.handleChange(
                    field.state.value.filter((_: any, idx: number) => idx !== i)
                  )
                }
              >
                Remove
              </button>

            </div>
          ))}

          <button
            type="button"
            onClick={() =>
              field.handleChange([...(field.state.value ?? []), ""])
            }
          >
            Add
          </button>

        </div>
      )}
    </form.Field>
  )
}