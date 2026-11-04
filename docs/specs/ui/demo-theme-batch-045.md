# B3 媒体门户与播放器主题接入（demo theme batch 045）

> PLAN-045 交付合同（SD-01）。B3 媒体门户/播放器三教学示例
> **019-video-app、020-music-player、030-video-player** 的主题迁移实证：
> 逐例正式包接入形态、既有默认、内容豁免、原教学业务与支持轨守恒、
> 038 具名阻点闭合结果与后继交接。分类：feature ｜ 状态：active ｜
> 来源计划：docs/plans/045-media-demo-theme-migration.md

本文只记 045 实际执行证据（探针日志在 plan-artifacts/plan-045/，
merge 时折叠）。**已迁移 ≠ 全部 B3**：本批仅覆盖上表三例；其余 B3
示例（018/024/025/026/027/029/031-image-viewer/031-paint/041）仍按
[专用示例主题迁移支持面](specialized-theme-migration-readiness.md)
的后继批次合同，未随本批冒称关闭。

## 1. 三例接入形态（C-13/C-14 落地）

| 例 | 接入 | 既有默认 | 阻点处理 |
|---|---|---|---|
| 019-video-app | `dep themes { path: "../../../packages/themes" }` + 根 widget `use themes.themes: Auto, Solarized` + `var theme_name str = "Auto"` + `design { theme: .theme_name, mode: if .dark_mode {...} }`；主题魔法变量自 VideoStore 上移根 widget | `theme: "dark"`、`accent: "pink"`（官方表外 → 回退 indigo，保留既有应用内 accent 状态） | 五 accent if 链迁移：logo 文本 → `text-primary`（applyAccent 驱动 --primary），色块按钮本体字面色保留（选择器内容豁免）、选中态 ring → 语义 `ring-primary`；Vue e2e 补跑 10/10 |
| 020-music-player | 同 dep/use/design 形态（根 widget 原已有 dark_mode/accent_color 魔法变量） | `theme: "dark"`、`accent: "emerald"`（回退 indigo，保留） | **ToggleDarkMode 死代码接线**：NavSidebar 补「深浅主题」按钮走既有 msg；播放腿真实验证（受控 `<video>` + back scan/stream 206，E:\Music 小样本） |
| 030-video-player | 同 dep/use/design 形态（根 widget 原已有魔法变量） | `theme: "dark"`、`accent: "indigo"`（合法） | **两轨 media API 无 back contract**：新增最小 `src/back/api.at` `/api/player/status`（后端端口可用）；Vue 受控 video 真实播放（caelestia.mp4）+ seek/4K/错误文案；VM 诚实降级文案保留 |

## 2. 内容豁免与装饰（口径沿用 037 §SD-01 C 系）

- 图像上叠字 `text-white`（缩略图/封面/元信息条恒白）：内容域。
- 封面渐变 `from-emerald-600` 等（020 stage.at 黑胶唱片面 `bg-zinc-950`、
  唱臂 `bg-slate-*`）：装饰豁免，不随主题。
- 030 viewport `bg-black`/`text-white`/`text-amber-400`（画面区与叠字）：
  内容域；VM 降级面为诚实降级文案而非主题面。
- 语义状态色（020 rose/emerald 徽章、019 分类/品牌色）：保持，不重染。

## 3. 原业务守恒实证（AC-02）

- 019：内存种子 12 条/5 分类、路由 / + /watch/:id、list_videos/get_video/
  add_view/toggle_like/categories、SettingsPanel（深浅 + 5 色块 + 关闭）
  全部保持；分类选中态跨换肤保持（vmtheme 探针）；e2e 10/10 含业务流。
- 020：原队列/选曲/播放时间/音量、scan/stream 206、空库诚实诊断
  （`empty-020.log`）保持；真实播放验证 `play-020-v2.log` ALL PASS；
  原 VM 测试漂移（links 归属冲突，038 具名）按真实 UI 文本修定位
  （深浅主题/暂停/顺序播放/收藏歌曲），**未删红断言**。
- 030：viewport/队列/OSD/seek/音量/原快捷键保持；降级与真实播放分列；
  e2e 14/14（含真实播放/seek/4K/错误文案）；assert-style 迁移后 PASS。

## 4. 两族 × 明暗真实可见（AC-01）

- vmtheme 四 fixtures（solarized_light/solarized_dark/auto_dark/auto_light）
  三例 ALL PASS（VM 轨真实 apply/快照）；Vue 轨主题注入/深浅切换
  （计算样式/根类变化）019 `vuetheme-019.log`、020 `vue-020.log` PASS。
- V-04 两实例隔离：`isolation-019-020.log` ALL PASS——020 切 light、019
  保持 dark，互不串扰；公共面变、内容不乱染。

## 5. 来源与既有红交接（AC-04/05）

- 三例 theme 输入同一官方包（worktree `packages/themes`，构建时经
  junction 链接到各例 `deps/themes/`→本树单源，无外部下载）；篡改拒绝/
  恢复/910 隔离证据见 `plan-artifacts/plan-045/auto-ui/t05-source-freeze-and-isolation.md`
  （该 scope 于 merge 清理，核心结论折入本 Plan §9 收据）。
- **既有红（非 045 引入）**：新工具链对 019 重建触发 auto-ui-tools vue.rs
  `generate_main_ts` 拼接缺陷（plan-041 store_ctx_import 模板缺换行 →
  TS1005），主检出未迁移 019 同复现；owner = 后继生成器修复批次。045
  交付证据（冻结工具 build exit 0 + receipt `9fa07742dc68`、终态 e2e 10/10）
  不受影响；020/030 新工具 build 全 PASS。
- 计数：本批 3 例迁移后，B1+B2+B3(045)= **25 例已迁**；B3 余 9 例
  （018/024/025/026/027/029/031-image-viewer/031-paint/041）归 046/047 等
  后继批次。真实产品（OS055–061 谱系）独立演进，不随本批。
