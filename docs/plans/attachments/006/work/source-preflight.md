# PLAN-006 T-01：实际交付与固定素材核验

2026-10-29；唯一实施/顺序复审树 `D:/autostack/.wt/ui-006/auto-ui`，分支 `codex/plan-006-dev`，base `80d9bbe3095892fcf5507dd1b9e26af523111fd4`。主检出仅既有其他计划文档WIP，未纳入本任务。用户 `/goal 完成AG计划006` 授权执行、独立复审、有界修复、合入、规范沉淀和guard清理；不授权扩大验收或预算。

## 前置身份

- 005 delivery `351919c76b4ef1d940fc7b8120254e321c41cba2` 已是当前master祖先（`git merge-base --is-ancestor` exit0），归档r3的独立R3、冻结producer门和cleaned收据齐备，已无ui-005实施树。
- 004 delivery `e0fce13810f9e1af28182950d62cfce8722aa30a`，内容身份修复 `0261ae9dacf786beb6dceb630be6af267f242625`，均由005交付闭包承接。
- 在未修改的006实施基线上 `python experimental/autographics-auto-driver/tools/verify_plan005.py --controls` exit0：1456 hashes、完整源/素材集合、错pin/删除/修改/未列文件/逃逸等负控全部通过，恢复后再绿。此证据属于005交付基线，不代替未来006身份。
- `prepare_sdk.py --from-snapshot` exit0：681 files /17 local packages；pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467` 不变。driver/AGP两锁Git blob `47176cc00bbfd7ea53bab4cf09fc1310bf687ff0` / `bc7287b5c63e70d3fcacc8b6d08b82e9d2d33eb6` 与005收据相同。未改SDK、依赖锁或其它仓代码。

## 原件实读

源 `kenneyNL/Starter-Kit-3D-Platformer` 固定 `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`，沿用005保留的7文件逐长度/SHA清单（清单SHA256 `74a88dbc289494f77b9be38180602e26bb7053f637d3ecbf26d2d0f43281b58c`），未取tip或转换动画。README载素材CC0，源码LICENSE为MIT，两份原文在sample内保留。

本地probe `D:/autostack/auto-ui/docs/plans/attachments/006/work/audit_fixed_source.py`，命令 `python <probe> D:/autostack/.wt/ui-006/auto-ui` exit0；报告 `D:/autostack/plan-artifacts/auto-ui/plan-006/source-preflight.json` SHA256 `0ca84f059687a7e472d5750e4fc7553858edcd5c19953d18067b19e1973f24f4`。脚本与JSON仅本地保留；本摘要可提交。

character 87624B，SHA256 `7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1`；8节点、6 mesh节点、无skin。原始层级 character→root→双腿/torso，torso→双臂/antenna。

| ordinal/name | channels | channel key samples | duration seconds (f32) |
|---|---:|---:|---:|
| 0/static | 21 | 42 | 0.10000000149011612 |
| 1/idle | 6 | 84 | 0.6666666865348816 |
| 2/walk | 9 | 117 | 0.4333333373069763 |
| 3/jump | 8 | 94 | 0.5 |

44 channels、337 channel key samples，逐channel时间与TRS输出累计6456B（输入accessor复用时本计数保守重复计）。全部LINEAR，时间有限/非负/严格递增，输出count/type/span匹配，四元数单位误差<1e-4、scale全部正，无重复node/path。实际原件在计划的16clips/128channels/8192samples域内。此probe只证明原件形状，不冒充生产准入、内存总账、Auto预算或GPU画面通过。

## 下一任务

T-01关闭。T-02整路径合同/probe仍开放：新公开profile与一次视觉setter、phase/yaw/wire/价格、完整Auto运动/跳/相机/触发/一次读取组合、同帧采样及实际输出必须先验证；仍保持32credits、96KiB受管源/352KiB总源、64MiB CPU/GPU，不能因素材通过而跳过预算探针。T-03..10均未执行。无新的用户决策。
