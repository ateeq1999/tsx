import { createServerFn } from "@tanstack/react-start"
import { getRequestHeaders } from "@tanstack/react-start/server"
import { z } from "zod"
import { loginSchema, registerSchema } from "@/schemas/auth"
import { auth } from "@/lib/auth"

export const loginFn = createServerFn({ method: "POST" })
  .inputValidator(loginSchema)
  .handler(async ({ data }) => {
    return await auth.api.signInEmail({ body: data })
  })

export const registerFn = createServerFn({ method: "POST" })
  .inputValidator(registerSchema)
  .handler(async ({ data }) => {
    return await auth.api.signUpEmail({ body: data })
  })

export const logoutFn = createServerFn({ method: "POST" })
  .handler(async () => {
    return await auth.api.signOut()
  })

export const updateProfileFn = createServerFn({ method: "POST" })
  .inputValidator(z.object({ name: z.string().min(1, "Name is required") }))
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.updateUser({ body: { name: data.name }, headers })
  })

export const changePasswordFn = createServerFn({ method: "POST" })
  .inputValidator(
    z.object({
      currentPassword: z.string().min(1),
      newPassword: z.string().min(8, "Minimum 8 characters"),
    })
  )
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.changePassword({
      body: {
        currentPassword: data.currentPassword,
        newPassword: data.newPassword,
        revokeOtherSessions: false,
      },
      headers,
    })
  })

export const forgotPasswordFn = createServerFn({ method: "POST" })
  .inputValidator(z.object({ email: z.string().email() }))
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.forgetPassword({
      body: { email: data.email, redirectTo: "/auth/reset-password" },
      headers,
    })
  })

export const resetPasswordFn = createServerFn({ method: "POST" })
  .inputValidator(z.object({ token: z.string().min(1), newPassword: z.string().min(8) }))
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.resetPassword({
      body: { token: data.token, newPassword: data.newPassword },
      headers,
    })
  })

export const listSessionsFn = createServerFn({ method: "GET" }).handler(async () => {
  const headers = getRequestHeaders()
  return await auth.api.listSessions({ headers })
})

export const revokeSessionFn = createServerFn({ method: "POST" })
  .inputValidator(z.object({ token: z.string() }))
  .handler(async ({ data }) => {
    const headers = getRequestHeaders()
    return await auth.api.revokeSession({ body: { token: data.token }, headers })
  })

export const deleteAccountFn = createServerFn({ method: "POST" }).handler(async () => {
  const headers = getRequestHeaders()
  return await auth.api.deleteUser({ headers })
})
