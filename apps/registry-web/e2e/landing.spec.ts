import { test, expect } from "@playwright/test"

test.describe("Landing page", () => {
  test("loads and shows hero content", async ({ page }) => {
    await page.goto("/")
    await expect(page).toHaveTitle(/tsx registry/i)
    // Hero heading
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible()
  })

  test("shows registry stats section", async ({ page }) => {
    await page.goto("/")
    // Stats counters (packages, downloads, versions)
    const statsSection = page.locator("[data-testid=stats], .stat-card, [class*=stat]").first()
    await expect(statsSection).toBeVisible()
  })

  test("nav links are present", async ({ page }) => {
    await page.goto("/")
    await expect(page.getByRole("link", { name: /browse/i })).toBeVisible()
    await expect(page.getByRole("link", { name: /docs/i })).toBeVisible()
  })

  test("install command is visible", async ({ page }) => {
    await page.goto("/")
    await expect(page.getByText(/tsx/)).toBeVisible()
  })
})
