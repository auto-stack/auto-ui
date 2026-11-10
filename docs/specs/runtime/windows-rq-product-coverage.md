# Windows RQ 产品投影覆盖与公开入口闭合（PLAN-077）

本页是 Todo/Video 产品覆盖拒绝根因修复、两种执行形态能力准入、负控与
OS066 消费交接面的 canonical 规范。来源：PLAN-077（r1）的实施与实测。
承接 [RQ 执行/输入/宿主服务](rq-execution-input-and-host-services.md)
的公开入口（本页管**覆盖判定与能力准入**），与 [保真合同](rq-fidelity-contract.md)
[资源重放](rq-render-packets-and-resources.md) 管载荷/资源。**未在本页写明的
行为不视为已实现。**

## 1. 背景与根因（OS063 战役遗留）

OS063 战役（PLAN-063 r4 有限交付归档）实测：todo/video 在客户端覆盖门即拒，
不到 attach——`todo: style:list-none`；`video: style:native-unstyled,
style:stroke-1, tag:video`（PLAN-075 T-03 实测，2026-11-10）。本页承接该
拒绝清单的根因修复与能力准入闭合。

| 拒绝项 | 归属 | 处置 |
|---|---|---|
| `style:list-none`（Todo） | **覆盖表差额**——native `View::List` 渲染无 marker（列表项直排，无圆点/编号），list-none 语义 = 移除 marker = 真实渲染面成立（style_parity.rs:308 断言在册）；`native_style_token` 映射在册（ListNone→"list-none"）但 `style_prefixes` 漏登 | **精确放行**（native_queue_set 与 target_set 同册） |
| `tag:video`（Video） | **真 not-yet（queue 投影面）**——`native_projector::layout_view_node` 无 Video 臂（`other =>` 占位盒 + `uncovered` 留痕，防静默错绘）；渲染能力 inproc 已具备（mpv→持久纹理→shader widget，PLAN-617 T-19，element_coverage.rs:539 登记"差投影不差渲染"） | **拒绝保持**（queue 面）；产品正确公开形态 = remote |
| `style:stroke-1`（Video） | **真 not-yet**——StrokeWidth（lucide 图标描边）native 位图/字形通道 not-yet | **拒绝保持** |
| `style:native-unstyled`（Video） | **fallback 未映射类**（如 ObjectFit 等未入 `native_style_token` 的变体 → `_ =>` 兜底名） | **拒绝保持**（未知类拒绝契约） |

## 2. 两种执行形态与能力准入（C-01）

公开入口（client_entry.rs）两臂：

| 形态 | 覆盖门 | 帧源 | Todo | Video |
|---|---|---|---|---|
| remote（`--render-mode rq\|desktop` / `--render=remote`，`ClientOpts.remote=true`） | **不适用**（iced 全集，构造保证） | `HeadlessFrameSource`（DisplayList v2 产线） | 可用 | 可用（mpv widget 真渲） |
| queue（非 remote Commands：显式 `queue` / Auto 缺省——`effective_frame_mode` Auto 恒 Commands） | `RqProjector::ensure_covered()` 权威拒绝（NotCovered = `[render] <gate>` + Err 退出留痕，禁静默错绘） | `RqProjector`（DrawList v1） | **可用**（list-none 放行后 Covered） | **拒绝**（`tag:video` 等，保持） |

- Auto 缺省统一走 native 门（PLAN-033 T-04 后解释轨不再降级 independent）：
  NotCovered 由臂启动覆盖门权威拒绝。
- 缺 Host/未知能力具名拒绝（desktop 端点缺席 = 报错含端点名与"先启动虚拟
  桌面"提示；未知 `AUTO_VM_RENDER` = 观测行 + legacy queue）——既有合同，本页不重写。
- direct/显式 legacy 兼容不被误改：独立窗缺省、显式 queue 拒绝语义、remote
  显式选用均保持（I1 零删除不变式）。

## 3. 覆盖表增量（C-02，2026-11-10）

- `Coverage::native_queue_set().style_prefixes` 增 **`"list-none"`**（精确
  token，非 `"list-"` 通配——list-disc 等 marker 渲染未接必须保持拒绝）。
- `Coverage::target_set().style_prefixes` 同册（解释态参考表防 drift）。
- 负控（永久反例 `p077_*`，auto-ui-runtime lib）：
  - `p077_todo_list_none_covered`：Todo 同构 List + ListNone → Covered；
    `p077_list_disc_still_rejected`：list-disc/list-inside/list-outside/
    list-decimal 拒绝（marker 渲染未接）。
  - `p077_video_tag_styles_gate_kept`：Video + StrokeWidth + ObjectFit →
    NotCovered 含 `tag:video`/`style:stroke-1`/`style:native-unstyled`。
  - `p077_video_product_queue_gate_rejects`：Aura `<video>` 同构 queue 臂
    拒绝信息含 `tag:video`。
  - `p077_remote_mode_skips_coverage_gate`：remote 形态无覆盖门观测行。
  - `p077_todo_product_queue_renders`：Todo 同构（input/button/checkbox +
    list-none）过门 + 帧非空 + 业务消息驱动帧文本更新。
  - `p077_video_remote_assembly_renders`：媒体同构 probe 经 HeadlessFrameSource
    remote 装配渲染 v2 帧含媒体内容。
- 命令：`cargo nextest run --locked -p auto-ui-runtime --lib --ignore-default-filter
  -E 'test(ui::desktop_protocol::coverage::) or test(p077_)'`。

## 4. 来源与限制（OS066 交接）

- 产品 pin（冻结源）：`auto-todo@0510e05`（`src/front/todo_list.at:63`
  `style: "list-none m-0 p-0"`，SHA256 D2B15716…）；`auto-video@0922b44`
  （`src/front/viewport.at:53` `<video>`，SHA256 38BD6F69…）。
- OS066 消费：Todo 产品公开形态 = queue/Auto（list-none 放行后过门，RqProjector
  真渲）；Video 产品公开形态 = **remote**（`--render-mode rq|desktop` /
  `--render=remote`，HeadlessFrameSource + mpv widget 真渲）；Video queue 形态
  保持覆盖门拒绝（防静默错绘，非缺口）。
- 限制（不视为已实现）：真实 Todo 业务完整语义（新建/编辑/完成 + 持久化
  API）与真实 mpv 播放/seek 的最终矩阵归 OS066 全 Demo/产品验收；本页交付
  两产品与同根因最小实际消费者（进程内 harness + 产品同构）。
- 全链重建（auto 生成 → 两 Host 真实运行）依赖 Man/产品链同步重建（旧
  `auto.exe` plan-890 不同源，不冒充当前行为）——OS066 按上表命令/形态消费。
