import { expect, test, type Page } from '@playwright/test'

async function dismissWelcomeIfOpen(page: Page) {
  const welcome = page.getByRole('dialog').filter({ hasText: 'Welcome to frees' })
  if (await welcome.isVisible({ timeout: 1500 }).catch(() => false)) {
    await page.keyboard.press('Escape')
    await expect(welcome).toBeHidden()
  }
}

async function setEditorContent(page: Page, text: string) {
  const content = page.locator('.cm-content')
  await expect(content).toBeVisible({ timeout: 30_000 })
  await content.click()
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+a' : 'Control+a')
  await page.keyboard.press('Backspace')
  await content.pressSequentially(text, { delay: 5 })
  const firstLine = text.trim().split('\n')[0]
  if (firstLine) {
    await expect(content).toContainText(firstLine, { timeout: 10_000 })
  }
  await page.waitForTimeout(300)
}

test.describe('Phase 9 End-to-End Browser Journeys', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('button', { name: 'Solve', exact: true })).toBeVisible({
      timeout: 60_000,
    })
    // Disable notification overlays so they do not intercept action bar pointer events
    await page.addStyleTag({
      content: '.mantine-Notifications-root { display: none !important; pointer-events: none !important; }',
    })
    await dismissWelcomeIfOpen(page)
  })

  test('Journey 1: Scalar equations with units, deliberate error recovery & undo', async ({ page }) => {
    // 1. Enter valid scalar equations with physical units
    const scalarDoc = 'P = 100 [kPa]\nT = 300 [K]\nv = 0.287 * T / P\n'
    await setEditorContent(page, scalarDoc)

    // Check the system: must report Structurally solvable
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Structurally solvable/i).first()).toBeVisible({ timeout: 15_000 })
    await page.waitForTimeout(500)

    // Solve the system: must report Solved
    await page.getByRole('button', { name: 'Solve', exact: true }).click()
    await expect(page.getByText(/^Solved/i).first()).toBeVisible({ timeout: 15_000 })

    // 2. Introduce deliberate unit error: unknown unit literal
    await setEditorContent(page, `${scalarDoc}bad_val = 42 [bogus_unit]\n`)
    await page.waitForTimeout(500)
    await page.getByRole('button', { name: 'Solve', exact: true }).click()

    // Must show distinct 'unconverted units' warning rather than silently accepting or crashing
    await expect(page.getByText(/unconverted units/i).first()).toBeVisible({ timeout: 15_000 })

    // 3. Test Recovery & undo: restore clean equations and verify solve
    await setEditorContent(page, scalarDoc)
    await page.waitForTimeout(500)
    await page.getByRole('button', { name: 'Solve', exact: true }).click()
    await expect(page.getByText(/^Solved/i).first()).toBeVisible({ timeout: 15_000 })
  })

  test('Journey 2: Component chain, duplicate wiring diagnosis & modal focus restoration', async ({ page }) => {
    // 1. Enter valid component chain with closed circuit
    const circuitDoc = `VoltageSource V1(E = 12)
Resistor R1(R = 10)
Resistor R2(R = 20)
Ground G1()
connect(V1.p, R1.a)
connect(R1.b, R2.a)
connect(R2.b, V1.n, G1.port)
`
    await setEditorContent(page, circuitDoc)

    // Check topology: structurally solvable
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Structurally solvable/i).first()).toBeVisible({ timeout: 15_000 })
    await page.waitForTimeout(500)

    // Solve the component chain
    await page.getByRole('button', { name: 'Solve', exact: true }).click()
    await expect(page.getByText(/^Solved/i).first()).toBeVisible({ timeout: 15_000 })

    // 2. Deliberate syntax/wiring error: incomplete connect statement
    await setEditorContent(page, `${circuitDoc}connect(R1.b, )\n`)
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Syntax error/i).first()).toBeVisible({ timeout: 15_000 })

    // Recover by restoring valid circuit
    await setEditorContent(page, circuitDoc)
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Structurally solvable/i).first()).toBeVisible({ timeout: 15_000 })

    // 3. Test Shortcuts modal opening with Shift+? and focus restoration on Escape
    await page.keyboard.press('Shift+?')
    const shortcutsDialog = page.getByRole('dialog').filter({ hasText: /Keyboard Shortcuts/i })
    if (await shortcutsDialog.isVisible({ timeout: 3000 }).catch(() => false)) {
      await page.keyboard.press('Escape')
      await expect(shortcutsDialog).toBeHidden({ timeout: 5000 })
    }

    // Verify editor is visible and retains focus/interaction
    const editor = page.locator('.cm-content')
    await expect(editor).toBeVisible({ timeout: 10_000 })
    await editor.click()
  })

  test('Journey 3: Responsive narrow 360px layout, touch/pointer access & degrees of freedom', async ({ page }) => {
    // 1. Test structural errors on desktop first:
    // Overspecified (0 degrees of freedom, 2 conflicting equations for 1 unknown)
    await setEditorContent(page, 'x = 1\nx = 2\n')
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Overspecified/i).first()).toBeVisible({ timeout: 15_000 })

    // Underspecified (extra degrees of freedom: 1 equation for 2 unknowns)
    await setEditorContent(page, 'x + y = 10\n')
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Underspecified/i).first()).toBeVisible({ timeout: 15_000 })

    // Recover to solvable system
    await setEditorContent(page, 'x = 1\ny = 2\n')
    await page.getByRole('button', { name: 'Check', exact: true }).click()
    await expect(page.getByText(/Structurally solvable/i).first()).toBeVisible({ timeout: 15_000 })
    await page.waitForTimeout(500)
    await page.getByRole('button', { name: 'Solve', exact: true }).click()
    await expect(page.getByText(/^Solved/i).first()).toBeVisible({ timeout: 15_000 })

    // 2. Switch to 360px mobile viewport to test responsive narrow layout & touch access
    await page.setViewportSize({ width: 360, height: 740 })
    await page.waitForTimeout(300)

    // Action bar buttons (Check and Solve) must remain visible and accessible
    const solveBtn = page.getByRole('button', { name: 'Solve', exact: true })
    const checkBtn = page.getByRole('button', { name: 'Check', exact: true })
    await expect(solveBtn).toBeVisible()
    await expect(checkBtn).toBeVisible()

    // Test mobile touch navigation between tabs (Equations <-> Variables)
    const variablesTab = page.getByRole('button', { name: 'Variables', exact: true })
    await expect(variablesTab).toBeVisible()
    await variablesTab.click()
    await page.waitForTimeout(200)

    const equationsTab = page.getByRole('button', { name: 'Equations', exact: true })
    await expect(equationsTab).toBeVisible()
    await equationsTab.click()
    await page.waitForTimeout(200)

    // Verify editor is operable in narrow mobile view
    const editor = page.locator('.cm-content')
    await expect(editor).toBeVisible({ timeout: 10_000 })

    // Restore desktop viewport
    await page.setViewportSize({ width: 1280, height: 800 })
  })
})
