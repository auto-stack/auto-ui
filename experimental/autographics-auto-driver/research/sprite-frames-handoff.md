# PLAN-004 交付说明（work handoff，r1）

工作树：`D:/autostack/.wt/ui-004/auto-ui`，分支 `codex/plan-004-dev`，base `d3bd6b4`。
计划：`docs/plans/004-autographics-public-images-and-sprite-frames.md`（主检出簿记）。
合同：`docs/plans/attachments/004/work/sprite-resource-contract.md`（本地证据，含 T-02..T-05 实测）。

## 1. 交付内容

| 任务 | 结果 |
|---|---|
| T-01 | 核 baseline/两锁/SDK，建唯一树；`cargo check --all-targets` 绿 |
| T-02 | 冻结 `autographics.sprites2d/1`（profile 20 / number 109）与合同；真实 Auto 整组合 probe 通过 |
| T-03 | PNG 准入/预算、scene/5 绑定、项目闭包（`assets/<sha>.png`）、编辑导入/绑定/解绑；V-01 |
| T-04 | AGP 纹理绘制（nearest/clamp、UV-only flip、sRGB）、故事化像素 oracle；V-02/V-03 |
| T-05 | Pause/Step/Stop/新局、prepare 失败整组 last-good、20 轮 texture owner；V-04 |
| T-06 | 本文件 + SD 草案 + `verify_plan004.py` / `check_sprite_frames.py` / 独立 consumer |

关键实现：`api/sprite2d_v1.at`、`src/authoring/runtime_sprites.rs`、`src/authoring/assets.rs`（PNG 载荷）、`document.rs`（scene/5）、`storage.rs`（PNG blob）、`session.rs`（`Edit::SpriteObject`）、`play.rs`（sprite 块）、`host/viewport.rs`（命令构造）、`host.rs`（prune/owner）、`host/commands.rs`+`panel.rs`（导入/绑定 UI）；AGP `renderer/sprite.rs`、`assets/png.rs`、`host/surface_runtime.rs`。

## 2. 复现与结果

全部在 worktree 内、单实例串行执行（GPU 用例不并发）：

| 命令 | 结果 |
|---|---|
| `cargo check --locked --all-targets`（driver 与 AGP） | 绿；改动文件 0 warning |
| `cargo fmt -- --check`（两包） | 干净 |
| `cargo test --locked --lib`（driver / AGP） | 33 / 30 passed |
| `cargo test --locked --test public_sprite_frames -- --test-threads=1` | **15 passed**（连续 3 次 + 最终一次） |
| `cargo test --locked --test sprite_resources -- --test-threads=1`（AGP） | 3 passed |
| `cargo test --locked --test runtime_triggers_api` | 14 passed（旧 profile 定向面） |
| `cargo test --locked --test authoring_assets/host/play_host` | 17 / 7 / 7 passed |
| `cargo test --locked --test authoring_hierarchy/document/session/storage` | 10 / 1 / 4 / 3 passed |
| `cargo test --locked --test surface_contract` / `surface_review_r2` / `asset_resources` | 29 / 5 / 1 passed |
| `python tools/verify_plan004.py` | 1429 hashes；PASS |
| `python tools/verify_plan004.py --controls` | 编辑/删除/未列/错 pin 全部拒绝；PASS |
| `python tools/check_sprite_frames.py --controls` | 5 个 case + 像素/预算/暖帧/partial/foreign runner 控制；PASS |
| `python tools/verify_source.py` | PLAN-004 身份 + SDK 681 files + 479 包闭包；PASS |
| `python tools/package_consumer.py <B>` → B 内 `verify_plan004.py` + `cargo test --test public_sprite_frames` | 见 §4 |

## 3. 实测数字（debug，单实例）

- 组装 managed 源 80367 字节（api 76518 / entry 831 / transport 3018），合并 80418；上限 96KiB 未动。
- 精灵块：2 个绑定时 74 个读取；上限 116。
- 真实 Auto 整组合：compile ≈200ms；tick 中位 5.5–5.6ms、P95 8.5–9.1ms（同进程混跑重载时曾测到 52–61ms，属测量污染）。
- 纹理：96×32 图上传 12288 字节，20 轮 Play/Stop 只创建 1 张。

## 4. 独立 consumer

`python tools/package_consumer.py D:/autostack/.wt/ui-004/consumer-004` 导出 2111 文件 / 41.4MiB（源码 + `.sdk-source` + 两个 Cargo.lock，无 target/rlib）。B 内独立跑身份校验与 `public_sprite_frames` 套件（结果见 §2 最后一行与 Plan §9 收据）。B 自带 target，不复用生产者构建产物。

## 5. 覆盖边界与未决门（merge 所有）

- GPU 证据是**无窗口真实内容**（`probe_pixel` 读回），未做 OS 抢焦点/置顶/真实键鼠自动化；manual CLI 路径验收按计划仍 pending。
- 未跑全量 AG 套件与根 crate 档；未做 release 性能复测与长时间 churn。
- 桌面 Windows x64；未验证 Linux/Web/移动。
- 不声称 G09 内容包、完整 M2、生产 I2/I3/AutoUI 或通用图片浏览器/下载器。
- 知识仓 Spec 仍为草案（`research/sprite-frames-spec-delta.md`），merge 阶段落地并刷新 ledger。

## 6. 已知限制

- 图片与模型共用 8 身份/32MiB/64MiB 预算；超域素材必须回 new，不得抬限。
- 每 Run 最多 8 个绑定叶 + 8 个槽行；第 9 个绑定与超长输出被命名拒绝。
- 精灵 tint 为中性白 × 对象 alpha；对象 rgba 仍是平色回退色（不污染导入美术）。
- 解码/上传仍是同步的 stage 边界工作；不抢占 native 调用。
