# 宿主主题传输与策略核心（host-theme-transport）

> 来源：auto-os/PLAN-053（2026-11-03 交付；本仓分支
> `codex/plan-053-host-theme-ui`）。宿主策略主合同在 auto-os
> `docs/specs/shell/host-theme-policy-and-delivery.md`——本页记录本仓
> （runtime/compiler/tools）交付的机制面，UI 端 031 选择合同不变
> （theme-scope-and-publication.md）。

## 策略核心（runtime `ui::theme_policy`）

- **单入口提交**：`stage_candidate`（完整校验 → 候选字段改写，含 legacy
  兼容映射与 revision 分配）→ 调用方 `desktop_config::save` →
  `publish_staged`（内存 revision 发布）。全等候选 `Unchanged`（比较不含
  rev——杀回环）；拒绝具名、零改动。bus 动词、config 外写轮询、standalone
  轮询三路共用。
- **目录发现**：`init_catalog_from_discovery` env 双源
  （`AUTOOS_THEME_PACKAGE_DIR` → `AUTO_UI_ROOT/packages/themes`），逐候选
  verify + digest，**无主检出 fallback**（缺席 = 降级具名拒绝）。
- **boot 恢复**：`boot_from_config` 从落盘策略恢复内存镜像（校验失败具名
  拒绝保持 None）。
- **评估登记**：`note_scope_eval/last_scope_eval`——eval_app_design 在
  publish 成功/各拒绝路径记录，宿主状态面 applied/rejected/pending 判定
  的唯一记录点。
- **digest 唯一实现**：`auto-ui-compiler::design_tokens::theme_package::
  theme_package_content_digest`（auto-ui-tools 委托；runtime 经 compiler
  消费——目录身份算法与包语义同址）。

## 线上面（desktop_protocol）

- **能力声明**：`Hello` 尾段追加 `theme_caps`（`theme-policy/1`）——空 =
  旧端线字节等价；宿主只向声明会话推送（旧客户端零未知帧）。
- **帧 17/18**（Control 通道，连接级寻址）：
  `ThemePolicyPush{revision, record}`（五段 FIELD_SEP 快照）→ 客户端
  喂 031 端口（`Component::on_theme_policy` → `set_host_theme_inputs`）→
  产帧评估 → `ThemePolicyResult{revision, applied|rejected|unsupported|
  stale, reason}`。宿主按连接记 `theme_pushed_rev`，低 revision 结果记
  `stale` 不覆盖。
- **组件面**：`Component::theme_capable/on_theme_policy/theme_policy_
  status` 缺省不支持；`DynamicComponent` 覆写（scope 在场 = 采纳者；
  031 端口喂入 + 帧后读回）。采纳者 `dark_mode` 模型位由已提交策略
  派生 mode 驱动（三写点：execute_set_theme/外写差异臂/allocate 播种）。

## 采纳与运行形态

- **adopt 零装载**：broker `adopt␟<name>` 外部 rqhost 客户端（自持 VM/
  组件）→ 宿主 `BrokerClient.adopt` 零装载激活（registry 不查，簿记壳
  组件）；rqhost daemon 采纳预算 15s（冷客户端自建后握手）。
- **standalone 跟随**：`arm_standalone_follow`（`AUTOOS_DESKTOP_CONFIG`
  env 门）→ 目录装载 + boot 恢复 + 400ms ServiceTick 订阅武装；轮询
  差异臂与宿主同入口。
- **状态自续拍**：shell `__theme_status` 含 pending 行时标记刷新脏位，
  eval 落定后下拍复检，落定即停。

## 边界（来源：PLAN-053 review G-1/G-2）

真实 OS 主题事件端到端 = 宿主侧具名 hold（本仓无写路径；`system_prefers_
dark` 只读消费已实证）；桌面组合式 outproc 推送端到端待载体 app（协议
两端点单测已闭）。
