# Node3D 方向光与阴影作者链可行性

PLAN-059，2026-11-05，研究结论：**bounded-go（静态描述＋Auto 动态标量的研究适配链）**。
底层 MeshPipeline 单方向光/阴影可用，普通 Node model/Life.Tick 可以驱动
研究绑定；普通 Auto Node 尚无正式 Light 声明/API。本研究不交付该 API。
本研究不新增生产代码，不认领 three.js 覆盖，不改变 M2 交付或预算。

研究源码固定为 auto-ui `e02ca9f8452555db2d44e66363ebf5e317b5716f`。
`pipeline.rs` SHA256 `a0a31372d4f7b32e0d3eb8d00b2c62eb3fa284492a392278ed2458fcfe36234a`。
SDK 冻结输入逐文件核验 681/681；pilot 编译使用历史作者侧 auto-lang
`593c334805909c25518b7443f9a11707712a45da`，与 driver SDK 分图。
具体命令、完整研究探针、输入、数字与摘要保存在同号归档 Plan。

## 已证与缺口

| 层次 | 实测结论 |
|---|---|
| 正常声明 | `light3d` 未知元素；三维叶的 direction/energy/shadow 均具名拒绝 |
| 局部事件服务 | `refs.receiver.set_energy` 未知服务；已有服务仅位置/显隐/查询 |
| Auto 状态→光参数 | 一种完整研究候选已成立：普通 Node→既有生成器原生 get_r_*→固定 SDK681 Session 的 main/app_start/app_update→受检静态绑定→LightDef/真实 GPU |
| Rust→GPU | RTX 4060 Ti/Vulkan 真离屏 GPU 像素通过；无原生窗口/输入证明 |
| 数值准入 | 原始 LightDef 未拒绝非法值；研究 adapter 在 GPU 前拒绝零/非有限/溢出方向、非法能量、非0/1开关、非法投影/bias/缺绑定。原始观测不是受支持 fallback |

固定微样本为自制受影四边形与一个投影四边形，单光、opaque 平色。
独立 CPU Lambert 与光线/平面交点选点，sRGB 每通道容差 3/255。
方向反转、energy=0/0.5、shadow on/off、去投影 pass、移动投影物、
非均匀缩放的逆转置法线均可区分。阴影点 RGB 为 (89,99,108)，
开关关闭/去 pass 后为 (135,149,162)，非遮挡点保持后者。
未使用整图 hash 作为正确性 oracle。

## 资源与寿命边界

两网格实际上传 344B/4 次；每帧 camera 64B + light 160B + 两实例 128B
=352B/3 次重写。重复 mesh identity 不重传。MeshStore 持 mesh/cache，
MeshPipeline 持 uniform、2048² Depth32Float 阴影图与按 target 尺寸重建的深度 view；
OffscreenTarget 持颜色目标。阴影图名义 16MiB，不等于实测显存驻留。

20 轮研究 Run 每轮重新从磁盘读取描述/生成源/资源，真实编译 VM、启动、
执行 Tick、64→128 尺寸切换、零尺寸拒绝、Stop 通过；初始状态每轮重置，
Stop 后 VM strings/heap 弱引用失活，pipeline owner 弱引用失活，warm mesh
上传仍为344B。它证明逻辑 owner 生命周期，物理 GPU 显存释放字节未知。
同一 Run 入口实测坏生成源、缺失 getter 描述、非法 mesh 资源拒绝，
保留正在运行的 last-good，精确修复后下一次实际 Run 恢复并重新查询/绘制。
这些是受控研究 runner 层，**不是生产 Host 或原生输入**。
最后一轮样本：device 创建 703122µs、mesh 上传 389µs、pipeline 创建
20901µs；双实例阴影帧 CPU 提交 225µs、提交加 wait 395µs。
它们包含驱动/同步成本，不是 GPU ms 或分离 shader 编译耗时；无 GPU
timestamp、物理释放字节实测。修复后的研究 runner 另有实际 compile/Tick
CPU 耗时及 SDK callback steps/slices 记录，详见 Plan；这些不是GPU时间，
也不代表旧game32credits包络。

研究绑定仅使用 x/z 平面光向（z绝对值≥0.01、长度平方有限且非零）、
energy∈[0,1]、shadow∈{0,1}；投影radius>0、0≤near<far、bias∈[0,.01]，
输出矩阵必须有限。光向与正交投影射线一致。业务状态变化全部在 Node
局部事件中；Rust 只做读取、类型/数值检查、通用投影和 GPU 接缝。
SDK 调用先执行通用空 main 入口以初始化生成 globals，再执行 app_start，
不会用 Rust 初始化或替换作者业务。此研究接线不冒充稳定生成器/生产协议。

## 最小后继合同建议

另立正式能力合同：最多一盏方向光，明确声明/局部状态的方向、能量和
阴影开关端口，连接真实 Auto 状态与受检运行时描述；方向须有限且非零，
能量有限非负且有预算上界，投影范围/near/far/bias 有限且非退化。
同一方向生成投影矩阵，保留现有 culling/bias 与逆转置法线语义；
非法输入必须在 GPU 写入前具名拒绝且保持 last-good。
应补真正 driver Run/Stop/resize/坏资源恢复与 owner 释放证明。
正式作者能力仍须上述完整合同，不能把研究 bounded-go 改写为产品灯光支持；本研究不覆盖
多灯、点光/聚光、PBR、透明材质、混合 UI 或完整目录容量。
