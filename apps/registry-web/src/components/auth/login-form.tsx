import { revalidateLogic, useForm } from "@tanstack/react-form"
import { useNavigate } from "@tanstack/react-router"
import { toast } from "sonner"

import { loginSchema } from "@/schemas/auth"
import { loginFn } from "@/server/auth/mutations"

import { Button } from "@/components/ui/button"
import { FormField } from "@/components/form/form-field"

export function LoginForm() {
    const navigate = useNavigate()

    const form = useForm({
        defaultValues: {
            email: "",
            password: "",
        },

        validationLogic: revalidateLogic(),

        validators: {
            onDynamic: loginSchema
        },

        onSubmit: async ({ value }) => {
            try {
                await loginFn({ data: value })
                toast.success("Signed in successfully")
                navigate({ to: "/dashboard" })
            } catch {
                toast.error("Invalid email or password")
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
