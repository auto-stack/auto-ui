# AG M2表现与资源阶段（2026-10-29；原规划v1，当前接棒见§5）

引擎/计划归auto-ui，Gallery内容/计划归ag-examples。旧canonical Specs仍按显式跨仓路径消费，本设计不迁移或改写其现状。已核事实/源码指纹及缺失素材收据风险见[基线](../../plans/attachments/004/planning/baseline.md)。004..006初始合同均在规划v1建立；当前已交付/在途与新规划见§5，历史进入门不覆盖最新收据。

## 1. 本阶段边界与顺序

| 计划 | 独立结果 | 硬前置与启动条件 |
|---|---|---|
| [004 公共图片与2D精灵帧](../../plans/archived/004-autographics-public-images-and-sprite-frames.md) | 安全PNG/透明采样、版本化场景/完整离线closure、真实Auto取帧/翻向与共同发布 | 002实际交付已满足，用户启动work后T-01/T-02 |
| [005 3D纹理与UV](../../plans/005-autographics-textured-models-and-uv.md) | 固定静态素材色图/UV/变换、依赖闭包/身份、纹理绘制与寿命 | 004实际交付，先恢复固定源清单再验证素材整链 |
| [006 节点动画与视觉朝向](../../plans/006-autographics-node-animation-and-visual-facing.md) | 无skin节点TRS clip、Auto phase/yaw、碰撞独立与same-tick呈现 | 005实际交付，先核原clip与整组合预算 |

三份改动同一资源/Frame/assembly/renderer合同，按004→005→006实际delivery串行；T-02均先冻结有限支持、准入资源和真实整链，失败回new，不用接口名字或一个小动画掩盖不可组合。004不声称3D动画已可用，005仍不接受含动画的character，006才闭合原character四clip。004给2D的帧由Auto自行选，Host不代玩法；006亦然。

## 2. 资源、时间与事务

原始字节身份与依赖闭包身份分别定义：图片完整SHA不变，带外图模型另以root+有序依赖边/子身份形成closure身份；保存、重开、consumer、GPU cache必须消费同一闭包。资源准入先检查所有声明/跨度/预测预算再decoder/upload，绝不让通用GLB加载器按URI任意IO。

CPU/GPU/磁盘/运行预算沿前序实际交付。新图片/UV/纹理/节点clip计入原有合计，不能每类额外获得8资产/32MiB或64MiB。支持域由各Plan明确限制，超域/损坏/超额分开拒绝；必须素材不适配则带原件/反例回new。005保留原始节点与primitive映射，使006不用推翻一套已经交付的不可逆烘焙接口。

运行视觉由真实Auto固定tick输出，Host校验身份/界限并采样；角色/相机/对象/事件/精灵或pose只发布一份Prepared。坏输出/prepare失败last-good全体保留。动态实例带完整Run/slot/generation；销毁撤销、重生初值，旧代不能改新实例。视觉朝向/pose与物理碰撞分离；颜色与shadow必须用同pose。

## 3. 内容与并行边界

[ag-examples/001 G11](../../../../ag-examples/docs/plans/archived/001-autographics-gallery-runtime-objects.md)继续固定消费002已交付版本，可与004/005/006开发并行；独立树/SDK准备/target/报告，真实GPU/共享重型门机器单实例。新引擎不反向升级G11锁，内容仓遇002合同缺口应反馈owner，不由本轮接管。

下一份Gallery本地号当前为002，**本轮不预占**。G09待004实际交付且ag-001共享目录/工具基线落地后另建本仓内容Plan；G10a待005、G10b待006，届时按实际写面拆一至两份。资源准备可提前只读/自有素材分析，改共享catalog/gallery.py/verify_packages的内容计划仍需串行；不能因为“2D/3D分开”假定可并写。G09/G10的README、目录、按项锁、真实公开启动、资源许可、单项ZIP离线A→B由各自内容合同认领，不塞引擎Plan、不省验收。

## 4. 后继与完成分母

声音/HUD（G12）、两款独立仓完整关卡、资源制作/发行、Gallery首12项最终闭环仍未立本轮编号。004..006只补M2表现前置，不代表M2完成。两款游戏仍必须真实Auto玩法，不用Rust专用业务/截图代替；不因新切分重算或删旧G2/G3验收分母。

桌面Windows x64目标保持，Web/移动研究独立旁路；生产AutoUI声明树/HIR/ACE融合不插为本阶段前置。根crate零改不跑根Cargo档，AG只跑受影响用例/新AC必要正反例与可复现旧样本；OS抢焦点自动化暂停，全量AG留里程碑。独立review同树顺序、冻结门通过后才merge，清理必须先wt-guard clean。

## 5. 2026-10-29 新并行规划与当前进入门

本节覆盖§1/3的历史候选/在途描述。004功能e0fce138与008身份修复0261ae9均已交付；G11/ag-examples001及G09/ag-examples002的代码/Spec均已落地，后者清理/批门记录由原owner收尾，不能从archived单独推断。005目前executing r3、T-03闭合；006仍需005实际交付后实施。

| owner合同 | 范围/前置 | 与主线的并行方式 |
|---|---|---|
| [auto-ui/011 2D完整玩法整链预算](../../plans/011-autographics-g2-full-loop-budget-feasibility.md) | 新drafting r1、0/10；固定004＋008公共profile，跑跳/敌人/三币/胜负/新Run/相机/精灵同程序与最坏callback；G2-09缺公共服务单列 | 不等005/006，不改其engine/API/身份/根Cargo，只写独立研究harness；负研究结论可交付，不静默抬限 |
| [ag-examples/003 G10a](../../../../ag-examples/docs/plans/003-autographics-gallery-textured-models.md) | 新drafting r1、0/10；原件纹理/UV/闭包/许可/阶段目录/单项离线包，硬等005实际delivery | 现在仅规划；005交付后可与006并行实施内容，不消费005开发tip |

011与003无相互完成依赖；T-02分别冻结自己的实验/素材与验收合同，不互相借准备目录/target。GPU和重型门仍机器单实例，OS抢焦点暂停，新增Plan只做受影响检查。声音/HUD未因记录事件意图而通过，完整游戏仍以后各自独立仓；G10a只新增g10/textures，当前inventory预期11逻辑项19入口，G10b动画/朝向仍pending，不能宣布G10/首12项已齐。

本次仅规划两份，不实施/建树/agent/发布。预算Plan编号按本仓已归档010的最大本地号取011（当时.next-id落后为009，已在独占取号下推进012，不回填/重排009）；Gallery取本仓003，后续号各仓实时重核。
