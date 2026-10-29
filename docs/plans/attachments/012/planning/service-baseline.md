# 公共声音/HUD与Gallery后继规划基线（2026-10-30）

本附件绑定规划只读观察，不是功能/性能/PCM/GPU验收。用户只授权规划；所有新文件/API/银行/字体导入/声音与HUD测试入口尚未实施。引擎012/013与Gallery004/005/006分别持合同，旧canonical Spec仍在auto-lang。

## 1. 当前来源与保持面

auto-ui调查HEAD28adf989d43155f3fae6468ccf1b9e39a44f0655；006 executing r2，读取时T-05已有实测记录、尚未实际delivery，唯一ui-006树/owner不接管。其候选node-animation3d version110/profile21、ag_visual_pose和1credit ag_actions_snapshot只用于规划参考；012/G10b实施必须读正式delivery。005已实际交付351919c76b4ef1d940fc7b8120254e321c41cba2；Gallery003内容4816807835b484653c2c31b6fbfff9907aef1ebf，归档/清理以该owner最终收据为准。

ag-examples调查HEAD1cb640a7584d8b1cd483eb940b1867297a688fde，11逻辑项19入口；G10仅textures，锁005与历史receipt保持。本轮不碰其原件/旧十项/已交付锁和acceptance。UI旧007/008/README/主题设计WIP保留，只提交本次新Plan/相关AG索引与取号。

观察：kira0.9/ogg已是两AG包依赖；driver src/host.rs的Audio是旧Breakout专用one-shot，squash_host.rs::Music是专用loop/score路径，不能直接宣称公共资源/cue已支持。AGP render2d/text.rs用fontdue0.9，TextRenderer需font bytes/字符集合；缺字可跳为空格，所以公共HUD必须先拒绝不支持字符而不是沿此退化当通过。

## 2. 固定源素材只读核验

源kenneyNL/Starter-Kit-3D-Platformer，commit3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0，本地保留副本仍是该Git HEAD。下表是此次实际文件长度/SHA，不是已导入/转换/打包或播放证据。

| 原件 | 字节 | SHA256 |
|---|---:|---|
| `models/character.glb` | 87624 | `7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1` |
| `models/Textures/colormap.png` | 10108 | `106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3` |
| `sounds/jump.ogg` | 5805 | `f090c16689e9ec04c4bef80bfcbf68bc4b63ab4ce29f7fbb5cee05a5641d5694` |
| `sounds/coin.ogg` | 6620 | `df55e931e6b78e5aecc015922e873a0e8bfa4e97cfb19abc64149d54afe89216` |
| `sounds/land.ogg` | 5885 | `145a1c17125dc4d344ac135bc5073e35ea12e19700ca30ee6e1e477010b4dca0` |
| `fonts/lilita_one_regular.ttf` | 26828 | `4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa` |
| `fonts/license.txt` | 4391 | `31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b` |
| `README.md` | 1688 | `d8f51d172d58e62d289105a5a1561e02e54cccc4c6d1215472169bc50ff2f91f` |
| `LICENSE.md` | 1062 | `f755bf8f21002f8d5baedaf75564141d363971f3f7bc8c4062956b0b15a6f04f` |

README图像/模型/音效CC0声明、代码LICENSE MIT与font OFL/RFN分别保留；禁止用代码MIT概括字体/声音许可。模型角色含原四clip、无skin；实际dtype/phase/原件限制以006交付为准。声音三个原Ogg合18310B；字体26828B；均不等于将来的单项ZIP或PCM/atlas大小。

G12a计划制作单PCM16 bank与jump/coin/land三cue，记录原解码样本/采样/量化界/派生hash与配方，尚无bank字节/SHA。G12b原样使用唯一审计Lilita font，不修改原字体。完整资源设计5模型/共图+bank1+font1=7原始身份，但T-02/最终游戏仍需实际union/暂存/历史/CPU+GPU证明；不能为音频/字体另开免费预算。shader资源、PCM owner、voice、glyph表/atlas和输入/命令队列都计原额。

## 3. 接受与验证的必要边界

32credit约束不能靠删字段解决。012的有限新bundle（一次/回调、1visual+4cue）以及013扩至4HUD槽，显式1credit；每次CPU/步数实际计，旧API和profile字节/价格不变。2D输入bit沿旧0/1左右、2跳、3重开，4/5零；3D沿006六action，不为统一packet重排旧语义。完整输入+运动/grounded+next/pair/phase+bundle的32和式只是目标，T-02真实有事件分支/额外1credit负控才可关闭。

坏candidate不提交新声音，visual/HUD维持last-good；已发样本不可撤回，postaccept设备错误另作具名Fault/停旧voice/不重试，不能错误要求回滚物理音频。Pause冻结、Step静音调试消费ordinal不积压、Stop从下一本服务quantum无旧Run PCM、fresh Run清身份；硬件已缓存边界单列。native本stream观察不录麦克风或系统其他应用音频，PCM波形/voice及实际设备后端与API计数分层。HUD必须是游戏surface新鲜GPU实际字形，不用Iced label/日志/旧截图。

## 4. 分仓升级与计数

Gallery004把G10整项升006，同时保留textures源/005旧锁/历史receipt并在006重验原G10a九oracle；不引入子入口锁体系。Gallery005建G12a音频项/012锁；Gallery006再显式把G12整项升013并重验audio、保012历史。旧其它项不跟升，normal launch只公共CLI。source binding包括原件/派生bank/font/closure/cue/源/根license_sources的实际字节，exact必需oracles/空/partial/错误pin均受检。

当前11/19 → G10b后11/20 → G12a后12/21 → G12b后12/22；每次只对真实已交付入口标Ready。G10/G12齐备不替代完整两关、统一最终引擎矩阵/发行与全AG里程碑。无远期空号/新agent/实现树，本批只规划五份当前直接后继。

## 5. 调查指纹（时间点观察，非完成证据）

执行需读依赖实际delivery再冻结，不能要求仍在写的006进度文件永远保持此hash。

| 仓/相对路径 | SHA256 |
|---|---|
| `auto-ui/experimental/autographics-auto-driver/src/host.rs` | `019d5eca324b9c0fd10080a2f4b07ae26b32cdc36bbd0dd88a63fde008b9d0f8` |
| `auto-ui/experimental/autographics-auto-driver/src/squash_host.rs` | `91ce8f3e5dacdbeac7756e2098cc5d60ab17eea2e9f2206365387a25498cadf1` |
| `auto-ui/experimental/autographics-prototype/src/render2d/text.rs` | `ea6b2e2789cc8fdf40f931e1116ce5940c2e08e753396b664343d0d9a23fdcd7` |
| `auto-ui/experimental/autographics-auto-driver/Cargo.toml` | `5ad9e193f1dd8592bf7c2e098110a28afcf3ef7939c7acfb01120be5b5bfc69d` |
| `auto-ui/experimental/autographics-prototype/Cargo.toml` | `0706b9bea76c1cbc71e928adebcf28d6d5ec4b267819996c1fa737b35e9da029` |
| `auto-ui/docs/plans/attachments/006/work/node-animation-contract.md` | `145382c65fe9ae7c2610912e3672740002909e3fdb64be1e94ca1eeaa3906eb4` |
| `auto-ui/docs/plans/attachments/006/work/input-budget-revision-r2.md` | `659e4c3bd353ec5c0a622496b6897eb59ccfcb4b4cc9c1f4261aa7019d2e6a66` |
| `ag-examples/tools/gallery.py` | `51594c27bb49e4a63018421ad835dcc28a4c34f1ea681b45358776700bcd2be8` |
| `ag-examples/tools/verify_packages.py` | `fa0eef8c349ca3fa6951468465480c052c34c7a3273c866a303e47b69eabde4b` |
| `ag-examples/examples/g10-textured-models/example.json` | `c6b6b4056938b0859cb85edf9d204b82d74a9dfd11a06fe25c3b62ae2f0e5b17` |
| `ag-examples/engine-locks/auto-ui-005.json` | `97ee758a314ace9e3525868e47ae8b2f606d5a5cbcafc54b3d8a9576f7c15516` |
