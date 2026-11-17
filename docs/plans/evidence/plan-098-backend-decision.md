# PLAN-098 判定 artifact（T-01 诊断 / T-04 后端决策 / T-06 复现轮甄别）

日期：2026-11-16。工具链：ui-098 组构建（auto-man `54fe407` + auto-ui
worktree `plan-098-dev`@T-02/T-03/T-05 落地后，doc-surface feature，
debug profile）；对照：reader-013 cohort（PLAN-081 时代源码 `7823ced3`）。
环境：**设计值 env**（`AUTO_HTTP_REQUEST_TIMEOUT_MS=30000` /
`AUTO_HTTP_STEP_BUDGET=10000000`，经脚本 setdefault 语义外部覆盖），
`WGPU_BACKEND=dx12`（run_vm.rs session_env 注入，T-04 候选 A）。
原始物：`D:/autostack/plan-artifacts/plan-098/`（t06_run1~3.log、
cohort_control_run.log、各 t08run-*/ 窗口与监视日志）。

## T-04 后端决策：候选 A（dx12）采纳

- 停摆态栈实锤楔死在 Vulkan 路径（ash wait_for_fences ← wgpu
  acquire_texture，诊断转储 44964）；dx12 = 最小改动预防面（仅
  vm-session 子进程 env，不污染全局；用户显式 `WGPU_BACKEND` 不覆盖）。
- **本批 dx12 ×3 轮：0 次 30s+ 冻结**（历史 Vulkan 工具链 5/5 轮含冻结，
  诊断轮实录；注意基线为旧构建，非同批 Vulkan 对照臂——局限记录在案）。
- 帧心跳（T-05）已挂：`[SCHED-DIAG] frame_seq` 每秒签名 +
  `FRAME_SEQ` 原子；未来任何楔死 5 分钟内可定位。
- 候选 B（present_mode）/C（present 补丁超时）无需启动；C 的受限分析
  （acquire 楔死在主线程内无法同线程自救）保留于计划 §5.3。

## T-01 诊断判定：按计划 §5.1 降级依据收口

- 三轮 vm 复现**零楔死**（无停摆态可抓）→ H1（primary 漂移 take-once
  死亡）未获日志直证；按计划预置的降级依据收口：F1-F5 静态结构
  （take-once + 订阅身份 = BTreeMap 最小 AppId + apps.remove 漂移面）+
  **T-01b 漂移模拟单测**（弃 stream → 断口期发 3 条 → 重挂按序补交；
  旧实现在此序列确定性全灭）+ 在线证据：三轮全部搅动面（boundary
  重启 ×2/轮、冲突横幅、抽屉 ToggleFp）注入链全程健康（窗口日志
  `[VM-PARKED]` park/resume 全部完成，零 park 线停摆）。
- `[PLAN098]` 诊断面（seq/delivered/mapped_none/停滞标志 +
  `ActionResult.delivery`）已常驻——未来任何注入异常可直接判别。

## T-06 三轮甄别矩阵（核心结论）

| 维度 | cohort 旧工具链 | ui-098 新工具链（本计划修复） |
|---|---|---|
| 30s+ 冻结 | 历史 5/5 轮（诊断轮） | **3/3 轮零冻结** |
| 进程静默退出（wgpu panic） | panic 实录在案 | **3/3 轮零 panic** |
| 事件注入丢失（park 线停摆） | U2b/U3a 冻结族实录 | **3/3 轮零丢失**（全 park/resume 完成） |
| R-03 boundary 楔死族 | **boundary re-enter failed 直接崩**（对照轮实录） | 全腿推进（含 U3a run3 诚实报错 PASS） |
| R-03-U2b/U2c/U2d（冲突横幅腿，诊断 run4 冻结重灾区） | 未达 | run1 全 PASS；run3 U2b/U2d 摇摆红 |
| 整轮完成度 | R-03 中途崩 | run1/3 全腿推完（run2 止于 R01，见下） |

### 残红甄别（三轮 + 对照轮，全部**仓外域**）

1. **WebView2 CDP 起动摇摆**（run2 首窗 180s 不可达；run1 boundary 窗
   不可达；run3 全通）：doc-surface/wry 域——非三模式；cohort 连
   boundary re-enter 都崩，同域既有不稳定。计划 §1 边界外。
2. **reader 恢复语义摇摆**（U2b banner 时序 1/3 红、U2d 1/3 红、
   U3c/U3e 恢复落库 rows=0 2/3 红）：reader 应用（.at）+ notes 域——
   计划 §1 边界条款明文「.at 层传输分类/错误投影/检查点恢复均已实现
   且 vue 轨验证（READER-013 修复在案）」；notes read-snapshot stall
   （第一现场，诊断已定 owner ① notes 仓）在对照轮与本批均有实录
   （脚本自带 `restarting notes for truth` 处置）。auto-ui 运行时侧
   义务（事件送达/渲染/park-resume）日志证据完整。
3. **notes 域瞬态**（`notes read-snapshot stall`，两轮实录）：notes 仓。

### 结论

- 本计划三失效模式修复目标（G1-G3）**达成且证据充分**；
- AC-04 前半（「R-01～05 全绿」）依赖上述仓外域修复，**本仓内不可
  达成**；AC-04 后半（「notes kill 场景 native UI 保持可交互、错误
  投影可见」）已达成（run3 U3a 诚实报错投影 + 全轮可交互零冻结）。
- 处置建议（待用户裁决，见计划 §10 Q5）：修订 AC-04 为本仓义务面
  （三模式零复现 ×3 + 可交互 + 诚实投影 + 本甄别矩阵），残红按域
  立债/移交（WebView2 CDP 摇摆 → auto-ui doc-surface 新债；恢复语义
  → reader/READER-013；notes stall → notes 仓既有 owner）。
