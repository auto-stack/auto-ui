# AGE M2首批后引擎扩展设计

2026-11-17 target-state v1。统一入口[三路线strategy](../../../../auto-lang/docs/design/strategy/age-m2-three-route-strategy.md)，共同分层/收据见[架构](../../../../auto-lang/docs/design/autographics/age-m2-three-route-architecture.md)。本设计只规划，不改已交Spec/旧档/M1锁或运行任何探针。新增四Plan均drafting0/10。

## 1. 增量与去重

| 新合同 | 旧能力/研究实际复用 | 必须新增 | 冻结可观察最小目标 |
|---|---|---|---|
| [UI099 MAP](../../plans/099-age-m2-tilemap-chunks-and-layers.md) | 055/AG022预算与截断证据，058移动/query/sprite、022图集 | map数据/可见chunk/layer批绘、同Tick空间投影、资源/身份 | 64²×2层、16²块，4个空间resident块（两层共8 layer-chunk）/2048quad；跨块与255/256/257非截断，CPU/ROI/上传各有证据 |
| [UI100 ASSET](../../plans/100-age-m2-scene-assets-and-material-import.md) | GLB/贴图/模型模板、MeshStore/SpriteStore staged/pin | scene/subresource身份、依赖闭包/材质元数据/候选图 | 两node共享mesh、两material、不同TRS，冷/暖上传/坏依赖恢复 |
| [UI101 PBR](../../plans/101-age-m2-opaque-pbr-and-multi-light.md) | 080 LightParams/单硬阴影、034材质/贴图 | 明确GGX BRDF/颜色空间与1..4方向灯、一caster | 独立解析`.97/pi`样本/多色贡献/GPU ROI、第五灯/透明/坏值拒绝 |
| [UI102 CLIP](../../plans/102-age-m2-transform-clips-and-state-transitions.md) | 006/022/034朝向/帧；现node_animation通用TRS/LINEAR/STEP/短弧/层级采样、资源/时钟 | 复用通用TRS采样器；新增播放状态/两clip过渡与Schema/AV接入 | t=.25→x2.5、两clip半混7.5；双view不双Tick、pause/seek/loop/失败恢复 |

这四项不是新游戏/新编辑器，不复制AG029/030内容。普通Node夹具由每包I任务提供；真正增强混合样例由AG031原T-04补在现showcase中，只验证组合，不抵各包独立分母。UI080已交T-04作者链保原有限数值/Light范围，新PBR不重写其历史。

## 2. 代码层与公共源

`experimental/autographics-prototype/src/assets/{glb,textured,node_animation}.rs`、`render2d`、`renderer/sprite.rs`、`render3d/{pipeline,light,anim}.rs`是已有owner；`autographics-auto-driver/src/age_session.rs`复用age-ace-session共享体。只在这些owner增量，不重建loader/scene/renderer。新Plan的结构、数字域/预算、接口草图与合法/拒绝/恢复case是可执行合同，T-01核真实版本/ABI不重新发明范围。

一份原始Node/asset closure通过981共同前门。新domain资源类型与服务签名由UI包给事实、981 LANGUAGE/SESSION原任务完成Schema/唯一verify/自动Rust binding与AV使用，不能在旧pilot hand parser补完新生产链。Float model/向量/载荷/方法等缺口需实际provider正式交，暂不支持明确拒绝，不用缩放整数冒新Float能力；080历史缩放范围仅保原证据。

## 3. 分阶段交付和最终完成

M2A-08→本包所需正式AV组件/typed端口→中立实现/AV独立R/fold→新UI包归档清树。每包`IF-AGE-GFX-AV/<cap>`含源、SDK/Schema/cap/ABI/预算/运行/独立oracle/错误/资源。新增核心仅选择其受影响AG文件/命名case，旧M1无改不跑全量。

UI-W2-AC及各cap AV已交后，1068 T-04＋981 LANGUAGE/FUSION接同源AC实际机器业务/同host；UI-W3-A2R及cap AC已交后，981 RUST＋980 B-RUST接真实A2R/rustc/同host。它们的共同语言/宿主实现若要修改UI只在同组兄弟树按同一现有owner做，不重开已归档AV Plan，也不另建“各端同功能”计划。跨端新缺陷可以原在途后端任务修复并保原source/Spec范围。

AG031 r2新增M2-EXT-MAP/ASSET/PBR/CLIP行：四cap三端独立行为/资源/错误/恢复＋增强同源真实混合＋包/OS原门齐，才AGE M2-ready。新包AV完成不是全三端；原UI波签章不等所有新cap，避免cap/后端循环。旧058/AG023/029/030可先AV交子面，不等A2R；它们原全量AC/V/包/资源在相应波保持，尚未全闭不勾整项。

## 4. 大规模、性能与兼容

每Plan T-02最低负载先证；旧容量研究原计量/源保留，不能把旧VM1000步当ACE fuel或把32credits当最终语言规则。CPU逻辑/构建/上传/GPU/加载内存分列；计量使用真实adapter/toolchain/模式，未测不捏SLA。新预算是versioned cap的准入域，旧profiles逐字节/身份对照保持；超过本包域必须清晰拒绝或needs_replan，不砍对象/分时偷偷成功。正式压力/里程碑全量独立时段，普通计划不占40分钟全AG门。

## 5. 下一滚动候选

骨骼/蒙皮与动画编辑、完整刚体/导航、粒子/后处理、多shadow/其他灯型、完整glTF扩展/跨平台等，先以真实样本/平台和容量报告决定每个中等合同。当前只登记依赖于四包真实证据的触发条件，不预占号/不把这些候选算已计划产品。原已承诺项目仍按旧AC，不因本段排除而删除。
