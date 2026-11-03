# reader-native-surface-closure — 原生输入链与文档表面收口（PLAN-029）

状态：已交付（双 profile 聚合 exit0）｜来源：PLAN-029（2026-11-02 起草，
2026-11-02..03 执行）｜前置：[026 探针](reader-engine-probe.md)（其具名 GAP
由本计划闭合）｜实验 profile 边界仍以
[document-surface-boundary](../specs/runtime/document-surface-boundary.md) 与
[document-surface-native-input](../specs/runtime/document-surface-native-input.md) 为准。

## 1. 冻结 cohort（T-09 交付态）

| 项 | 值 |
|---|---|
| auto-reader | main @ 7b42765（运行时代码；驱动修复至 28c765a/e6d3d41/2d09938/8c843fd/78857d1/cc6cb7a/e7ef427/edf6f99） |
| auto-ui（UI worktree） | codex/plan-029-dev @ 98633fd4（doc-surface 弹出窗形态 + N-05 隐藏修复） |
| auto CLI | auto-man target/debug/auto.exe，version 0.1.0+plan-890-verified-r7-t19-14-g47d8f7b（sha256 6f1925ea…） |
| auto-ui-tool | worktree target/debug/auto-ui-tool.exe（PLAN-816 发现序第一键 cohort） |
| 引擎 | foliate-js fork@3d794195（受管扁平副本，plan-artifacts/plan-029/engine-assets） |
| 语料 | probe-alpha/probe-beta ≤1MiB EPUB（sha256 与 corpus-manifest 一致：d973dad8/88c25a1c） |
| 预算 | 15/10/5/30/10s 章节预算 + 600s 自动链（DPI 交互门独立 600s×2，§5.2 语义修订）不变 |

## 2. 根因与修复账（按发现序）

| 发现 | 根因 | 修复 | owner/commit |
|---|---|---|---|
| 026 NV-02 原生输入 GAP | driver_vm `_abs_xy` 用错 GetSystemMetrics 索引（坐标放大 ~65536×）+ 未声明 DPI-aware | 坐标/ DPI 感知修正（T-00） | reader c606850 |
| 资产装载失败 | prepare_engine_assets 扁平化未同步 view.js 引用 + `:name` 单段路由 404 | flat-vendor 重写一致化 | reader c606850 |
| R-05 直连 api.at 404 | 工具链侧（auto-man PLAN-007 迁移 app corpus 解析） | cohort 内工具链交付（g55c98c7→g47d8f7b 区间） | auto-man（非本计划仓） |
| R-06vm 坏图负控 rc=0 | 驱动 `generate_all` 无条件重写夹具，门注入的毁损被静默还原 | `overwrite=False` + 驱动 `--no-regen` | reader 6887ed0 |
| b-continues FAIL | 记录轮询 60s 窗 < 适配器串行 occurrence 计数延迟（页内阻塞数十秒，T-07 在案） | 轮询窗参数化（b-continues 120s） | reader 2d09938 |
| dpi restore/track FAIL | oracle 公式混用切换前 vw × 新 scale（错位 216px）——弹窗实际正确跟随**窗口比例** | oracle 改窗口相对比例不变式（popup_w/win_client_w） | reader 2d09938 |
| N-05.reflow vue GAP | 页面列 max-w-2xl(672) 钳制使 resize 未破钳；字号注入/gap 属性不经分页器观察 | 跌破钳制的 resize（viewport 560）：seam 627→512 真实重排 | reader 28c765a |
| N-04 合成焦点往返盲区 | CDP `document.hasFocus()` 不随外部 SetFocus 迁移（Chromium 焦点信念）+ iced 组件焦点不随 OS 焦点恢复 | §10 既定配对协议：CDP 断言回执 + 在案人工目击（T-04 轮用户手打 1231）组合判定 | reader 7b42765 |
| MCP 裸调用被挂起打断 | 输入框 oninput 键入期间 MCP 服务线程阻塞（实验 profile 已知族） | `_mcp_text` 有界重试 + N-04 容错值读取 | reader cc6cb7a |

## 3. 必需用例终态（T-09 冻结跑）

入口：`python tests/probe/engine_surface/verify_engine_surface.py
--profile vue|vm --auto-bin <auto> --out-dir <隔离>`。
**vm：92 记录 aggregate_ok=True exit0（914s，t09-freeze-final）；
vue：findings=0 exit0（t09-freeze-final-vue）。两端旧门同跑内嵌：
verify_standalone vue 28/28、vm 27/27。**

| Case | 内容（输入→预期→实测） | 终态 |
|---|---|---|
| N-01 | 同宿主同文档 CDP 输入管线 vs SendInput 双通道拖选；焦点/捕获/命中逐通道取证 | 双通道 PASS + pair PASS |
| N-02 | 真实拖选第 2 处重复句（中文/emoji/跨行）→ .at accepted → CFI → oracle occ=2 → sidecar HTTP+磁盘 | vm/vue 双臂 PASS |
| N-03 | 正常关闭+强杀 → 新进程冷恢复同出现位置；旧记录/身份不变 | vm/vue PASS |
| N-04 | 真实 MS Pinyin 组合（'nihao'+Space→中文落定）/Esc 取消/组合期 Enter 仅提交原文/焦点往返（配对协议：CDP hasFocus 断言 + 在案人工目击） | 四子项 PASS |
| N-05 | resize 跟随/字体重排/滚动重拖/遮挡像素三段/隐藏展示/交互 DPI（track+redrag+restore 窗口比例 oracle） | vm 全腿 PASS；vue（resize 重排/repage/遮挡/隐藏）全 PASS |
| N-06 | 系统无障碍树（UIA）定位正文 + UIA SetFocus + 真实 ArrowRight/Left 翻页往返 | PASS |
| N-07 | A/B 双实例不同书/会话；A WM_CLOSE 关闭后旧事件拒绝零写入；新 A 不接旧会话；B 跨扰动继续真实选择保存 | 全子项 PASS |
| N-08 | WM_CLOSE 真实关闭 10s 释放/取消隐藏/崩溃无孤儿/坏 tool 具名失败/占口不杀 | 全子项 PASS |
| N-09 | 错 book/hash/session/schema/kind/quote/occurrence/oversize/stale-seq 逐类拒绝 + 字节级零写 + 合法对照接受；href 不透明语义 + 资产逃逸零泄漏 | 全子项 PASS（双 profile） |
| N-10 | 聚合器受控负例 10/10 非零 + 坏 tool 不回落 + 占口不杀 | PASS |
| N-11 | 磁盘变异已存 locator → oracle 实红 → 恢复落点异位（位置指纹）→ 字节级还原绿 | vm/vue PASS |
| N-12 | vue 既有链隔离重跑（34 记录）+ R-05 verify_epub 22/22 + 两端旧门（28/28、27/27） | PASS |

## 4. 边界与后续（不超 claim）

1. **配对协议**：合成注入的焦点往返盲区（Chromium 焦点信念不随外部
   SetFocus 迁移 + iced 组件焦点不随 OS 焦点恢复）按 §10 既定协议以
   「CDP hasFocus 断言 + 在案人工目击（T-04 轮用户手打 1231）」组合补证；
   门记录透明注明两半来源。
2. **MCP oninput 阻塞**：输入框键入期间 MCP 服务线程挂起（实验 profile
   已知族）——门读取已容错重试；产品化需修复（候选后续计划）。
3. **分页器观察边界**：body 字号注入/gap 属性不经分页器重算（巨列形态
   innerW 异常）——vue 重排走 resize 通道；字号变体由 vm 轨覆盖。
4. 大书/真实出版物/其他格式/PDF/漫画/TTS：未验证（非本计划目标）。
5. 双实例 seq 乱序拒绝依赖到达序（适配器串行计数阻塞同族）——已记录。
