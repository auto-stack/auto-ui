# 专用示例主题迁移支持面（specialized theme migration readiness）

> PLAN-038 交付合同（SD-01）；PLAN-045 接续（SD-03，§5）。B3 十二专用示例的
> **实测**支持面、证据边界、具名前置与后继分包。分类：feature ｜ 状态：active ｜
> 来源计划：docs/plans/038-specialized-examples-theme-readiness.md、
> docs/plans/045-media-demo-theme-migration.md
>
> 本文只记**实测结论**：每例结论仅在其实测覆盖面内成立；"无主题面"=未声明
> theme 状态/design 绑定，壳不随族/模式输入改变（实测），是后继迁移的第一步
> 工作项而非缺陷。研究归档 ≠ 迁移完成：B3 十二例在 038 生效时**仍未迁**；
> 045 已交付其中三例（019/020/030，见 §5），余九例仍未迁。

## 1. 支持面实测结论（C-01）

| 例 | 原门实测 | 主题面实测 | 结论 |
|---|---|---|---|
| 031-paint | 31/31（画/擦/填充/undo/redo/存取；P553-D1 未触发） | 无声明 state；文档像素哨兵跨进程+跨 theme 恒守 | go（有界；无主题面→补声明） |
| 041-auto-edit | 50/50（含磁盘回读/热重载/keymap/退出） | 无声明 state；语法着色 owner=runtime code_editor | go（有界；无主题面→补声明） |
| 027-file-manager | 真实 fs 链+坏路径拒绝（行/cwd 守恒）；套件 T4 触发器漂移具名 | `--theme` CLI Δ234；应用内 toggle 缺席（desktop-host 回写设计） | go（有界） |
| 029-photo-gallery | 无原 tests；夹具腿 scan 哨兵恒守 | seeding+应用内 toggle live Δ203 | go（有界；photo_root env-first 化） |
| 020-music-player | `--back-port` 覆盖下真实 scan/stream；原 test 入口 links 冲突具名 | seeding+handler 派发 live Δ216；ToggleDarkMode 无视图绑定（死代码） | go（有界；playback blocked 具名） |
| 025-sys-monitor | 7/11（退役页/文案陈旧具名；kill 只验对话框+取消） | seeding Δ236 | go（有界） |
| 019-video-app | vm-smoke PASS（设置面板交互；mock 词负控内建） | seeding Δ153；5 accent if-链=迁移主战场 | go（门户/VM 轨；vue e2e 未跑） |
| 024-charts | 12/19（484 后状态契约 7 腿不存在；tick 2.48 在门内） | 无声明 state→无主题面（luma 恒定） | go（有界；测试修定 owner 具名） |
| 026-database | 0/8 断言全失配（app 本体健康实证） | 无声明 state→无主题面（luma 恒 23.5） | go（主题=后继补声明） |
| 018-book-reader | 5/10：书卡→详情空渲染+进度回写/添加书/ThemeToggle 挂载失效（具名） | 逃生舱失效；现输入=生成器默认 html.dark | 有界 go（阅读链半通）；5 红具名 |
| 030-video-player | vm-smoke ALL PASS（诚实降级+mock 词负控）+assert-style 静态门 PASS；两轨均无 media API（无 back contract） | VM 降级面未测 | 有界 go（壳/静态门）；media 数据面与播放 blocked 具名 |
| 031-image-viewer | a2r 生成后端 7 编译错（该例形状触发；同族 020/025/018 编译 ✅）；`--server vm` 挂起 | 无实际腿 | **blocked**（双具名前置；静态=全 token 0 hex） |

## 2. 主题输入通道语义（C-02，实测口径）

- CLI `--theme`/`--accent`（Plan 458）覆盖 pac `theme:`；**直设 `AUTO_UI_THEME`
  env 会被 CLI 按 pac 解析重写**——覆盖 pac 默认必须走 CLI。
- seeding 只写**已声明**的 `dark_mode`/`accent_color` 状态（仅启动首帧；运行期
  状态翻转进入壳的路径=视图重解析）；未声明即无主题面（031-paint/024/026/041
  实测 luma/类指纹恒定）。
- VM 独立窗 toast 反馈不入快照树/像素（027 坏路径拒绝仅 state 守恒可见）——
  失败可见性以 state/oracle 为准（具名观察）。

## 3. 具名前置与后继分包（C-03；不取号、不自动实施）

1. **媒体播放/门户**（020/030/019）：前置=生成消费者归属根 links 冲突、030 无
   back contract（media API 两轨不可达）、mpv+合法夹具环境、020 死代码接线。
2. **图像与文件**（029/031-image-viewer/027）：前置=031iv a2r codegen 7 错
   （唯一硬阻断）；031iv 性能门条件保留（本机冷启未达具名）。
3. **阅读/编辑/数据/专用绘制**（018/041/026/025/024/031-paint）：前置=018 vue
   轨组件挂载/路由回渲染五红；四处测试定位修定；四例主题声明补齐。

共享资产（stylekit/remote-frame-e2e/032）按来源保持义务，不按业务 app 强套
主题；038-minesweeper 属 UI037 谱系不随本表。

## 4. 验收锚点

- 证据：PLAN-038 §5.4 矩阵 + plan-artifacts 探针日志（merge 时折叠）。
- 双向对照：同族后端 020/025/018 编译 ✅ × 031iv ✗（codegen 缺陷定位）；
  有/无声明 state 的 luma 响应对（029/020/025/019 vs 031p/024/026/041）。
- 不变量：文档像素/图像 URI/文件数据/编辑内容哨兵跨 theme 输入恒守（全绿例
  均实测）；原性能/压力条件（031iv 冷启、029 render_cap）不在本包缩样或重跑。

## 5. 045 接续结果（PLAN-045 交付，媒体播放/门户分包已闭合）

> 038 §1 表内 019/020/030 三行与 §3 第 1 项前置为本批按冻结反例
> 逐项处理后的当前状态；038 研究结论/历史原文保留，不覆盖。

| 例 | 038 具名差额 | 045 处理与结果（证据见 demo-theme-batch-045.md 与 plan-artifacts/plan-045） |
|---|---|---|
| 019-video-app | 五 accent if 链=主映射面；vue e2e 未跑 | 迁官方机制（text-primary/ring-primary + 色块字面色内容豁免）；迁移后 Vue e2e 10/10、vmtheme 4 fixtures ALL PASS、冻结工具 build exit 0 + receipt `9fa07742dc68` |
| 020-music-player | 原 test 入口 links 归属冲突；播放 blocked；ToggleDarkMode 无视图绑定（死代码） | links 冲突=旧测试定位与真实 UI 不符，按真实文案修定位（未删红断言）；播放腿受控 `<video>` + back scan/stream 206 真实播放 ALL PASS；ToggleDarkMode 按 README 承诺接线 NavSidebar「深浅主题」按钮 |
| 030-video-player | 两轨 media API 无 back contract；播放 blocked | 新增最小 `src/back/api.at` `/api/player/status`；Vue 受控 video 真实播放 + seek/4K/错误文案 e2e 14/14；VM 诚实降级文案保留，降级与真实播放分列 |

- §3 分包 1（媒体播放/门户 020/030/019）**已由本批交付**：links 冲突、030
  back contract、mpv+合法夹具、020 死代码接线四项前置按冻结反例闭合；
  无需新格式/新平台。剩余 B3 分包 2（图像与文件 029/031iv/027）与
  分包 3（阅读/编辑/数据 018/041/026/025/024/031-paint）仍按后继批次合同。
- 本批既有红交接：新工具链对 019 重建的 TS1005（auto-ui-tools vue.rs
  store_ctx_import 模板拼接，plan-041 引入，主检出同复现）为非 045
  生成器缺陷，owner=后继生成器修复批次；不随本批冒称关闭。
