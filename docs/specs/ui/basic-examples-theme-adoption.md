# 基础示例主题接入（basic examples theme adoption）

> PLAN-033 交付合同（SD-01/SD-03）。B1 剩余六例（001-helloworld、
> 004-profile-card、006-hero-section、007-stats-board、008-pricing-table、
> 009-article-feed）接入官方主题包的**逐例可验收**规则：接入形态、逐字段
> 颜色映射与豁免、双端驱动通道、隔离/优先级、包 receipt 与生命周期、B1
> 收口清单与 OS053 交接。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/033-basic-examples-theme-migration.md

## 1. 接入形态（C-01）

- 六例统一：pac `dep themes { path: "../../../packages/themes" }` +
  app `use themes.themes: Auto, Solarized` + `theme_name`/`dark_mode`
  模型态 + `design { theme: .theme_name, mode: if .dark_mode … }`（C-14
  形态；不复制调色板，不新增菜单/业务状态）。
- 独立默认：001/004/007 = Auto/light（原外观）；006/008/009 保留既有
  显式 pac `theme: "dark"` + 模型 `dark_mode = true`（031 契约：显式
  默认精确保留，不暗改成未声明 App 的 Auto/light）。
- 001/004/007 补声明的 `theme_name`/`dark_mode` 是主题选择态，非业务
  模型；原问候文本/字段/布局逐字节保持。

## 2. 逐字段颜色映射与合法豁免（C-02）

| 例 | 映射/豁免 | 依据 |
|---|---|---|
| 004 | 在线状态点 `bg-green-400` → `bg-success`（包 success 槽，Auto light `#22c55e`/dark `#4ade80`） | 状态语义色；闭 Plan-637「主题系统立项时评估回迁」悬案 |
| 004 | 顶部 `from-blue-500/to-purple-600` 渐变保持字面 | 装饰性插画（r2 口径），非语义面 |
| 007 | 四 stat chip（`bg-{red,blue,green,purple}-100`+同色 `text-*-500`）与三 avatar 底保持固定字面 | **分类身份色**：映射 success/error/info 会伪造状态语义（"Promotional Offers"≠成功态），包内无 purple 类目角色；两模式下与原字面一致（原 app 明暗两臂本就同串），双端截图+computed 断言为可读性证据 |
| 006/009 | 暗支渐变豁免维持（r2 口径） | 内容插画 |
| 全部 | 其余表面已是公共角色（bg-background/card/border/…），值随包数据 | Plan-637 B2 映射基底 |

## 3. 双端驱动通道（C-03；测试/宿主切换不新增菜单）

- **VM**：`autoui_fixture`（PLAN-623，`AUTOUI_TEST_FIXTURES=1`）状态写 +
  `{widget,event,input}`/`{handler}` 派发；006 的 `SetTheme`/`SetAccent`
  宿主传播消息面保持（宿主→内嵌 demo 既有通道，实测到达
  `dark_mode`/`accent_color`）。宿主适配端口 `set_host_theme_inputs`
  按 §5.2 优先级（override > follow > design 默认 > legacy）。
- **Vue**：setupState 模型驱动（`#app.__vue_app__._instance.setupState`
  写 `theme_name`/`dark_mode`——真实响应式模型层写入，VM fixture 的
  模型层等价物；不直接篡改 CSS/类）；模式每载切换经
  `AUTO_UI_THEME`（CLI/env 显式覆盖）与系统通道（standalone 页宿主
  即 OS，bootstrap 实时跟随 prefers-color-scheme）。
- **同帧族+模变更**：发射的 `watch(theme_name)` 按模型值取 isDark
  （PLAN-033 修复——DOM dark 类在 watch flush 批次内是渲染前旧值）。

## 4. 隔离、优先级与生命周期（C-04）

- 同进程多 App 各持 scope：相反主题互不串（真实源腿
  `plan033_b1_scope_isolation`：001 Solarized dark `#002b36` 与 006
  Auto dark `#09090b` 同帧共存）；宿主 override 生效/释放回 design
  默认；未注册族（override 或 design 值路径）整帧拒绝保旧；关闭重开
  得独立新 scope（不继承临时选择）。
- 同页（宿主页多 iframe）同规则：逐帧驱动相反主题无串色，单帧重开
  他帧不受影响。
- 投影面（PLAN-033 修复）：窗口投影路（`dynamic_view[_vwin]`）与
  view 构建路同持组件 scope——语义色转换读同一份快照；缺此安装时
  design 绑定族不达像素。

## 5. 包 receipt 与生命周期（C-05）

- 声明 `dep themes` 的 App 每次生成写 `<gen>/front/vue/.theme-inputs.json`
  （owner `auto-ui` / `themes` / `0.2.0` / 内容摘要）。六例 receipt 同
  一摘要（交付时 `9fa07742…`，031 未改包即恒定）。
- 生命周期（真实生成 e2e，001）：篡改（内容变更+保 mtime+清单未更）→
  生成准入具名拒绝；合法发布（内容+清单同步、mtime 保持）→ 摘要前移
  生成取新值；字节还原 → 摘要回原身份。
- 外部第二根消费由 [正式主题包](official-theme-package.md) 的
  `theme_external_package_build` 持续钉住（外部副本哨兵色真实解析）。

## 6. B1 收口与后继清单（C-06；不冒称全量）

- **B1 完成**：002/005（PLAN-007）+ 003/010（PLAN-031）+ 本包六例
  = 8 个基础示例全部接入官方包。
- **后继**（逐 app 独立状态，不以框架完成冒称）：B2 十二 +
  B3 十二共 24 应用；032 验证例与共享夹具（remote-frame-e2e、
  stylekit）按各自来源保持义务，不按业务 app 强套主题。
- **OS053 交接**：共享同版正式包（`themes` 0.2.0 + 内容摘要身份）/
  follow 优先级约定；本包六例 + theme-lab 即独立消费参考实现；OS053
  的 OS gitlink/持久化/系统事件按其自身合同，不以六例替代。

## 7. 验收锚点

- 双端逐例：`examples/ui/00{1,4,6,7,8,9}-*/tests/test_00X_{vm.py,vue.mjs}`
  （fixture/setupState 往返、print/资源/等高布局/固定字面 oracle）。
- 进程内隔离/优先级/重开：`plan033_b1_scope_isolation`（runtime）。
- 既有面回归：`example_theme_migration`（003/010）、
  `theme_lab_build`/theme-lab、002 实跑（计数/切族/切模保持）。
