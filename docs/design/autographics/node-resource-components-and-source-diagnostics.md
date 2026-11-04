# AGE模型对象组件与Node源码诊断

> **2026-11-04 当前M2补齐路线**：M2现明确包含新语法基础UI＋2D＋3D同应用组合，新增M2-10；[能力核查/补齐设计](m2-mixed-ui-2d-3d-composition.md)拆为UI048基础UI→049混合引擎→AG021综合示例，均仅规划。AG019已交付3daf2eb并归档，原候选fc82061c/旧合同/holds不改，不据此声称扩展M2完成；043在途、044可选不作本链前置。AGE命名沿[决议](age-naming-and-content-repository.md)，内容目录仍ag-examples；020 r2默认019＋021及扩展M2必需门收口后维护窗口更名，历史号/交付身份保持。

2026-11-04 v1，target-state。用户要求为auto-ui继续规划后继。[UI043](../../plans/043-autographics-node3d-model-object-templates.md)模型对象模板与[UI044](../../plans/044-autographics-node-source-diagnostics.md)源码定位均drafting/r1/0-10，仅规划；不实施/建树/复制资源/探针/构建/GPU/agent。041/042已被其它UI组件store/重载路线占用，本轮独占重取043/044，不覆盖或复用他人号。

## 1. 已有交付与内容现场

UI040已实际交付fc82061c、canonical node-editor-projects.md SHA1daa7111…、独立上下文final/归档/清理收据可核，Editor OS层15项仍D-02 pending；不因归档改成已证。035公共3D组合5903839e、039三维查询8f577a34已交付，相关历史同会话final限制按实际收据保持。

AG016现在executing/r1/3-10，唯一树D:/autostack/.wt/ag-016/ag-examples，固定035与其语言/SDK681；正在做完整游戏的独立验收。本轮不接管其实现，不把模型接线改进转成它的新增阻塞。AG017/018/019仍未开工草案；018引擎来源040已具备，完整编辑验收/必要独立证据依原合同；019候选可按原040门推进，完整联验仍等016/017/018实交付。Gallery活源仍19/22，不提前加迁移计数。

## 2. 为什么补模型对象模板

driver runtime_objects已有3D Model支持：Dim::supports_kind接受Model，instance_object继承模板resource与变换，原模板multi-mesh denominator/owner门已存在。Node作者链lower_components3d_events仍只许builtin模板、TemplateRowIn.resource写None；034虽接静态纹理/动画声明，却不代表模型可作为可销毁运行对象模板。

AG016 T-03实际收据记录：模板builtin限制下，币GLB视觉另接有状态Coin；原静态model3d还可能进入角色碰撞盒收集。它在035域内已有接线先证，是否满足原半径/几何/玩法参数仍由内容后续验收与独立复审核，T-03 go不代终验，不据该适配批准原合同变化；043提供后继更直接的组件作者能力，让一个owner持运行句柄，模型外观真正属于活对象，而非靠另一常驻叶联动。

043只接已有平色/opaque静态GLB和完整图像闭包，模板仍一个静态叶、无参数/model/msg/on，业务仍普通有状态Node；名义Slot/Template/Pair引用不变。原多part/纹理/原URI/许可字节保留，真实spawn/move/visible/despawn、单个销毁/新代/触发与两个同定义owner隔离都要GPU+实际Auto证明。模板/实例不成为solid/character/camera目标或混入空间查询世界；故意把隐藏模板放角色路径做轨迹对照，不靠放远处避错。动画实例pose/动态材质/TRS另题。

T-02最多两候选：先复用既有Model运行协议/034元数据/035组合，只补作者和真实缺口；需要新显式组合身份时准确版本化，旧profile字节/价目保持。不能仅删builtin拒绝或画一张替身图就算接线完成。

## 3. 为什么补源码定位，哪些位置不能承诺

当前load已有module/line，SDK compile已有受检assembly user/managed region定位；不是全无诊断。问题在Node多个作者模块/局部handler经过lower重组成world.at后，原模块/Node/事件的来源链尚未完整交付。driver behavior.rs明示当前SDK runtime没有结构化span，因此不能宣称能把所有运行错误精确映成原行。

044显式可选旁件/调试消费，不改默认生成源/执行。结构化发射记录原module SHA/可证span或声明范围、Node/handler/helper及精度，SDK offset先经原受检region再进作者映射；managed/合成派发不冒充用户代码。精度分exact/range/context/generated/unmapped，缺span/实例身份或多候选就明确上下文/范围，不从错误对象或“最后handler”猜行/栈。

首批components-2d、components-3d-audio-hud及editor-projects前端错误有明确支持矩阵；纯声明040无world，只有前端原声明诊断，不伪造运行map/world hash；T-02最多两真实错误全链先证，至少跨模块helper真实compile span与Node事件上下文必须成立，不能一律context充全部目标。当前AST信息不足则降精度，不在此改语言仓AST/SDK或提前引入ACE调试协议。

map绑定真实generator/world/模块闭包/assembly身份及接受Run源快照，UTF8 byte区间与展示字符列分开；Run后磁盘源变更或删除不改变指向接受的原源，SaveAs/离线搬迁不借外部路径。坏/旧/越界/重叠/错hash映射拒用、显式映射门失败、诊断回退真实generated位置；程序本身准入/原stage/code/last-good不变，不让可选诊断影响玩法成败。新增metadata CPU/包开销有界实测，不每Tick读盘/插桩，不偷偷加语法上限或服务费。

## 4. 并行关系与M2冻结边界

| 责任线 | 新合同 | 前置/独立关系 | 明确不追加的当前内容义务 |
|---|---|---|---|
| auto-ui资源组件 | UI043 | 030/034/035已交付；T-02先证后实施；不等044或游戏交付 | AG016继续035，无中途改模型/触发/关卡或新依赖 |
| auto-ui作者诊断 | UI044 | 020/030/035/040已交付；T-02先证；043非前置 | AG017/018不跟升新调试模式，旧报告不是新映射证据 |
| ag-examples M2 | 原016/017/018/019合同 | 原固定上游和独立证据门保持；后续内容用043/044须另约 | AG019不因UI有新计划改成等043/044或自动选它们的tip |

新能力是M2范围外的显式后继，允许UI在内容闭环期间继续演进；未来是否消费/升候选另以具体证据和合同决定。019仍按原已批准候选选择/统一版本规则，不能用“后继”偷换冻结版本。

043/044功能独立，可各自唯一树并行；共享load/translate/main/生成identity/Storage按owner短窗单写，合入交叠后只补真实受影响验证。当前另有041/042 UI组件store路线，不凭标题推为AG前置，也不覆盖其设计/WIP。

## 5. 原预算、验证与收尾

保原32世界credits/32物理写/8slot/8pair/8状态Node/120base+8reserved128、mesh512、raw8身份32MiB单件8MiB/CPU-GPU64MiB和源VM/time、已用音画域。模型模板base+8×worst-template meshes计hidden，真实owner/current/history/pending/外部pin并集不漏；20轮warm上传解码0和最后pin释放实证。诊断不改变VM指令计数/业务源/32池，metadata额外内存/加载/包尺寸在原合法源域下线性有界，T-02冻结后才依赖实现。

每Plan只跑真实触面/新case/必要旧消费者和负控，非日常全AG或语言根；GPU/窗口焦点/音频重门机器单实例，OS抢焦点自动化暂停保持。新Plans的独立上下文final同权威树顺序完成，不因本轮无agent授权用自查替代；旧同会话收据/040 OS pending/D-05批门按适用性保留，不自行结清。

本轮只提SD目标，不写canonical Specs。材料开工仅plan-artifacts/plan-NNN/auto-ui，登记owner/用途；merge摘要折Plan/全输出对账后清自有scope/构建物，仅具名未解决问题持有。禁attachments/第二暂存区/worktree链接/派生JSON，清树先fresh guard，不动其它路线/仓同号材料。长期同程序适配任意屏幕/组件与执行组织解耦方向保留，生产UI/ACE/CPC/跨端不在本批实现。
