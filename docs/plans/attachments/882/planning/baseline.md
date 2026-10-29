# PLAN-882/883 规划输入基线（2026-10-29）

本页只记录只读观察；未运行Cargo/SDK准备/OS/GPU探针，不是实现或验收通过收据。

## 已交付来源与仓界

- AG唯一开发权威：auto-ui master `4adf2d037a1804741adc9c5a012f08cfa566439d`，规划读取时工作区clean。
- 879语言侧迁出指引f07b5ec8，旧875源1cc68aba保历史；新代码不再合autographics-dev。
- 中央Plan/canonical Specs/ledger仍auto-lang；ag-examples内容另仓，SDK修改仍需语言路线独立合同。
- 当前source-repository-and-sdk Spec记录：SDK681快照/两AG锁/1387当前身份，冷准备--from-snapshot、17包29边；本轮不重跑这些构建门。

## auto-ui关键Git对象

| path | blob |
|---|---|
| `experimental/autographics-auto-driver/src/authoring/behavior.rs` | `f69ad93727d2f288f74afa42af8f02e12fc9c5bf` |
| `experimental/autographics-auto-driver/src/authoring/behavior/assembly.rs` | `91e67134bfe12dfc0b12ef6f801cb633d0fd7abe` |
| `experimental/autographics-auto-driver/src/authoring/play.rs` | `af194714f489af17d1a0bbd88fa87b06fc66d31e` |
| `experimental/autographics-auto-driver/src/authoring/spatial.rs` | `969e0e5a830631d7dab8989a163f94e1c58ac912` |
| `experimental/autographics-auto-driver/src/authoring/document.rs` | `ffae4ef3a9cc2465e0a9d69e49aa6caa813c8276` |
| `experimental/autographics-auto-driver/src/authoring/host/play_adapter.rs` | `a4fd3c202e0cfc10527f76454e8a574cf810fae3` |
| `experimental/autographics-auto-driver/src/authoring/host/viewport.rs` | `3eb438396144cbe6ca4947d39c96735a57c51123` |
| `experimental/autographics-auto-driver/src/authoring/game_input.rs` | `292bc1bfc5e81e6656fa7329b67174d88cffa6b5` |
| `experimental/autographics-auto-driver/src/authoring/runtime_camera.rs` | `a017095b99e229b1cc22750c8beff3cc2462e76d` |
| `experimental/autographics-auto-driver/api/behavior_v1.at` | `c7a7b6dc71dc74e8d4eec43dec1e93cc1b9e3ca0` |
| `experimental/autographics-auto-driver/api/behavior_spatial_v2.at` | `4498d2d7bc378fb46d1db72e21416f3ebf9b58ea` |
| `experimental/autographics-auto-driver/api/character3d_v1.at` | `bc3d5a70038f1fb525cdd473e92dd9f621c9c1bb` |
| `experimental/autographics-auto-driver/api/runtime_camera3d_v1.at` | `c8df4c6fe8c9603eff000113d6a1baab51d0ae66` |
| `experimental/autographics-auto-driver/Cargo.lock` | `47176cc00bbfd7ea53bab4cf09fc1310bf687ff0` |
| `experimental/autographics-prototype/Cargo.lock` | `bc7287b5c63e70d3fcacc8b6d08b82e9d2d33eb6` |
| `experimental/autographics-auto-driver/tools/verify_plan879.py` | `98c2e6f717b8b8648e7e404fb60bc916917d953d` |
| `experimental/autographics-auto-driver/tools/prepare_sdk.py` | `512e49c2441a247ecd8dddffc7b6bb326741a612` |
| `experimental/autographics-auto-driver/research/plan879-identity.json` | `f14e2b77b1c76af27f2366cf65ffa380e54a6bb4` |
| `Cargo.toml` | `6d528f03cbffb2170b66f9c4ca238599e627386c` |
| `Cargo.lock` | `a7dd2c11300ffbdc439b94b283672de4567d2868` |

## 实际代码观察与新合同边界

- behavior_v1.at：ag_valid只检查ordinal，固定拓扑/资源，无实例generation。
- Play::admit：固定4+22*N+camera长度、Int/Float严格区分，profile/run/tick/count与identity/kind/dim/parent/resource受检。
- Frame含静态Document/objects/nodes与camera；新动态表不能只放宽原count检查。
- Descriptor绑定固定base hash与≤128行；Host::prepare_frame先预算潜在mesh≤512，再按Frame.objects渲染。
- v2 ag_aabb_overlaps为闭区间，没有引擎事件流；球/圆触发和新局保旧事务是拟新增，尚未实测。
- 882初版cap8非solid根叶，base+reserved≤128；883cap8显式对/每tick≤8事件。原32credits/步数/源空间不升，整路径先行probe归work T-02。

## 880在途边界（不接管）

- ag-examples master=`6a223d9b3637ee1ce9a00335c8ee4f46d3832b26`。
- 用户明确告知880已开始开发，唯一工作树lang-880/ag-examples存在；读取时分支codex/plan-880-dev仍锚点6a223d9、工作区clean、Plan簿记drafting0/10。此观察不否定用户授权/后续异步进展，不能推断无人实施。
- 本轮不改880任何文件/勾选/状态，也不写其工作树。其固定875输入保持。
- 882可并行；883引擎等882交付，G11内容T-05另等880实际交付，按其最终schema接线。

## 分配与规划

- 中央取号器已为其他路线881取号（shared-record-method-lowering），不可复用。
- .git/plan-allocation.lock是既有空锁文件；首次CreateNew被拒，本轮未删除/覆盖该锁。随后以FileStream OpenOrCreate + FileShare.None取得排他句柄，在持锁期间核max/next并串行调用new-plan.sh两次，分配882/883，分配后.next-id=884。未修改取号脚本。
- 本批只规划两份：882 draft-ready（T-02先决机制门），883 dependency-blocked；所有work/review/merge任务均未执行。
- 无新增agent/消息调度/源码工作树/构建/发布。

- 规划收尾再次只读观察880树：已有catalog/index.json修改以及checks/、g07-character3d/、g08-runtime-cameras/未跟踪内容，证实现场持续开发；初次clean只是带时点快照。本轮零内容仓写入，不将这些WIP计入本Plan或提交。auto-ui主检出仍clean。
