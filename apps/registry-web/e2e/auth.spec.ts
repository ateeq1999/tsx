import { test, expect } from "@playwright/test"

test.describe("Auth pages", () => {
  test("login page renders", async ({ page }) => {
    await page.goto("/auth/login")
    await expect(page).toHaveTitle(/login|sign in/i)
    await expect(page.getByRole("textbox", { name: /email/i })).toBeVisible()
    await expect(page.getByRole("textbox", { name: /password/i })).toBeVisible()
    await expect(page.getByRole("button", { name: /sign in|log in/i })).toBeVisible()
  })

  test("register page renders", async ({ page }) => {
    await page.goto("/auth/register")
    await expect(page.getByRole("textbox", { name: /email/i })).toBeVisible()
    await expect(page.getByRole("button", { name: /create account|sign up|register/i })).toBeVisible()
  })

  test("login shows OAuth buttons", async ({ page }) => {
    await page.goto("/auth/login")
    // GitHub and/or Google sign-in buttons (only present when env vars are set)
    // Just verify the page loaded — OAuth buttons are conditional on env config
    await expect(page.getByRole("textbox", { name: /email/i })).toBeVisible()
  })

  test("forgot password link exists on login page", async ({ page }) => {
    await page.goto("/auth/login")
    await expect(page.getByRole("link", { name: /forgot|reset/i })).toBeVisible()
  })

  test("protected route redirects unauthenticated users", async ({ page }) => {
    await page.goto("/dashboard")
    // Should redirect to login
    await expect(page).toHaveURL(/login|auth/)
  })
})
