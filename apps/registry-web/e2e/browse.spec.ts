import { test, expect } from "@playwright/test"

test.describe("Browse / search", () => {
  test("browse page loads", async ({ page }) => {
    await page.goto("/browse")
    await expect(page).toHaveTitle(/browse/i)
  })

  test("search input is visible and accepts text", async ({ page }) => {
    await page.goto("/browse")
    const input = page.getByPlaceholder(/search/i)
    await expect(input).toBeVisible()
    await input.fill("tanstack")
    await expect(input).toHaveValue("tanstack")
  })

  test("pressing / focuses search from browse page", async ({ page }) => {
    await page.goto("/browse")
    // keyboard shortcut
    await page.keyboard.press("/")
    const input = page.getByPlaceholder(/search/i)
    await expect(input).toBeFocused()
  })

  test("sort options are present", async ({ page }) => {
    await page.goto("/browse")
    // Sort should exist in some form (select or buttons)
    const sortEl = page.getByRole("combobox").first()
    await expect(sortEl).toBeVisible()
  })

  test("empty search shows empty state", async ({ page }) => {
    await page.goto("/browse?q=xyzzy_no_such_package_12345")
    await page.waitForTimeout(1500) // let query settle
    await expect(page.getByText(/no packages/i)).toBeVisible()
  })

  test("URL reflects search query", async ({ page }) => {
    await page.goto("/browse")
    const input = page.getByPlaceholder(/search/i)
    await input.fill("react")
    await page.keyboard.press("Enter")
    await expect(page).toHaveURL(/q=react/)
  })
})
