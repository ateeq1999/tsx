import { createFileRoute } from "@tanstack/react-router"
import { LoginForm } from "@/components/auth/login-form"

export const Route = createFileRoute("/auth/login")({
  component: LoginPage
})

function LoginPage() {
  return (
    <div className="max-w-md mx-auto py-12">
      <h1 className="text-2xl font-bold mb-6">Login</h1>
      <LoginForm />
    </div>
  )
}