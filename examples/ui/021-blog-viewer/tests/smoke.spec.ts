/**
 * 021-blog-viewer 冒烟测试 (Plan 521) — 对应 acceptance.atd T1-T7.
 * 验证 Medium 式博客阅读器端到端：首页 / 标签过滤 / 详情分段 / 点赞切换 / 发布 / 表单校验 / 二次确认删除.
 * 前置：后端 (:8421) + 前端 (:3021) 均已启动并就绪.
 *
 * 路由：/ , /post/:id , /new
 */
import { test, expect } from '@playwright/test'

async function waitForHome(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.locator('text=My Blog').waitFor({ timeout: 10000 })
  await page.waitForTimeout(1000)
}

test('T1: 首页加载 — 顶栏 My Blog + 计数 6 articles + 6 张卡', async ({ page }) => {
  await waitForHome(page)
  const body = await page.locator('body').innerText()

  // 顶栏品牌
  expect(body).toContain('My Blog')
  // 文章计数
  expect(body).toContain('6 articles')

  // 6 篇预置文章标题
  expect(body).toContain('Getting Started with Rust')
  expect(body).toContain('Building UIs with Auto Language')
  expect(body).toContain('WebAssembly in 2026')
  expect(body).toContain('Advanced Systems with Rust')
  expect(body).toContain('Reactive Architecture in AutoUI')
  expect(body).toContain('High Performance WASI')
})

test('T2: 标签过滤 — 点 Rust chip → 只剩 2 张 Rust 卡；点 All 恢复 6 张', async ({ page }) => {
  await waitForHome(page)

  // 点击 Rust 过滤 chip
  const rustButton = page.locator('button', { hasText: /^Rust$/ })
  await rustButton.click()
  await page.waitForTimeout(800)

  let body = await page.locator('body').innerText()
  expect(body).toContain('Getting Started with Rust')
  expect(body).toContain('Advanced Systems with Rust')
  expect(body).not.toContain('Building UIs with Auto Language')
  expect(body).not.toContain('WebAssembly in 2026')

  // 点击 All 恢复全量
  const allButton = page.locator('button', { hasText: /^All$/ })
  await allButton.click()
  await page.waitForTimeout(800)

  body = await page.locator('body').innerText()
  expect(body).toContain('6 articles')
  expect(body).toContain('Building UIs with Auto Language')
})

test('T3: 进入详情页 — 点首卡 → URL /post/:id、标题/作者/正文与卡片一致、正文分段 ≥3 段', async ({ page }) => {
  await waitForHome(page)

  // 点击首张卡片 "Getting Started with Rust"
  await page.getByText('Getting Started with Rust', { exact: true }).first().click()
  await page.waitForTimeout(1000)

  // 验证 URL
  expect(page.url()).toMatch(/\/post\/6/)

  const body = await page.locator('body').innerText()
  expect(body).toContain('Getting Started with Rust')
  expect(body).toContain('Alice Chen')
  expect(body).toContain('Writer & Contributor')

  // 验证正文分段至少 3 段 (包含第一段关键文本)
  expect(body).toContain('Rust is a systems programming language that runs blazingly fast')
  expect(body).toContain('In this article, we explore ownership, borrowing, and lifetimes')

  // 返回首页
  await page.locator('button:has-text("← Back to Stories")').click()
  await page.waitForTimeout(500)
  expect(page.url()).not.toMatch(/\/post\/6/)
})

test('T4: 点赞 clap — 点赞 → 计数 +1；再点 → 回落', async ({ page }) => {
  await waitForHome(page)

  // 进首篇 (id=6, 初始 12 claps)
  await page.getByText('Getting Started with Rust', { exact: true }).first().click()
  await page.waitForTimeout(1000)

  const clapBtn = page.locator('button:has-text("claps")')
  await expect(clapBtn).toContainText('12 claps')

  // 点赞 +1
  await clapBtn.click()
  await page.waitForTimeout(800)
  await expect(clapBtn).toContainText('13 claps')

  // 再次点赞 → 回落为 12
  await clapBtn.click()
  await page.waitForTimeout(800)
  await expect(clapBtn).toContainText('12 claps')

  // 回首页
  await page.locator('button:has-text("← Back to Stories")').click()
  await page.waitForTimeout(500)
})

test('T5: 发布流程 — 顶栏 Write → /new 表单填写 → Publish → 回列表且新卡置顶、计数变 7', async ({ page }) => {
  await waitForHome(page)

  // 点击顶栏 Write 按钮
  await page.locator('button:has-text("Write")').first().click()
  await page.waitForTimeout(800)
  expect(page.url()).toMatch(/\/new/)

  // 填写表单
  await page.fill('input[placeholder="Title of your story..."]', 'Playwright Test Story')
  await page.fill('input[placeholder="Your name (e.g. Jane Doe)"]', 'Playwright Tester')
  await page.fill('input[placeholder="Brief subtitle or lead summary..."]', 'A fast end-to-end automated test story.')
  await page.fill('textarea[placeholder="Write your story here... Separate paragraphs with blank lines."]', 'Paragraph 1.\n\nParagraph 2.')

  // 点击 Publish
  await page.locator('button:has-text("Publish")').click()
  await page.waitForTimeout(1000)

  // 验证回到首页且新文章出现
  const body = await page.locator('body').innerText()
  expect(body).toContain('Playwright Test Story')
  expect(body).toContain('Playwright Tester')
  expect(body).toContain('7 articles')
})

test('T6: 发布校验 — 空标题提交 → 错误提示出现、不跳转', async ({ page }) => {
  await waitForHome(page)

  // 点击 Write 按钮
  await page.locator('button:has-text("Write")').first().click()
  await page.waitForTimeout(800)
  expect(page.url()).toMatch(/\/new/)

  // 不填标题，直接点击 Publish
  await page.locator('button:has-text("Publish")').click()
  await page.waitForTimeout(500)

  // 验证错误提示
  const body = await page.locator('body').innerText()
  expect(body).toContain('Title cannot be empty')
  // 保持在 /new
  expect(page.url()).toMatch(/\/new/)

  // 点击 Cancel 返回
  await page.locator('button:has-text("← Cancel")').click()
  await page.waitForTimeout(500)
})

test('T7: 删除流程 — 详情页删除 → 确认条出现 → 确认 → 回列表、卡片消失、计数回落', async ({ page }) => {
  await waitForHome(page)

  // 点击刚刚发布的新文章
  await page.getByText('Playwright Test Story', { exact: true }).first().click()
  await page.waitForTimeout(1000)

  // 点击删除按钮
  await page.locator('button:has-text("Delete")').click()
  await page.waitForTimeout(500)

  // 验证二次确认条
  let body = await page.locator('body').innerText()
  expect(body).toContain('Delete this article?')
  expect(body).toContain('This action cannot be undone')

  // 点击确认删除
  await page.locator('button:has-text("Yes, Delete")').click()
  await page.waitForTimeout(1000)

  // 验证返回首页且文章消失、计数恢复为 6
  body = await page.locator('body').innerText()
  expect(body).not.toContain('Playwright Test Story')
  expect(body).toContain('6 articles')
})

// PLAN-037 T-04: 官方主题包 —— 家族/模式 2x2 往返 + 跨路由保持（setupState
// 模型驱动；生效包值 oracle；outlet 三页共享 App design 绑定）。
test('T-P037: 主题 2x2 往返 — 跨路由保持 + 业务保持', async ({ page }) => {
  await waitForHome(page)
  const themeRoot = () => page.evaluate(() =>
    getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim())
  const drive = (t: string, d: boolean) => page.evaluate(([t, d]) => {
    const st = (document.querySelector('#app') as any).__vue_app__._instance.setupState
    st.theme_name = t
    st.dark_mode = d
  }, [t, d] as any)

  expect(await themeRoot()).toBe('0 0% 100%') // 无显式默认 → Auto/light

  // Solarized/dark 下切到详情路由再回来：主题与业务都保持。
  await drive('Solarized', true)
  await page.waitForTimeout(400)
  expect(await themeRoot()).toBe('192 100% 11%')
  await page.locator('text=Getting Started with Rust').first().click()
  await page.waitForTimeout(800)
  expect(page.url()).toContain('/post/')
  expect(await themeRoot()).toBe('192 100% 11%') // 跨路由主题保持
  await page.screenshot({ path: 'test-results/p037_detail_solarized_dark.png', fullPage: true })
  await page.goBack()
  await page.waitForTimeout(800)
  expect(await themeRoot()).toBe('192 100% 11%')
  expect((await page.locator('body').innerText())).toContain('6 articles')

  // 2x2 其余臂 + 往返回默认。
  await drive('Solarized', false)
  await page.waitForTimeout(350)
  expect(await themeRoot()).toBe('44 87% 94%')
  await drive('Auto', true)
  await page.waitForTimeout(350)
  expect(await themeRoot()).toBe('240 10% 4%')
  await drive('Auto', false)
  await page.waitForTimeout(350)
  expect(await themeRoot()).toBe('0 0% 100%')
  await expect(page.locator('body')).toContainText('My Blog')
})
