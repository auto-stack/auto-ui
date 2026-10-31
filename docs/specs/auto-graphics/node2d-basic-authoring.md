# AutoGraphics Node2D 基础二维作者入口（auto-ui PLAN-019）

Status: delivered by auto-ui PLAN-019 r1（实现/验收提交见计划归档收据；本文件
只描述实际交付并验证的能力，不声称任何未实现的语言/运行时特性）。

## 0. 范围

`node2d-lower --profile scene-2d|character-2d|camera-2d` 接受普通 Auto 模块
工程（`node`/`model`/`view-2d`/`on`/`Life.Start`/`Life.Tick`/普通 `use`/具名
资源与引用），按声明生成 `autographics.scene/6` 工程与（脚本 profile 的）
真实用户 Auto 源，经既有场景层级服务、character/1 与 runtime_camera2d/1
受信 assembly、项目 play 路径与 pinned SDK 真执行、真离屏 GPU 呈现。
首波五入口（G01 图元/alpha/变体、G03 二维层级、G06 两布局走跳、G08 主/alt
跟随相机）在三条真实整链上逐项重建并通过原数据保真对比（独立复审 101 点
0 失败）。

非范围（不冒称支持）：3D/view-3d、多呈现/混合 UI、`Life.Frame`、多角色、
新物理、查询/PNG/GLB/精灵/声音/HUD、G04 编辑器反向源码同步、ACE/AOT/ECS/
HMR、完整平台游戏。016 legacy 与 017 g11-2d profile 保持字节冻结
（verify_plan016/017 再生控制），不被自动升级。

## 1. 声明面（T-02 冻结拼写）

- `shape2d name { kind: circle|quad, position: (x, y), size: (w, h),
  material: resources.<color>, visible: true|false? }` —— 纯视觉叶子；
  circle 要求 `size.x == size.y`（直径）；`visible: false` 保对象身份
  （场景行 visible=false，不是 if 销毁）；颜色/alpha 来自具名
  `material2d.color` 资源。
- `group2d name { position: (x, y)?, visible: bool? } { props/children
  混合块 }` —— 同维父组，仅平移与可见性继承；仅 scene-2d 允许；深层
  world 坐标由层级服务逐层 f32 求和，祖先可见性 AND。
- `character2d name { position: (x, y), material: resources.<color> }` ——
  恰一个（character-2d/camera-2d）；盒体钉 858 半宽（0.8×1.0）；可在根
  view 直声明，或在共享组件 view 内声明（同一定义经普通 `use` 复用，
  008 的"同一 Player 定义"要求由此承接）。
- `solid2d name { position: (x, y), mesh: resources.<mesh>,
  material: resources.<color> }` —— 可见 + 碰撞静态对象；仅脚本 profile
  根 view；声明序生成 `ag_char_solid` 绑定。
- `camera2d name { target: refs.<char>.actor, bounds: (min_x, min_y,
  max_x, max_y), view_size: (w, h) }` —— camera-2d 恰一个，仅根 view；
  生成 `ag_camera_follow_2d`（app_start 一次，零 tick 计费）。
- 组件节点：参数（float/int/Material2D/Mesh2D 四类 typed 端口）+ 纯
  view（shape2d，脚本 profile 另允许恰一 character2d）；拒绝
  model/msg/on。同 view 元素名唯一。场景 id 1 起对象/组共享命名空间、
  声明序深度优先；对象数组即 2D 绘制序（渲染端 alpha 升序/等 alpha 原数
  组序/不透明先绘为既有渲染器行为，G01 链实测保持）。
- profile 越界在执行前带定位拒绝：scene-2d 拒绝 character/solid/camera/
  trigger/model/msg/on/if；character-2d 拒绝 camera2d/trigger2d/if/
  group2d；camera-2d 恰一相机且 target 必须指向根角色；错 kind/尺寸/
  属性/资源/重名均拒绝；总节点（对象+组）≤128 与层级服务公式一致。

## 2. 降层形状与调度

- 共享 016/017 的 load/translate 基础设施：根 model 字段为 `r_<field>`
  标量；作者 `fn` 原文 pooling（纯函数，不含服务调用）；`Life.Start`
  体并入 `app_start`（生成绑定先于作者体）；`Life.Tick` 体即
  `app_update` 体。玩法逻辑（gravity/vy/speed/jump 条件）全部在作者
  model/on，Rust 不实现任何规则。
- 服务拼写（共享 translate 冻结映射）：`input.axis(Left, Right)` →
  `(ag_action_held(1) - ag_action_held(0))`（held 计费 2）；
  `input.pressed(Jump)` → `ag_action_pressed(2)`、`input.pressed(Enter)`
  → `ag_action_pressed(3)`（消耗性边沿，计费 1）；
  `refs.<char>.move_and_slide((dx, dy))` → `ag_char_move(dx, dy, 0.002)`
  （26）；`refs.<char>.grounded(q)` → `ag_char_grounded(q)`、
  `refs.<char>.support(q)` → `ag_char_support(q)`（各 1，token 语义沿
  character 服务：0=冻结初值、旧 token fault 13）。
- G06 型完整 tick 成本实测恰 30（pressed 1 + held 2 + move 26 +
  grounded 1；第 33 计费调用 fault 10）；相机配置仅在 app_start；纯场景
  无 VM、不占用 runtime 槽。
- 场景绑定：脚本 profile 的 scene `behavior.source` 绑生成 world 源
  sha256/byte_length 与 API 版本串；显式 profile 选择，无文件名特判。

## 3. 预算与边界

共享公共服务价格不变（32 credits/callback、VM 1000steps/2ms 分片、
100000steps/120resumes/250ms 累计、30s init/2s tick、8 slots）。角色
128 solid 上限与 127-solids 超限负例沿 character 服务；层级 128 节点、
同维单父/无环、有限坐标沿层级服务。本 profile 未提高任何旧额度。

## 4. 验证面（全部在本仓）

- `cargo test --locked --offline --test node2d_basic_gates`（9 例：
  profile 越界/坏 kind/尺寸/属性/资源/重名定位拒绝、再生字节等价、
  G01 基线数据逐值匹配、组件参数敏感、改名不改几何）
- `cargo test --locked --offline --test node2d_basic_flow --
  --test-threads=1`（driver，13 例：G01 五变体真 GPU 像素类与 B01-11
  原规则、G03 derive world (35,5)/(55,5)+祖先 AND+文档不变、G06 逐 tick
  f32 独立 oracle（静止高度=面+半宽+0.002 clearance）、跳沿/apex 离散
  界/空中沿弃用不缓冲、layout-b 坑道真掉落、speed4→5 恰 30·dt 差、
  G08 clamp/letterbox 115px/同 tick 共发布/alt 506.67×380、Pause/Step/
  Stop、20 轮 Run/Stop 全 fresh、成本恰 30 负控、篡改绑定拒绝）
- `python experimental/autographics-node2d-pilot/tools/verify_plan019.py
  --controls`（19 组：13 产品再生字节等价+身份绑定+夹具零漂移、变异
  敏感性、负诊断、cold A→B 四组=仅持 closure 再生字节等价/篡改捕获/
  精确还原恢复；`--build-record`/`--verify-record` 绑定实际 generator
  二进制 sha+Cargo 锁+rustc+13 产品 hash，锁漂移拒绝）
- 旧链回归：`node2d_flow` 6/6、`node2d_g11_flow` 15/15、
  verify_plan016 OK controls（016 字节冻结）、verify_plan017 PASS、
  dialect/compiler `node2d_pilot` 6+6（共享消费者）。

## 5. 依赖与 provenance

生成器只读引用固定 SHA 兄弟树 auto-lang 98270a4f5 + auto-web f29c7d1 +
auto-down c3464d0（detached，零写面）；driver pinned SDK 7b987ca8
（`.sdk-source` 681 文件，`tools/prepare_sdk.py --from-snapshot` 可复现）。
两图绝不同图。消费/来源合同见
[node-authoring-consumption](node-authoring-consumption.md)；总迁移路线见
`docs/design/autographics/gallery-node-authoring-migration.md`。
