# PLAN-005 固定输入恢复

2026-10-29 从 `https://github.com/kenneyNL/Starter-Kit-3D-Platformer` 恢复并 detached checkout 固定提交 `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`，不消费当前分支tip。本地原件位于 `D:/autostack/plan-artifacts/auto-ui/plan-005/kenney-source`；逐文件长度/SHA/git blob/材质/图像/动画原始清单为同级 `fixed-source-manifest.json`，SHA256 `74a88dbc289494f77b9be38180602e26bb7053f637d3ecbf26d2d0f43281b58c`。

| 原件 | 长度 | SHA256 |
|---|---:|---|
| models/platform.glb | 13352 | 78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c |
| models/coin.glb | 22124 | a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5 |
| models/flag.glb | 13276 | 0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae |
| models/character.glb | 87624 | 7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1 |
| models/Textures/colormap.png | 10108 | 106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3 |
| README.md | 1688 | d8f51d172d58e62d289105a5a1561e02e54cccc4c6d1215472169bc50ff2f91f |
| LICENSE.md | 1062 | f755bf8f21002f8d5baedaf75564141d363971f3f7bc8c4062956b0b15a6f04f |

固定README明确图像/模型/音效为CC0，代码LICENSE为MIT，保留两份原文。三个静态必需模型都引用 `Textures/colormap.png`，仅声明 `KHR_texture_transform`，UV0，无动画，sampler minFilter=9987，材质doubleSided=true。用户已授权r2保留原件并支持opaque双面；角色有8节点和static/idle/walk/jump四clip，仅作为后继006输入/本期拒绝对照。

实施基线 auto-ui/master `2b88bd69b2a48172bf761615cc4e78a1df563b62`；004 delivery `e0fce13810f9e1af28182950d62cfce8722aa30a`，修复 `0261ae9dacf786beb6dceb630be6af267f242625`。当前driver/AGP Cargo.lock Git blob分别为 `47176cc00bbfd7ea53bab4cf09fc1310bf687ff0` / `bc7287b5c63e70d3fcacc8b6d08b82e9d2d33eb6`；005工作树 `prepare_sdk.py --from-snapshot` exit0：681 files/17 local packages，SDK pin不变。
