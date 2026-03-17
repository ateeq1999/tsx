import { test, expect } from "@playwright/test"

test.describe("Navigation & 404", () => {
  test("header is present on all main pages", async ({ page }) => {
    for (const url of ["/", "/browse", "/auth/login"]) {
      await page.goto(url)
      await expect(page.locator("header")).toBeVisible()
    }
  })

  test("footer is present on landing page", async ({ page }) => {
    await page.goto("/")
    await expect(page.locator("footer")).toBeVisible()
  })

  test("unknown route shows 404 page", async ({ page }) => {
    await page.goto("/this-page-does-not-exist-xyz")
    await expect(page.getByText(/404|not found/i)).toBeVisible()
  })

  test("theme toggle is present in header", async ({ page }) => {
    await page.goto("/")
    // Theme toggle button (sun/moon icon)
    const toggle = page.getByRole("button", { name: /theme|dark|light/i })
    await expect(toggle).toBeVisible()
  })

  test("logo navigates to home", async ({ page }) => {
    await page.goto("/browse")
    const logo = page.getByRole("link", { name: /tsx|registry|home/i }).first()
    await logo.click()
    await expect(page).toHaveURL("/")
  })

  test("mobile nav hamburger opens on small screen", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 })
    await page.goto("/")
    const hamburger = page.getByRole("button", { name: /menu|nav|open/i })
    if (await hamburger.isVisible()) {
      await hamburger.click()
      await expect(page.getByRole("link", { name: /browse/i })).toBeVisible()
    }
  })
})
