# PLAN-004 规范增量草案（SD-01/02/03）

目标是 auto-lang 知识仓 `docs/specs/auto-graphics/`。本文件是**草案**：merge 阶段在知识仓落地正式 Spec，work/review 不改写既有 canonical Specs。所有条目以已实现并测试的行为为准（见 `sprite-frames-handoff.md`）。

## SD-01 add `sprites-and-image-resources.md`

**Rule（新）**：公共入口可以导入一张静态 PNG 并绑定到 2D 叶节点，显示整图或合法像素矩形帧；真实 Auto 每固定 tick 选择帧矩形与水平翻向，与角色/相机/对象/触发同 tick 提交。

支持域与拒绝（先结构准入、后分配/解码）：

| 项 | 规则 |
|---|---|
| 格式 | 静态 PNG；8bit；颜色类型 2(RGB) 或 6(RGBA)；无交错 |
| 拒绝 | APNG(`acTL`)、16bit、调色板、截断、尾随字节、错误签名、独立 chunk 链越界 |
| 尺寸 | 单边 1..=2048；`w*h*4 <= 16MiB`；原始单资源 `<= 8MiB` |
| 预算 | 与模型共享：≤8 个原始内容身份 / 32MiB 原始字节；解码像素+geometry+CPU payload ≤ 64MiB；纹理 ≤ 64MiB；不新增每类配额 |
| 采样 | nearest + clamp，无 mipmap；sRGB RGBA8 纹理写入 sRGB 目标，线性空间混合 |
| 帧 | 整数像素矩形 `x,y,w,h`，`w,h>=1`，界内；越界/负值/0 宽高/错图/错 Run/旧代次命名拒绝 |
| 翻向 | 只镜像 U；不改变显示尺寸、AABB、位置或实体规则 |
| 块 | 每 Run 定长：4 头 + 绑定叶行×7 + 8 槽行×7；Host 独立复核身份/界/Run/代次 |

公开 Auto（profile `autographics.sprites2d/1`，profile 20 / number 109）：

| 函数 | 价格 | 约束 |
|---|---|---|
| `ag_sprite_frame(id,x,y,w,h,flip) int` | 1 credit | 共享 32 credit/callback；每次写 1 次；同 callback ≤32 次 sprite 写 |
| `ag_sprite_slot(token,x,y,w,h,flip) int` | 1 credit | 槽必须存活；模板必须已绑定；新代先用模板默认帧 |
| `ag_sprite_bound(id) int` | 合法缺席 | 未绑定回答 0，不置 sticky error |

新增错误码 41（未绑定/非 2D/配置不完整）、42（帧域）、43（写预算）、44（组或非 2D）。

**寿命**：暂停不推进；Stop 撤销 runtime owner 并回到编辑默认帧；新 Run 回到初始化帧；纹理按内容 SHA 缓存，暖帧换帧/翻向不重传；最后一个 owner 释放后纹理实际消失。

## SD-02 modify `scene-assets.md`

- 项目闭包从「仅 glb」变为「glb 与 png 共存」：`assets/<sha>.glb` 与 `assets/<sha>.png` 同一目录、同一 8 文件/32MiB 预算（含残留与临时预留）。
- 资源身份 = 完整原始 SHA + 长度 + format；png 的 `width/height` 属于持久身份；资源名不充当身份。
- 旧静态 GLB 已交付规则不变；旧 schema 不接受新字段；模型引用必须是 glb，图片永不被当作空网格模型。
- scene/5 新增 `objects[].sprite = null | {asset, frame:[x,y,w,h], flip}`；1..4 迁移为内存 v5 且 sprite 为 null；旧文件带 sprite 或非 glb 资源 → 命名拒绝。绑定仅限 2D 叶；最多 8 个绑定。
- 磁盘：坏图/缺图/坏 SHA 不确认保存成功；外来或残缺文件报告且不覆盖；整目录搬迁后从新位置重开运行。

## SD-03 modify `behavior-api.md`

- 新增显式组合 profile `autographics.sprites2d/1`（profile 20、number 109）= 现有 triggers2d 组合 + sprite 单元；旧 13 个 profile 的装配字节逐字节不变（测试内置基线哈希）。
- 组合预算沿用：managed 源 96KiB、用户源 256KiB、合并 352KiB；未上调。sprite 块使每 tick 增加 `4+targets*7+56` 次 Host 读取（本夹具 74 次，约全部 390 次的 19%）。
- 价格与上限：sprite 写 1 credit；`ag_char_move` 等既有价格不变；每 callback 32 credit 与 32 次写预算不变。
- 输出块在 rounds 块之后；Host 以冻结目标表、已接受实例表与 registry 身份独立验证，任一失败保持整组 last-good。
- 公共入口：`autographics-authoring --open <scene> --behavior <main.at> --behavior-api sprites2d`。`--behavior-api` 的接受值列表由 `Api::cli_name()` 派生（`Api::cli_usage()` / `Api::parse_cli`），新增 profile 若无可选名字，其一致性测试会直接失败。

## 待 review/merge 处理

1. 本草案需在 auto-lang 知识仓落地为正式 Spec 并更新 ledger；本次未改知识仓。
2. 全量 AG 套件、native/手动 CLI 验收、release 性能复测属 merge 门；work 只提供受影响面证据。
3. G09 内容包、声音/HUD、完整游戏仍由后续计划认领，本草案不声称 M2 完成。
