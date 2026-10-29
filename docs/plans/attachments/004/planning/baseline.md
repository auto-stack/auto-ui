# AG表现阶段规划基线（2026-10-29）

本文件是004/005/006共享的只读调查摘要，不是实现证据。规划HEAD auto-ui `7e56dc8baa0b80f35ff7da757695e5e717047950`；001代码交付 `537122b09c025191355e260bc8e9880a4358cda2`，002代码交付 `0ee5ad0747ec3bb67e1288401fcc31daf3901c6e`，其归档/清树簿记进入当前HEAD。后继基线必须实际delivery，不消费开发tip。

ag-examples主检出 `9c9143dc0971637f71ff5a2ff4634f088b84feb9`；用户声明001在实施，本轮后段已见唯一 `D:/autostack/.wt/ag-001/ag-examples` / `codex/plan-001-dev` 注册在同仓，HEAD仍9c9143d。主Plan尚drafting/0只说明簿记未刷新，不能据此推翻在途事实；本会话不接管、不修改该Plan/树。auto-ui/003为另一Blog路线，实际本地分配从004起，独占锁同时核活动/两种历史归档路径、创建无覆盖并递增.next-id。

## 可直接观察的代码事实

- Document只将Circle/Quad判为2D，AssetRef仅glb；assets admission逐声明拒images/textures/animations/skins/外部URI。Renderer Quad只有平色与圆形，通用RGBA8缓存槽并非公共图片服务。
- AGP GLB ModelData仅位置/法线/索引/平色材质，loader烘焙node变换且拒baseColorTexture；anim.rs是Squash专用BobAnimation，不能声称公共GLB节点clip已交付。
- Host prepare_frame→publish已将camera/objects纳入一份Prepared；新增图片、纹理、pose也须进入受检候选，无第二发布旁路。
- 002归档r5/Spec明确桥接64→96KiB、组合320→352KiB，用户源256KiB和全部运行预算未变。后继不获自动再次抬限授权。两独立Cargo.lock、源快照/SDK闭包/32MiB主栈与16MiB工作栈需按实际交付保持；本轮未准备SDK或构建。

## 素材与历史来源风险

权威研究在auto-lang `docs/specs/auto-graphics/m2-game-sample-research.md` 与设计 `docs/design/autographics/m2-game-sample-matrix.md`：Kenney源固定 `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`；四GLB/PNG有外链色图与KHR_texture_transform，character含static/idle/walk/jump且无skin。研究称13文件203825原始字节、最大character87624；本轮未重取原件/重算，不把这些历史数字当新包/准入/动画支持证据。

研究引用的 `docs/plans/attachments/858/selected-source-candidate.json` 在当前auto-lang/master与保留autographics-dev的该路径均未找到；当前master同号858附件含ACE latency材料，**不得混作AG素材收据**。005 T-01须从固定Git源对象或可追溯保留副本重建长度/SHA/边/许可并写本Plan本地证据摘要；不能假定原附件存在、换当前源或减少选定素材。固定源若不可达且无可证副本，形成来源阻塞回new。M1历史tag映射差异/旧SDK历史对象不可达也不在本轮改写。

## 调查时文件SHA-256

下面只绑定规划观察；004实施核差异，005/006消费实际前序交付后重新绑定，不要求新实现保持这些旧指纹。

| auto-ui相对文件 | SHA-256 |
|---|---|
| `experimental/autographics-auto-driver/src/authoring/assets.rs` | `bf9b4b4f0a956bad79732152b5f5c25b44ceadb452c0e5a3c0afce4882b3d12b` |
| `experimental/autographics-auto-driver/src/authoring/document.rs` | `2068a187c06a0f6ad0f60f7c0f5fbc328a64f76f09a167b4ebf11d2e795da06f` |
| `experimental/autographics-auto-driver/src/authoring/storage.rs` | `baf303bc2900ad340c0bcd5810048ebb4c76fc9b4cd2ba0fc5a574abef712200` |
| `experimental/autographics-auto-driver/src/authoring/host/viewport.rs` | `096c253c3d671db2e86e25cf3ff51d35285d43e556a72aac2baf3546cb7a6d81` |
| `experimental/autographics-auto-driver/src/authoring/behavior/assembly.rs` | `669db066d10e17515fb40758f351a30877a632fabc6b8ec4108a1b151246ab19` |
| `experimental/autographics-auto-driver/src/authoring/play.rs` | `b997b3aa78809bfb4b3162876d5d133799b34f58e3298f3f7123063f9d75b416` |
| `experimental/autographics-prototype/src/assets/glb.rs` | `d6d458924023c2bfefef988aed3a1f2f655e1da4d8cca2d7e7dde79320be4117` |
| `experimental/autographics-prototype/src/renderer/pipeline.rs` | `04b0cdc84215a2dcb0fd303f76c4b1d2896d2097d5fb8d0b596cb1e8b7e575d8` |
| `experimental/autographics-prototype/src/renderer/resources.rs` | `28eaa8024616ee04e0e0d2e6ef1efdf96d80573d7b61e3ef7e58269805f07ab3` |
| `experimental/autographics-prototype/src/render3d/pipeline.rs` | `f5d9fee581af1636df8fd561d4755a6be65d78710721b054d1c6b605470d50c1` |
| `experimental/autographics-prototype/src/render3d/anim.rs` | `35ef5d6e638ff8550e63dc93cd07314906c3a3a5b6c0743ab209adfe1425fcd1` |
| `experimental/autographics-prototype/Cargo.toml` | `0706b9bea76c1cbc71e928adebcf28d6d5ec4b267819996c1fa737b35e9da029` |
| `experimental/autographics-prototype/Cargo.lock` | `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf` |
| `experimental/autographics-auto-driver/Cargo.lock` | `2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db` |
| `experimental/autographics-prototype/.cargo/config.toml` | `3552ddebb1ace8d9feb8871eb1ce53ace1f299a6823f5d9072e1d546b9df5c54` |
| `experimental/autographics-auto-driver/.cargo/config.toml` | `3552ddebb1ace8d9feb8871eb1ce53ace1f299a6823f5d9072e1d546b9df5c54` |
