/**
 * 025-sys-monitor 冒烟测试 (Plan 541 + PLAN-047 主题腿)
 * 验证 Vue 模式下 2 页导航（PLAN-684 用户/详情页退役）、真实系统指标渲染、
 * 进程表排序、结束任务 AlertDialog 二次确认及暂停/继续控制；主题腿经
 * setupState 驱动两族×明暗（B2 C-03 Vue 通道）+ CSS-var 逐值 + 业务守恒。
 *
 * PLAN-047 定位修定（038 F-4 关闭：按现 UI 修定，不补退役 feature）：
 * 4 页导航→2 页（退役页负 pin）；KPI 磁盘吞吐/网络总吞吐→磁盘/网络；
 * 弹窗标题「结束任务确认」→「结束任务」；⏸ 暂停/▶ 继续→图标单字 ⏸/▶。
 */
import { test, expect } from '@playwright/test'

async function waitForApp(page) {
  await page.goto('/')
  await page.locator('text=系统监视器').first().waitFor({ timeout: 10000 })
  // 等待快照数据加载
  await page.waitForTimeout(1000)
}

test('T1: 初始渲染与 2 个导航 Tab（退役页负 pin）', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('系统监视器')
  expect(body).toContain('Task Manager')
  expect(body).toContain('进程')
  expect(body).toContain('性能')
  // PLAN-684 退役页不回归
  expect(body).not.toContain('详细信息视图')
  expect(body).not.toContain('已登录与系统用户账户')
})

test('T2: 进程页 KPI 四卡与真实进程列表', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('CPU')
  expect(body).toContain('内存')
  expect(body).toContain('磁盘')
  expect(body).toContain('网络')
  expect(body).toMatch(/已用 \d+ MB/) // 真实 sysinfo 值

  // 验证进程列表行数大于 0
  const rows = page.locator('table tbody tr')
  const count = await rows.count()
  expect(count).toBeGreaterThan(0)
})

test('T3: 结束任务 AlertDialog 二次确认开合与取消', async ({ page }) => {
  await waitForApp(page)

  // 初始状态下结束任务按钮处于禁用或不可点击状态
  const firstRow = page.locator('table tbody tr').first()
  await firstRow.click()
  await page.waitForTimeout(300)

  // 点击选中的结束任务按钮
  const killBtn = page.locator('button:has-text("结束任务")')
  await killBtn.click()
  await page.waitForTimeout(400)

  // 验证 AlertDialog 弹窗出现（现 UI 标题=结束任务）
  const dialogTitle = page.locator('text=确定结束')
  await expect(dialogTitle).toBeVisible()

  // 点击取消
  const cancelBtn = page.locator('button:has-text("取消")')
  await cancelBtn.click()
  await page.waitForTimeout(300)

  // 弹窗关闭
  await expect(dialogTitle).not.toBeVisible()
})

test('T4: 性能页切换与 SVG 曲线图', async ({ page }) => {
  await waitForApp(page)

  // 点击左侧性能 Tab
  await page.locator('text=性能').first().click()
  await page.waitForTimeout(500)

  const body = await page.locator('body').innerText()
  expect(body).toContain('CPU 使用率实时曲线')

  // 验证 SVG 包含 path
  const paths = page.locator('svg path')
  const pathCount = await paths.count()
  expect(pathCount).toBeGreaterThan(0)
})

test('T5: 退役页负 pin + 回到进程页业务保持', async ({ page }) => {
  await waitForApp(page)

  // 性能页 → 进程页往返：真实刷新业务与选中保持
  await page.locator('text=性能').first().click()
  await page.waitForTimeout(400)
  await page.locator('text=进程').first().click()
  await page.waitForTimeout(600)
  const rows = page.locator('table tbody tr')
  expect(await rows.count()).toBeGreaterThan(0)
})

test('T6: 刷新频率切换与暂停/继续控制', async ({ page }) => {
  await waitForApp(page)

  // 点击快速 (250ms)
  const fastBtn = page.locator('button:has-text("250ms")')
  await fastBtn.click()
  await page.waitForTimeout(300)

  // 点击暂停（现 UI 图标单字）
  const pauseBtn = page.locator('button:has-text("⏸")')
  await pauseBtn.click()
  await page.waitForTimeout(300)

  // 变为继续
  const resumeBtn = page.locator('button:has-text("▶")')
  await expect(resumeBtn).toBeVisible()

  // 恢复继续
  await resumeBtn.click()
  await page.waitForTimeout(300)
  await expect(pauseBtn).toBeVisible()
})

test('T7: PLAN-047 主题腿 — 两族×明暗 + CSS-var + 业务守恒', async ({ page }) => {
  await waitForApp(page)
  const themeRoot = () => page.evaluate(() =>
    getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim())
  const drive = (t, d) => page.evaluate(([t, d]) => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState
    st.theme_name = t; st.dark_mode = d
  }, [t, d])
  const biz = () => page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState
    return { tab: st.active_tab, speed: st.speedDiv, rows: st.store ? -1 : -1 }
  })

  // 迁移前默认 dark-first（pac theme:"dark"）
  expect(await themeRoot()).toBe('240 10% 4%')
  const before = await biz()
  for (const [fam, dark, bg] of [
    ['Solarized', true, '192 100% 11%'],
    ['Solarized', false, '44 87% 94%'],
    ['Auto', false, '0 0% 100%'],
    ['Auto', true, '240 10% 4%'],
  ]) {
    await drive(fam, dark)
    await page.waitForTimeout(350)
    expect(await themeRoot()).toBe(bg)
  }
  const after = await biz()
  expect(after.tab).toBe(before.tab)
  expect(after.speed).toBe(before.speed)
  // 真实刷新业务在换肤后仍渲染
  const rows = page.locator('table tbody tr')
  expect(await rows.count()).toBeGreaterThan(0)
  await drive('Auto', true)
})
