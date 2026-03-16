import { createServerFn } from "@tanstack/react-start"
import { loginSchema, registerSchema } from "@/schemas/auth"
import { auth } from "@/lib/auth"

export const loginFn = createServerFn({ method: "POST" })
  .inputValidator(loginSchema)
  .handler(async ({ data }) => {
    return await auth.api.signInEmail({
      body: data
    })
  })


export const registerFn = createServerFn({ method: "POST" })
  .inputValidator(registerSchema)
  .handler(async ({ data }) => {
    return await auth.api.signUpEmail({
      body: data
    })
  })

export const logoutFn = createServerFn({ method: "POST" })
  .handler(async () => {
    return await auth.api.signOut()
  })
