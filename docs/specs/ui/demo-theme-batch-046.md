# 图像与文件教学示例主题迁移批次（demo theme batch 046）

> PLAN-046 交付合同（SD-01 备稿，worktree 内；merge 落地时发布）。
> B3 图像与文件三教学示例——**029-photo-gallery、031-image-viewer、
> 027-file-manager**——接入官方主题包的**逐例可验收**规则：接入形态、
> 逐例默认/映射与豁免、双端通道、038 具名阻点关闭记录、原业务守恒
> 与内容豁免、边界与未测交接。分类：feature ｜ 状态：active ｜
> 来源计划：docs/plans/046-image-and-file-demo-theme-migration.md

## 1. 接入形态（C-01）

- 三例统一：pac `dep themes { path: "../../../packages/themes" }` +
  app `use themes.themes: Auto, Solarized` + `theme_name`/`dark_mode`
  模型态 + `design { theme: .theme_name, mode: if .dark_mode … }`
  （C-14 形态；不复制调色板，不新增菜单/业务 handler）。
- 显式默认逐例保留（C-14：不暗改未声明 App 的默认）：
  - 029：App 内 dark_mode=true 首帧（迁移前既有）；`accent_color`
    为 SPEC 命名契约变量（host/CLI 播种面，ToggleDark/SetAccent 原
    handler 原样保留）；
  - 031-image-viewer：pac 显式 `theme: "dark"`（→ dark_mode=true 首帧）
    与 `accent: "indigo"`（生成期静态 accent，迁移前无运行时 accent 态，
    本批不新增）原样保留；
  - 027：pac 显式 `theme: "dark"` + App dark_mode=true 首帧；宿主
    SetTheme 回写链（PLAN-016）不变。
- family 独立默认 = 官方 Auto；模式独立默认 = 各例声明的 dark 首帧。

## 2. 逐例默认、映射与合法豁免（C-02）

| 例 | 默认 | 映射/豁免要点 |
|---|---|---|
| 029-photo-gallery | App dark-first + Auto | 全语义 token 早已映射（PLAN-016 系）；图片像素/缩略图不重染（媒体内容豁免）；`photo_root` env-first 化（pac 默认空，`AUTO_PHOTO_ROOT` 受管入口，未设=诚实空图库 root_missing） |
| 031-image-viewer | pac dark + Auto | 全语义 token（bg-background/card/border/muted/destructive 系）；媒体像素经共享 media URI 面，主题只动壳；image_surface 内容面不重染 |
| 027-file-manager | pac dark + Auto | 全语义 token（PLAN-016 T-01）；lucide 图标/扩展名 FileIcon 字面分支为内容身份；宿主 SetTheme 回写=dark_mode 链，族选择=theme_name 新态 |

## 3. 双端驱动通道（C-03；测试/宿主切换不新增菜单）

- **VM**：`autoui_fixture`（AUTOUI_TEST_FIXTURES=1）状态写
  theme_name/dark_mode + 现有 MCP 驱动；生效值经 autoui_state 断言。
  具名 input 派发按纯名+input（PLAN-659 T-05 双形态；SEP 载荷名退役）。
- **Vue**：setupState 模型驱动 + `[data-auto-theme-root]` CSS-var 逐值
  oracle（Auto/light `0 0% 100%`、Solarized/light `44 87% 94%`、
  Solarized/dark `192 100% 11%`、Auto/dark `240 10% 4%`，与 B2 批次同口径）。
- **CLI**：`--theme`/`--accent` 覆盖 pac（C-02 实测口径）；直设
  AUTO_UI_THEME env 会被 CLI 按 pac 重写——覆盖 pac 默认必须走 CLI。
- 逐例主题腿落在原门文件内（027 desktop_mcp / 031-image-viewer
  vm_matrix + vue-actions / 029 夹具探针）。

## 4. 038 具名阻点关闭记录（C-04）

| ID | 关闭方式 | 归属 |
|---|---|---|
| F-1（031iv a2r 7 编译错） | auto-web api_gen scaffold 四处修复：types-only types.at 合并进 generate_types_rs 并跳过伴随转译（E0428/E0106/E0422/E0425）、main.rs mod 去重、legacy 路径 api 型 reconciliation、crate 根 `mod image` shim（E0432）+ 伴随 `image.foo(`→`image::foo(` 掩码改写；a2r 转译器零改动 | auto-web（本批随组交付） |
| F-2（`--server vm` 挂起） | auto-man automan.rs：UI 工程改委托 VM+VM split（起 AutoVM HTTP 后端 + iced UI + MCP），纯后端契约保持原 server-only 臂 | auto-man（本批随组交付） |
| F-3（027 套件 SEP 触发器漂移） | 测试修定：纯名+input 派发；pick_free_port 改真 bind 探测、默认端口 9511（Windows 保留段漂移 9188–9487 取走旧 9427） | 027 tests |
| 029 photo_root 用户绝对根 | pac 默认空 + AUTO_PHOTO_ROOT env-first 受管入口 | 029 pac/SPEC |
| ENV（保留段端口） | 029/027/031iv 本批端口（4429/8027/8031）均不在保留段；027 测试默认端口修定见 F-3 | 各例/tests |

## 5. 原业务守恒与内容豁免（C-05）

- 029：scan/thumb/full 链、排序/密度/收藏/切图/错误恢复（后端未起→
  load_error；root_missing→空态）保持；收藏 Storage 键不变；活跃态换肤
  不重建列表。
- 031-image-viewer：open→session/URI→导航→view revision→损坏恢复
  （corrupt fixture）保持；媒体字节不进控制面；性能门条件保留（本机
  冷启未达为既有具名条件，不在本批重开）。
- 027：真实 fs 导航/排序/隐藏/选择/网格/open-with 保持；文件写入/删除
  仅隔离目录（测试用临时 workdir 副本）；独立与宿主原支持区别不变。
- 三例图片/媒体像素、文件数据、文档内容跨肤输入恒守（038 哨兵口径），
  切肤只动公共壳。

## 6. 边界与未测交接（C-06）

- 本批只迁 029/031-image-viewer/027 三例；B3 其余九例（018/019/020/
  024/025/026/030/031-paint/041）仍按 038 分包与具名前置接续，数量
  不以本批冒称。
- 真实产品（auto-photos/auto-explorer 等）与 OS055–061 产品迁移独立；
  041/042 更大能力、930 来源守门不因本批闭合。
- 共享 stylekit/remote-frame-e2e/032 按来源保持义务。
- 既有具名红保持 owner：031iv 本机冷启性能档、027 VM toast 独立窗不
  入树/像素（F-12 观察项）、031-paint/024/026/041 测试定位修定属后续包。

## 7. 验收锚点

- 证据：PLAN-046 §9 work 收据 + plan-artifacts/plan-046/auto-ui/
  （生成日志、套件日志、探针输出；merge 时折叠）。
- 逐例两族×明暗实际可见（CSS-var 四值/luma 对）+ 原业务门逐例重放 +
  换肤活跃态业务守恒 + 038 阻点同反例重验。
