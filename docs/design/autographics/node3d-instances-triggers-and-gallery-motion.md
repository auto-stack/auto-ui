# 三维实例/触发/新局Node接线与角色相机内容迁移

2026-11-02，设计v1；仅规划、暂不实施。承接[027/011设计](node3d-runtime-and-gallery-migration.md)、[作者模型](declarative-authoring-model.md)、[逐入口路线](gallery-node-authoring-migration.md)。设计/引擎计划归auto-ui，内容计划归ag-examples。

## 当前证据与两份合同

auto-ui主线867819bf，027实际交付f3dea0d并归档/清理，canonical node3d-character-and-runtime-camera.md SHA256 95a86660dfa15a09206baac45d8901a9f515fbe871cbdc676a210afed14ff621；025实际交付e9cc929、canonical/锁收口54899e9，均已归档/清理。不是仍在规划的能力。ag-examples主线2fcbaa72，010实际交付8bfddca，原18oracle与完整包/归档清理收据齐；其部分回执写9/22，当前example.json与src/generated清单实际为11/22（原21已迁10＋新增HUD1），剩11个。原回执作为历史保留，不据错计数减少验收。

AG011由用户告知正在开发；本轮初核主检出合同仍drafting0/10、无ag-011树登记，随后已核见唯一D:/autostack/.wt/ag-011/ag-examples（codex/plan-011-dev）及只读auto-ui@f3dea0d消费树，随后也已见owner的T-01/T-02执行记录，header尚未同步。仅按实证登记在途，不替owner勾任务/翻状态，不接管或重复建树。

| 合同 | 范围 | 进入/并行条件 |
|---|---|---|
| [ag-examples/012](../../../../ag-examples/docs/plans/012-autographics-gallery-node3d-character-and-cameras.md) | G07两布局与各main/main-fast、G08/3d主alt，普通模块/局部控制器/具名角色相机引用，三入口净增0 | 固定027实际交付f3dea0d；不等011/028，不追引擎开发tip；完整G08包仍包含固定019的2d |
| [auto-ui/028](../../plans/028-autographics-node3d-instances-triggers-and-rounds.md) | 在020/024/027上接三维template/instance/trigger/round与局部事件，补G11原Mob模板静态缩放，给后续G11/3d内容提供前置 | 027与公共001/002已经实际交付；不等待AG011/012，不写Gallery内容 |

两份均drafting r1、0/10，本轮不实施、不建树/源码/资产/探针、不构建/跑GPU、不创建agent。012与011只在各自权威树写内容，共享registry/gallery.py/包工具按owner短窗顺序接线；028只改引擎。各自target/报告/消费者独占，真实GPU/窗口/音频重型门同机单实例，OS键鼠/焦点自动化继续暂停。

## 012：完整承接027消费能力，仍按内容验收

027已经提供components-3d-runtime：角色定义持character3d与局部model/on，root相机可选；G07保持固定方向基且无运行相机，G08才使用camera3d读取。角色原半体积(.4,.5,.4)、move26、四held+pressed+grounded合32，所有额外绑定/读取仍按真实调用计费，不给每个模块新额度。保原G07九＋二与G08/3d十三共24oracle，原动作/输入清理/支撑/墙/坑/Jump沿与同tick相机/Stop/重开/像素不降。

原G08斜向在实际源码中两次乘n=.70710678；alt不仅相机变为6/30/ortho16，还变speed5/jump7/gravity18，不能迁移时顺手修正。G07两个布局均保main-fast，不能因027只冻结g07a-fast就删layout-b速度变体。原G07面向字段虽未画模型朝向，仍属于作者可观察局部状态，迁移时保留原更新规则。

内容必须形成新的实际source-build/模块/资源许可/SDK681/锁/工具链/生成器与runner身份，严格公开启动/校验、全变体真实Auto/GPU、两个完整单项包G07和G08的A→B/SaveAs/冷再生/坏件恢复。G08/2d仍固定019，不让完整包退化成3d-only；027引擎夹具绿不代内容验收。

## 028：复用公共服务，不复制017或023业务编译器

旧001/002已经有objects3d/1、triggers3d/1、八slot/八pair、显式重绑与accepted新Run。028增加的是Node作者接线，不是新对象管理器或触发几何算法。复用020局部状态/receiver和027三维声明；typed模板/Run实例/触发绑定/事件游标与角色引用区分，拒绝错维/错kind/过期代/跨Run误用。Node的局部model寿命与运行对象代次分开，despawn对象不隐式删除固定Node/SoA或共享资源。

计数/是否已收集/巡逻/胜败/重开条件全部由作者model/msg/on/helper决定。可用不同名字/字段与非G11触发例证明通用性；禁止按Coin/Goal/Fail/固定bag/state等名称生成Rust或编译器业务。固定模板多实例允许共享不可变mesh；作者使用typed symbol/ref，slot/pair表由工具管理并在identity中公开，不手写数字ID。

三条具体前置不能漏：027的player3d/mob3d不能写scale/yaw/pitch，G11三个Mob模板却分别缩放.5/.6/.7；新显式profile要支持原Document准入域内的静态模板transform，旧027拒绝/冻结字节不变。G11编辑角色y=1、Life.Start改成.5的tick0差别也要真实保留，不能修改编辑源初值冒充初始化。再者017的“移动读一/静止读三”和事件tick不提交巡逻位置是旧例的显式预算策略，不能变成所有Node的隐藏游戏规则；新例须能通过作者明确的有限读取/分派与提交调用表达它。

事件N在移动/derive后唯一evaluate，作者回调N+1消费N快照；同tick队列不凭空前移，未读不跨tick堆积。enter/stay/exit、事件顺序、销毁无合成exit、隐藏仍触发、同slot新代不自动重绑、post-evaluate geometry lock都沿002。自动事件绑定如需读取pair/phase/source，必须列出实际收费；不得通过“绑定”免费化getter或丢未绑定事件。根/子Tick、事件分派、巡逻提交和重开边沿的顺序T-02以完整链冻结，避免根先跑造成重开延一拍或改变旧轨迹。

原G11移动tick一held+26move+next+pair+despawn=30只是旧源码草算；新typed/phase/事件分派额外收费全部计入32。允许原例本来就有的事件tick跳过巡逻位置提交、idle读三，但不得推广成引擎自动停输入/隔帧跳业务。原32空间/VM/源额、8slot/8pair、120base+8reserved=128与512潜在mesh包络分账；不能把027“128场景行”当成objects档128base再加八实例。96KiB managed/256KiB user-source/352KiB组合是002已批准域，不再抬额。

## 先证、验收与后继

028 T-02最多两接法做完整先证：三不同Mob模板、原tick0高度差、六实例/六pair、相机相对运动、主收集胜/alt危险败/相机越界、静止同tick三件/隐藏/球切触/重绑旧代/accepted新局与原额。另有非游戏触发组件和两个同定义状态/receiver隔离例，防止业务特判。至少逐名保原G11/3d25项；新增寿命/布局/预算/失败候选/最后owner和旧档兼容反例另列。

028交付后再单独立G11/3d内容迁移合同；本轮不预领其编号，不把引擎25项夹具算Gallery已迁。再按实际能力规划三维查询、纹理/节点动画与编辑器教程；025通用服务是否用于后继内容另按真实合同验证。完整M2两游戏与整链预算、发布仍pending，ACE/AOT/ECS/CPC/跨端不加本轮前置。

canonical Specs本轮只在Plan中提出SD，review/merge后沉淀，不改旧档/旧收据或派生JSON。未来材料唯一plan-artifacts/plan-NNN/<repo>/，开工登记owner/用途/在途，merge折摘要进Plan后对账并删自有scope/构建物/外部输出，仅具名未解决问题按owner/理由/关闭条件持有。禁止worktree链接/attachments，清树先fresh guard。
