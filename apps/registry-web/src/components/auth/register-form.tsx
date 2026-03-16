import { revalidateLogic, useForm } from "@tanstack/react-form"
import { useNavigate } from "@tanstack/react-router"

import { registerSchema } from "@/schemas/auth"
import { registerFn } from "@/server/auth/mutations"

import { Button } from "@/components/ui/button"
import { FormField } from "@/components/form/form-field"

export function RegisterForm() {

    const navigate = useNavigate()

    const form = useForm({
        defaultValues: {
            name: "",
            email: "",
            password: "",
        },

        validationLogic: revalidateLogic(),

        validators: {
            onDynamic: registerSchema
        },

        onSubmit: async ({ value }) => {
          const res = await registerFn({ data: value })
          
          if (res.token != null) {
            navigate({ to: "/dashboard" })
            
          }

        },
    })

    return (
        <form
            onSubmit={(e) => {
                e.preventDefault()
                form.handleSubmit()
            }}
            className="space-y-6"
      >
        <FormField form={form} name="name" label="Full name" />
            <FormField
                form={form}
                name="email"
                label="Email"
                type="email"
                placeholder="you@example.com"
            />

            <FormField
                form={form}
                name="password"
                label="Password"
                type="password"
                placeholder="••••••••"
            />

            <Button
                type="submit"
                disabled={form.state.isSubmitting}
                className="w-full"
            >
                {form.state.isSubmitting ? "Signing in..." : "Sign In"}
            </Button>
        </form>
    )
}