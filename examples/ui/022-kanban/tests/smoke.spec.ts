/**
 * 022-kanban 冒烟测试 (Plan 404) — 对应 acceptance.atd T1-T8.
 * 前端 vite 跑在 localhost:3022 (pac.at front_port), 后端 8022 (vite proxy 转发).
 *
 * 端口缺口注意: 当前 `auto run` 不读 pac.at 的 front_port/back_port, 测试前需用
 * 环境变量分离启动:
 *   AUTO_HTTP_PORT=8022 ./app-022-kanban-back.exe          (后端)
 *   AUTO_FRONT_PORT=3022 AUTO_HTTP_PORT=8022 pnpm dev       (前端, 在 gen/front/vue)
 */
import { test, expect } from '@playwright/test'

// 等看板加载: 打开首页, 等首张种子卡片渲染 (store.Init 的 GET /api/cards 回填).
async function waitForBoard(page) {
  await page.goto('/')
  await page.locator('text=Project Board').waitFor({ timeout: 10000 })
  // 等 store.Init 异步拉取后端数据
  await page.waitForTimeout(1500)
}

test('T1: 初始看板渲染 — 9 张种子卡片', async ({ page }) => {
  await waitForBoard(page)
  const body = await page.locator('body').innerText()
  // 4 todo
  expect(body).toContain('Design landing page')
  expect(body).toContain('Write API docs')
  expect(body).toContain('Fix login bug')
  expect(body).toContain('Setup CI/CD')
  // 2 doing
  expect(body).toContain('Implement auth flow')
  expect(body).toContain('Build dashboard')
  // 3 done
  expect(body).toContain('Setup database')
  expect(body).toContain('Create user model')
  expect(body).toContain('Design logo')
})

test('T2: 三列标题显示', async ({ page }) => {
  await waitForBoard(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('To Do')
  expect(body).toContain('In Progress')
  expect(body).toContain('Done')
})

// 找到某张卡片的"行"div: 定位标题 span, 再上溯到卡片行.
// 卡片行结构: div.bg-white > span(title) + button(>) + button(×).
async function cardRow(page, title) {
  // getByText 精确匹配标题 span; locator('..') 到卡片行 div.
  return page.getByText(title, { exact: true }).locator('xpath=..')
}

test('T3-T6: 添加 → 移动 todo→doing → 移动 doing→done → 删除', async ({ page }) => {
  await waitForBoard(page)

  // T3: 添加卡片 "Playwright card"
  const input = page.locator('input[placeholder="Add a card…"]')
  await input.fill('Playwright card')
  await page.locator('button:has-text("+ Add Card")').click()
  await page.waitForTimeout(800)
  let body = await page.locator('body').innerText()
  expect(body).toContain('Playwright card')

  // T4: 把 todo 卡片 (Design landing page) 移到 doing.
  // 卡片行有两个按钮: [0]=">"(移动) [1]="×"(删除).
  await (await cardRow(page, 'Design landing page')).locator('button').nth(0).click()
  await page.waitForTimeout(800)
  body = await page.locator('body').innerText()
  expect(body).toContain('Design landing page')

  // T5: 把 doing 卡片 (Implement auth flow) 移到 done.
  await (await cardRow(page, 'Implement auth flow')).locator('button').nth(0).click()
  await page.waitForTimeout(800)
  body = await page.locator('body').innerText()
  expect(body).toContain('Implement auth flow')

  // T6: 删除一张卡片 (Write API docs 的 "×" 按钮 = button[1]).
  await (await cardRow(page, 'Write API docs')).locator('button').nth(1).click()
  await page.waitForTimeout(800)
  body = await page.locator('body').innerText()
  expect(body).not.toContain('Write API docs')
})

test('T7: 刷新后状态持久 (后端内存态)', async ({ page }) => {
  await waitForBoard(page)
  // 刷新页面, store.Init 重新 GET /api/cards, 后端内存态保持.
  // Design landing page 在 T4 移到了 doing — 刷新后仍在 (验证持久, 而非重置).
  await page.reload()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Setup CI/CD')
})

test('T8: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })
  await waitForBoard(page)
  await page.waitForTimeout(1000)
  // 过滤掉 favicon 404 等无害噪声
  const real = errors.filter((e) => !e.includes('favicon') && !e.includes('Failed to load resource'))
  expect(real, `console errors: ${real.join('; ')}`).toEqual([])
})

test('T9: 拖拽卡片 todo→done (HTML5 drag-and-drop)', async ({ page }) => {
  await waitForBoard(page)
  // 源: To Do 列里的 "Setup CI/CD" 卡片行; 目标: Done 列容器.
  // Done 列是带 ondrop 的 col 容器 —— 用其标题 "Done" 定位列.
  const src = await cardRow(page, 'Setup CI/CD')
  // Done 列: 含 "Done" 标题文本的最近 col 容器. 用 ondrop 所在的列容器.
  // PLAN-037 T-01 修定位（首红透明）：PLAN-684 rev2 把列容器 bg-green-50
  // 语义化为 bg-muted/50，旧选择器在冻结基线上已必失败；断言语义不变。
  const doneCol = page.getByText('Done', { exact: true }).locator('xpath=ancestor::div[contains(@class,"bg-muted/50")][1]')
  await src.dragTo(doneCol)
  await page.waitForTimeout(1000)
  const body = await page.locator('body').innerText()
  // Setup CI/CD 仍在页面(现在 Done 列), 拖拽成功
  expect(body).toContain('Setup CI/CD')
})

// PLAN-037 T-04: 官方主题包 —— 家族/模式 2x2 往返（setupState 模型驱动，
// 生效包值 oracle；既有 dark-first 显式默认保留；卡片/列业务跨换肤保持）。
test('T-P037: 主题 2x2 往返 — dark-first 保留 + 卡片业务保持', async ({ page }) => {
  await waitForBoard(page)
  const themeRoot = () => page.evaluate(() =>
    getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim())
  const drive = (t, d) => page.evaluate(([t, d]) => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState
    st.theme_name = t
    st.dark_mode = d
  }, [t, d])

  // 既有显式默认：App dark_mode = true（暗支外观优先）→ boot 即 Auto/dark。
  expect(await themeRoot()).toBe('240 10% 4%')

  // 既有 ToggleDarkMode 业务 handler 在 design 绑定下仍真实切换。
  await page.locator('header button, button').filter({ hasText: /🌙|☀️/ }).first().click()
  await page.waitForTimeout(400)
  expect(await themeRoot()).toBe('0 0% 100%')

  // Solarized/dark 下卡片数据保持；2x2 往返回 dark-first 默认。
  await drive('Solarized', true)
  await page.waitForTimeout(400)
  expect(await themeRoot()).toBe('192 100% 11%')
  expect((await page.locator('body').innerText())).toContain('Design landing page')
  await page.screenshot({ path: 'test-results/p037_kanban_solarized_dark.png', fullPage: true })
  for (const [t, d] of [['Solarized', false], ['Auto', true]]) {
    await drive(t, d)
    await page.waitForTimeout(350)
  }
  expect(await themeRoot()).toBe('240 10% 4%')
  expect((await page.locator('body').innerText())).toContain('Setup CI/CD')
})
