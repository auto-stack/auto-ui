# PLAN-004 T-02 精灵资源合同（冻结 v1）

本文件是 T-02 的产物：在实现前冻结公开签名、profile、价格、定长输出与有限支持域，并记录 T-01/T-02 的实测。规划合同见 Plan §5；此处只记录已经真实运行过的事实，不把规划数字当证据。

测量环境：worktree `D:/autostack/.wt/ui-004/auto-ui`，分支 `codex/plan-004-dev`，base `d3bd6b495dc8b690925e88deee9fbf2c42e7f991`；Windows x64 debug 构建，单线程，`cargo test --locked --test public_sprite_frames -- --test-threads=1`。

## 1. 身份与版本

| 项 | 冻结值 |
|---|---|
| behavior 版本 | `autographics.sprites2d/1`（旧 13 个版本不变） |
| `Api::number()` | 109 |
| `Api::profile()` | 20 |
| 组合 | 现有 triggers2d 组合 + 新增 sprite 单元（character2d + runtime_camera2d + objects2d + triggers + sprite） |
| API 区名 | `<runtime_sprites_v1.at>` |
| 装配字节 | 旧 13 profile 的 `api_hash/bridge_hash/combined_hash` 与 pristine 基线逐字节相同（测试常量 BASELINE） |

实测：profile 20 组装后 managed 源 80367 字节（api 76518 + entry 831 + transport 3018），合并源 80418；冻结上限仍是 96KiB（managed）与 256KiB（用户源）/352KiB（合并源），未上调。触发器 2D 基线（managed 73404）留给本 profile 的余量足以容纳 sprite 单元，因此不需要抬限。

## 2. 公开 Auto API（签名冻结）

绑定是**文档**行为（编辑期导入 PNG 并绑定 2D 叶），不是运行时行为；Auto 只负责每 tick 选择矩形与翻向。

| 函数 | 返回 | 语义 |
|---|---|---|
| `ag_sprite_frame(id int, x int, y int, w int, h int, flip int) int` | 1/0 | 一次写入同一静态叶的帧矩形与翻向；`id` 是稳定文档 id |
| `ag_sprite_slot(token int, x int, y int, w int, h int, flip int) int` | 1/0 | 一次写入该实例槽的帧矩形与翻向；`token` 是 001 句柄 |
| `ag_sprite_bound(id int) int` | 0/1 | 合法缺席查询：未绑定回答 0，不置 sticky error |

约束：
- `w/h >= 1`，`x,y >= 0`，`x+w <= 图宽`，`y+h <= 图高`（Auto 侧按其配置的图尺寸校验，Host 独立再验）。
- `flip` 只取 0/1，仅镜像采样轴，不改变显示尺寸、AABB、位置或实体规则。
- 每次写入 1 credit，共享 32 credit/callback 预算；每 callback 至多 32 次 sprite 写入（第 33 次命名拒绝）。
- 同一槽新代次未写入前使用模板默认帧/翻向；销毁后该行回到规范零行。

## 3. 定长输出块（Host 独立验证）

位置：`4 + 22*节点数 + camera 块 + objects 块 + rounds 块` 之后。

```
header:  status, bound_count, run_id, tick_id           (4)
变长固定行 × bound_count: node 序号, asset 序号, x, y, w, h, flip   (7/行)
槽行 × 8:            generation, asset 序号, x, y, w, h, flip       (7/行)
```

`block_len(targets) = 4 + targets*7 + 56`；一次 Run 内固定，随已接受文档的绑定数量变化（本次实测 2 个绑定 → 74）。Host 逐项验证：status=1、bound_count=配置数、run/tick 一致、node 序号与冻结表一致、asset 序号与文档 manifest 一致、图身份（SHA+尺寸）仍在 registry、矩形在界内、flip ∈{0,1}、槽 generation 等于已接受实例代次、槽行必须对应存活实例且其模板确有绑定、多余/缺失行拒绝。任何一项失败 = 整组 last-good，不发布半新半旧帧。

## 4. 有限 PNG 支持域与预算

- 静态 PNG；8bit；颜色类型 2(RGB)/6(RGBA)；无交错；拒绝 APNG(`acTL`)、截断、尾随字节、未知/调色板/16bit。
- 单边 1..=2048；解码 RGBA ≤ 16MiB；原始单资源 ≤ 8MiB（既有上限不变）。
- 图片与模型合计：≤8 个原始内容身份 / 32MiB 原始字节；解码像素 + geometry + 新增 CPU payload ≤ 64MiB；纹理分配 ≤ 64MiB（RGBA8 无额外行填充）。
- 先结构准入（签名/IHDR/chunk 链/尺寸预测）后分配与解码；解码后再次核对尺寸与长度；被拒绝输入不进入 decoder（`decode_attempts` 计数不变）。
- 采样规则冻结为 nearest + clamp，无 mipmap。

## 5. 文档与磁盘

- `autographics.scene/5` 新增 `objects[].sprite = null | {asset, frame:[x,y,w,h], flip}`；`frame` 恒为显式整数矩形。
- 旧 schema（1..4）迁移为内存 v5 且 `sprite` 为 null；旧文件若携带 `sprite` 或非 glb 资源 → 命名拒绝（不得静默接受新字段）。
- 绑定只允许 2D 叶（circle/quad），3D 叶、组、超过 8 个绑定均拒绝；绑定资源必须是 png 且矩形在图内。
- 资源身份 = 完整原始 SHA + 长度 + format；`width/height` 为 png 持久身份的一部分；资源名不充当身份。
- 模型引用必须是 glb；图片资产永不被当作 0 网格模型使用（`model_data()` 显式拒绝）。

## 6. 错误码（新增，命名且 sticky）

| code | 含义 |
|---|---|
| 41 | sprite 目标未绑定/不属于 2D 叶/配置不完整 |
| 42 | sprite 帧域（矩形越界/宽高<1/flip 非 0/1/默认帧越界） |
| 43 | sprite 每 callback 写入预算（33 次） |
| 44 | sprite 目标是组或非 2D |

说明：`ag_char_move` 等既有价格（char sweep 24、char_move 26 总价）不变；本 profile 不新增查询预算类别，仍共享 32 credit。

## 7. T-02 实测（本次唯一真实运行）

| 指标 | 值 |
|---|---|
| 组装 managed/api/combined | 80367 / 76518 / 80418 字节 |
| block_len（2 个绑定） | 74（上限 4+8*7+56=116） |
| 编译器编译耗时 | ≈169–185ms（debug，单次） |
| tick 中位/P95 | ≈5.56ms / 7.78ms（24 tick，含角色、相机、对象、触发、精灵块读取） |
| 精灵块读取调用数 | 74（旧 triggers2d 为 316 总数，本 profile 为 390） |
| VM owner 弱引用 | 存活（tick 结束仍在） |

判据：24 tick 内真实 Auto 角色 x 单调前进；帧序列 0→1→2 随真实 phase 切换；左向输入只切换 flip 而不改变运动；实例槽在事件读取后从默认帧切到 Auto 写入的帧 2；PM 帧/矩形全部通过 Host 独立验证。上述均在 `tests/public_sprite_frames.rs` 中断言。

价格结论：sprite 更新 1 credit 与既有 32 credit 预算兼容；整组合仍在一个 8ms pump 内可完成，但 debug 构建 P95 已接近预算上限（7.78ms），因此实现阶段不得再增加每 tick 的 Host 往返查询；GPU 准备/绘制成本在 T-04 单独测量，不并入本表。

## 8. 不在本计划内

3D 纹理/UV（005）、节点动画/朝向（006）、G09 目录与产品包、骨骼/3D billboard、声音/HUD、生产 AutoUI/HIR/ACE 融合、通用图片浏览器。夹具图为自有小型非对称图（`samples/sprite_frames/sheet.png` 96×32 三帧，左红右蓝 + 每帧标记块 + 透明顶行），不冒充 858 候选美术。

## 9. 待后续完成

- T-03：编辑期导入/绑定/保存/重开与完整离线 closure（含磁盘 8 文件/32MiB 共享预算、重解析路径拒绝、坏图拒绝/修复恢复）。
- T-04：AGP 真实纹理绘制（nearest/clamp、预乘混合、翻向只改 UV）、像素 ROI oracle。
- T-05：Pause/Step/Stop/新局、纹理 warm 复用与 owner 弱引用。
- T-06：fresh consumer、旧 profile 定向面、identity controls、fmt/warnings、SD 草案与交付说明。
