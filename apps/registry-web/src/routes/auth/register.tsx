import { createFileRoute } from "@tanstack/react-router"
import { RegisterForm } from "@/components/auth/register-form"

export const Route = createFileRoute("/auth/register")({
  component: RegisterPage
})

function RegisterPage() {
  return (
    <div className="max-w-md mx-auto py-12">
      <h1 className="text-2xl font-bold mb-6">Create Account</h1>
      <RegisterForm />
    </div>
  )
}