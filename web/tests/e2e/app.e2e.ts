/**
 * E2E smoke tests + a11y audits for D2R Marketplace.
 *
 * Tests run against Vite dev server (port 7340). Tauri IPC is unavailable
 * in the browser — IPC-dependent pages crash on mount. We test the shell
 * resilience and audit whatever UI is rendered server-side.
 */
import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

// ── Navigation ─────────────────────────────────────────────────────

test('app shell renders with navigation', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('text=D2R').first()).toBeVisible({ timeout: 10_000 })
  await expect(page.locator('nav, [role="navigation"], .d2emu-nav-desktop').first()).toBeVisible()
})

test('nav tabs are present', async ({ page }) => {
  await page.goto('/')
  const tabs = page.locator('button[role="tab"]')
  const count = await tabs.count()
  expect(count).toBeGreaterThanOrEqual(5)
})

test('navigating between tabs does not freeze', async ({ page }) => {
  await page.goto('/')
  const tabs = page.locator('button[role="tab"]')
  const count = await tabs.count()
  for (let i = 0; i < Math.min(count, 5); i++) {
    await tabs.nth(i).click()
    await page.waitForTimeout(300)
  }
  // App should still be responsive
  await expect(page.locator('h1').first()).toBeVisible()
})

// ── a11y audits on the home page (no IPC needed) ──────────────────

test('home page has no critical a11y violations', async ({ page }) => {
  await page.goto('/')
  await page.waitForTimeout(1_000)
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze()
  const critical = results.violations.filter(v => v.impact === 'critical' || v.impact === 'serious')
  if (critical.length > 0) {
    console.log('=== home a11y violations ===')
    for (const v of critical) {
      console.log(`[${v.impact}] ${v.id}: ${v.help}`)
      console.log(`  ${v.helpUrl}`)
      for (const n of v.nodes.slice(0, 3)) {
        console.log(`  → ${n.html.slice(0, 150)}`)
      }
    }
  }
  expect(critical.length).toBe(0)
})

test('warehouse page has no critical a11y violations', async ({ page }) => {
  await page.goto('/warehouse')
  await page.waitForTimeout(2_000)
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa'])
    .analyze()
  const critical = results.violations.filter(v => v.impact === 'critical' || v.impact === 'serious')
  if (critical.length > 0) {
    console.log('=== warehouse a11y violations ===')
    for (const v of critical) {
      console.log(`[${v.impact}] ${v.id}: ${v.help}`)
      for (const n of v.nodes.slice(0, 3)) {
        console.log(`  → ${n.html.slice(0, 150)}`)
      }
    }
  }
  expect(critical.length).toBe(0)
})
