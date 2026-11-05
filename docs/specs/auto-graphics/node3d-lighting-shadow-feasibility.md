# Node3D 方向光与阴影作者链可行性

PLAN-059，2026-11-05，研究结论：**no-go（固定版本的普通作者全链）**。
底层 MeshPipeline 单方向光/阴影可用；这不表示普通 Auto Node 已有灯光 API。
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
| Auto 状态→光参数 | 尚无受检生产端口，未执行这条 GPU 链；不以 Rust 改光冒称成立 |
| Rust→GPU | RTX 4060 Ti/Vulkan 真离屏 GPU 像素通过；无原生窗口/输入证明 |
| 数值准入 | 原始 LightDef 未拒绝零方向、NaN 方向/能量、Inf 能量、退化投影；观察值不是受支持 fallback |

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

20 轮底层 pipeline 创建/绘制/64→128 尺寸切换/drop、零尺寸拒绝通过；
20 轮坏源→修复的新编译进程恢复通过。这些是受控 fixture 层，
**不是作者灯光的 driver Run/Stop**；该支路因端口缺失不可达。
最后一轮样本：device 创建 703122µs、mesh 上传 389µs、pipeline 创建
20901µs；双实例阴影帧 CPU 提交 225µs、提交加 wait 395µs。
它们包含驱动/同步成本，不是 GPU ms 或分离 shader 编译耗时；无 GPU
timestamp、作者 CPU Tick、credits/steps、物理释放字节实测。

## 最小后继合同建议

另立正式能力合同：最多一盏方向光，明确声明/局部状态的方向、能量和
阴影开关端口，连接真实 Auto 状态与受检运行时描述；方向须有限且非零，
能量有限非负且有预算上界，投影范围/near/far/bias 有限且非退化。
同一方向生成投影矩阵，保留现有 culling/bias 与逆转置法线语义；
非法输入必须在 GPU 写入前具名拒绝且保持 last-good。
应补真正 driver Run/Stop/resize/坏资源恢复与 owner 释放证明。
在这些条件实证前，不可将本 no-go 改写为作者灯光支持；本研究不覆盖
多灯、点光/聚光、PBR、透明材质、混合 UI 或完整目录容量。
