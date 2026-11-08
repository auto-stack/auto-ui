# RenderQueue 连续媒体合同（PLAN-073 SD-01）

状态：已实施并实测（2026-11-09，PLAN-073 T-02/T-05/T-06；测量命令与输出目录见
文末）。本文件记录**实际能力与已测边界**，不预写未实现面。

## 1. 适用范围与所有权

- 所有者：auto-ui（renderqueue / rqhost / media 资源路径）。
- 消费方：OS063 全产品最终验收（M-02/06/07 物理面）、媒体类 App。
- 父合同：`rq-render-packets-and-resources.md`（资源身份/代际/槽信用）、
  `rq-fidelity-contract.md`（保真边界）。本合同只加**连续媒体**维度。

## 2. 媒体类资源类别（用户裁定 2026-11-08）

- 媒体类位图上传（`media_poster::is_media_frame` 在册帧）**豁免** per-App
  8 MiB/s 内容上传预算闸，改独立计量（`media_uploaded_bytes`，仅观测不设闸）。
- UI 内容类 8 MiB/s 预算**不变**；generation/释放/槽信用/驻留（256 MiB/App）
  语义不变。
- **槽几何规则（具名）**：媒体帧字节数 ≤ 4 × 宿主表面像素字节
  （`bm_slot_size = w×h×4×4+4`，host.rs）。1080p 帧需全高清级表面；
  超槽上传按构造拒绝（`Shm` 错误，可观测弃置）。

## 3. 连续流语义（已测）

- **身份**：媒体帧以内容寻址（blake3(w,h,rgba) → `bitmap://{rq-img-<seq>-
  <hash8>}`）逐帧过资源通道；同内容不重传（P-02 复用），异内容新身份。
- **节奏**：产者 tick 驱动重发布（实测 24fps 尝试全通过；release 单通道
  23.83fps / 188.5 MiB/s 实收，慢宿主 1:5 泵 23.79fps 合流零丢失）。
- **暂停**：产者停报（帧冻结最后已发布内容）；在途帧 ≤ 槽信用（2）自然
  排空；宿主保旧帧可解析（T-05 双宿主相位 ②）。
- **seek**：内容确定性跳变（同 n 同字节）；落地后旧身份不倒灌
  （no-backfill，T-02/T-05 双宿主相位 ③）。
- **重开**：新源新会话首帧重新落地（恢复语义，T-05 desktop 臂相位 ④）。

## 4. 端到端延迟与读回（V-06 冻结读数）

- **延迟分布**（debug，standalone 宿主，publish(tick on()) → 宿主内容首见，
  n=39 样本）：p50 **16.6ms** / p95 **18.1ms** / max **24.4ms**；冻结预算
  250ms（tick 粒度 41ms + 管线深度）。物理上屏延迟仍归 OS063。
- **独立读回**：宿主 put 时独立计算内容 blake3（`bitmap_content_hash_of`，
  驱逐同删）；验证方对产者 `frame_rgba(n)` 独立哈希对表——两侧不共享中间
  状态。资源身份即内容哈希（hash8 前缀），身份相同 ⇔ 内容相同。

## 5. 版本与能力

- 依赖能力：`resource/generation/1`（上传代际）、`welcome-caps/1`（协商）。
- 媒体类豁免不需要新 wire 原语（复用 BitmapReadyGen + 槽通道）；旧宿主
  （未协商 generation）按旧 tag 语义（即时翻新）——不承诺连续流代际保护。
- 冻结负控：冻结预算假实现 ≈1fps 必败（`p073_t02_media_class_exempt_
  continuous_1080p` 下界 12 帧/6s）、冻结 poster 假实现身份 ≤2 必败
  （`p073_t02_frozen_poster_fake_discriminated`）。

## 6. 测量命令与输出

- 命令：`cargo nextest run --locked -p auto-ui-runtime --lib
  --ignore-default-filter -E 'test(p073_t02) or test(p073_t05)'`
  （`p073_t02/p073_t05` 独占隔离窗，`.config/nextest.toml`）。
- 输出：`plan-artifacts/plan-073/auto-ui/`（t02-*.log、t05 运行日志；
  merge 折叠后删除，本文件即摘要）。
