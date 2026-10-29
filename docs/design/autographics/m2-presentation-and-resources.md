# AG M2表现与资源阶段（规划v1，2026-10-29）

引擎/计划归auto-ui，Gallery内容/计划归ag-examples。旧canonical Specs仍按显式跨仓路径消费，本设计不迁移或改写其现状。已核事实/源码指纹及缺失素材收据风险见[基线](../../plans/attachments/004/planning/baseline.md)。本轮全部新Plan drafting、0/10，只有规划授权。

## 1. 本阶段边界与顺序

| 计划 | 独立结果 | 硬前置与启动条件 |
|---|---|---|
| [004 公共图片与2D精灵帧](../../plans/004-autographics-public-images-and-sprite-frames.md) | 安全PNG/透明采样、版本化场景/完整离线closure、真实Auto取帧/翻向与共同发布 | 002实际交付已满足，用户启动work后T-01/T-02 |
| [005 3D纹理与UV](../../plans/005-autographics-textured-models-and-uv.md) | 固定静态素材色图/UV/变换、依赖闭包/身份、纹理绘制与寿命 | 004实际交付，先恢复固定源清单再验证素材整链 |
| [006 节点动画与视觉朝向](../../plans/006-autographics-node-animation-and-visual-facing.md) | 无skin节点TRS clip、Auto phase/yaw、碰撞独立与same-tick呈现 | 005实际交付，先核原clip与整组合预算 |

三份改动同一资源/Frame/assembly/renderer合同，按004→005→006实际delivery串行；T-02均先冻结有限支持、准入资源和真实整链，失败回new，不用接口名字或一个小动画掩盖不可组合。004不声称3D动画已可用，005仍不接受含动画的character，006才闭合原character四clip。004给2D的帧由Auto自行选，Host不代玩法；006亦然。

## 2. 资源、时间与事务

原始字节身份与依赖闭包身份分别定义：图片完整SHA不变，带外图模型另以root+有序依赖边/子身份形成closure身份；保存、重开、consumer、GPU cache必须消费同一闭包。资源准入先检查所有声明/跨度/预测预算再decoder/upload，绝不让通用GLB加载器按URI任意IO。

CPU/GPU/磁盘/运行预算沿前序实际交付。新图片/UV/纹理/节点clip计入原有合计，不能每类额外获得8资产/32MiB或64MiB。支持域由各Plan明确限制，超域/损坏/超额分开拒绝；必须素材不适配则带原件/反例回new。005保留原始节点与primitive映射，使006不用推翻一套已经交付的不可逆烘焙接口。

运行视觉由真实Auto固定tick输出，Host校验身份/界限并采样；角色/相机/对象/事件/精灵或pose只发布一份Prepared。坏输出/prepare失败last-good全体保留。动态实例带完整Run/slot/generation；销毁撤销、重生初值，旧代不能改新实例。视觉朝向/pose与物理碰撞分离；颜色与shadow必须用同pose。

## 3. 内容与并行边界

[ag-examples/001 G11](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)继续固定消费002已交付版本，可与004/005/006开发并行；独立树/SDK准备/target/报告，真实GPU/共享重型门机器单实例。新引擎不反向升级G11锁，内容仓遇002合同缺口应反馈owner，不由本轮接管。

下一份Gallery本地号当前为002，**本轮不预占**。G09待004实际交付且ag-001共享目录/工具基线落地后另建本仓内容Plan；G10a待005、G10b待006，届时按实际写面拆一至两份。资源准备可提前只读/自有素材分析，改共享catalog/gallery.py/verify_packages的内容计划仍需串行；不能因为“2D/3D分开”假定可并写。G09/G10的README、目录、按项锁、真实公开启动、资源许可、单项ZIP离线A→B由各自内容合同认领，不塞引擎Plan、不省验收。

## 4. 后继与完成分母

声音/HUD（G12）、两款独立仓完整关卡、资源制作/发行、Gallery首12项最终闭环仍未立本轮编号。004..006只补M2表现前置，不代表M2完成。两款游戏仍必须真实Auto玩法，不用Rust专用业务/截图代替；不因新切分重算或删旧G2/G3验收分母。

桌面Windows x64目标保持，Web/移动研究独立旁路；生产AutoUI声明树/HIR/ACE融合不插为本阶段前置。根crate零改不跑根Cargo档，AG只跑受影响用例/新AC必要正反例与可复现旧样本；OS抢焦点自动化暂停，全量AG留里程碑。独立review同树顺序、冻结门通过后才merge，清理必须先wt-guard clean。
