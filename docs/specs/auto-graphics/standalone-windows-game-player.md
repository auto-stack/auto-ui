# standalone-windows-game-player

状态：canonical候选（PLAN-072 SD-01 交付；本文随 PLAN-072 落主生效）。
本文描述 Windows x64 独立持续运行宿主 `age-game-player` 的产品接面：双击
`player.json` 旁的 exe 即进入指定游戏 Playing，只呈现游戏视口与 HUD，原生
键鼠与设备音频持续成立，终局 fresh Enter 重开、干净退出。游戏业务仍为原
Auto（无 Rust 玩法复写）；本 player 是既有 Host/Play/组合服务面的新增消费
入口，不替代编辑器。

## 1. 身份与启动

- 产物：`experimental/autographics-auto-driver` 的 `--bin age-game-player`
  （release，x86_64-pc-windows-msvc）。启动描述 = exe 同目录 `player.json`，
  schema `age-game-player/1`（serde `deny_unknown_fields`）：
  `{schema, game_id(1..64B), scene(相对 .agscene 路径，无 `..`、无绝对),
  profile(components2d|components3d), display{title(1..120B), width,
  height(320..3840)}}`。任一字段违规、未知字段、未知 schema：启动前具名
  拒绝（stderr + 用户数据目录日志 + 非零退出；产品入口另弹具名错误框，
  `AGE_PLAYER_HEADLESS=1` 或 `--qa` 抑制）。
- 资源根：exe 所在目录（`current_exe`），与 cwd 无关；场景经
  `Storage::load_project` 受检加载（scene/7|8 + `assets/<sha>` +
  `scripts/<sha>.at`，sha/长度准入）。`--config` 可显式指定 player.json；
  `--user-data-dir` 显式指定运行数据根（QA 必传；产品默认
  `%APPDATA%/<game_id>`），可读日志 `player-log.txt` 落于此，绝不写发行
  布局。
- profile 与场景 API 交叉核验：`components2d`↔`autographics.components2d/1`、
  `components3d`↔`autographics.components3d/1`，不匹配具名拒绝，无 sample
  回退、无未知 profile 兜底。

## 2. 呈现与输入（Host 玩家最小模式）

- `Host::with_project_player(doc, assets, code, two_d, window)`：与编辑器
  共用同一 Editor/Play/准入/pump/restart/音频机制；组合差异仅四项——top
  view 逻辑尺寸恰为游戏 surface 的既证布局（2D 800×380 / 3D 800×400），
  窗口按此缩放；恰一个游戏 surface portal 充满 top；iced panel 缩为 2×2
  不 portal 的见证（构造后永不重绘，编辑器界面零呈现）。resize 用模式
  top 布局；玩家模式 3D 滚轮命中区域为 top [0,0] 起。
- 启动即 `start_selected_play()`（项目行为路径：scene 携带的行为源 +
  `editor.code` 快照）。Esc = 具名退出（request_close：stop_play + leave
  close，音频流随 Host 释放）。窗口关闭按钮同路径。
- 输入沿单一路由：character 系 API 的方向/空格/Enter 经 `char_key`
  （OS repeat 不产生边）；失焦/最小化/遮挡/resize 为 `auto-Pause:` 具名
  暂停并清 held/捕获；失焦暂停仅在 Focused(true) 恢复，resize 暂停在
  尺寸稳定 600ms 后自恢复，cursor-left 只清捕获/拖拽不暂停（游戏不因
  鼠标越过窗口边停机）。固定 dt 与预算不变：tick/帧 1:1（16ms 节拍），
  backlog>4 具名暂停，渲染/resize 不另 Tick。
- 3D 相机：左拖 orbit（0.25°/逻辑px）、滚轮 zoom（每格 ×1.1^-1，距离夹
  4..12）、俯仰夹 15°..75°，与编辑器同一 runtime camera 契约。

## 3. 音频

- 场景带 cues 时 `begin_audio_run` 打开真实 CPAL 输出流（默认设备）；
  观测（callbacks/frames/nonzero/peak/deadline_misses）随
  `audio_snapshot` 公开。验收要求观测到设备非零 PCM（离线 PCM green 不
  抵实际输出）；设备缺失/不可用为 `audio_unavailable:` 具名错误，静音
  不计音频 PASS。

## 4. 平台事实（2026-11-08 实测，不凭构建成功外推）

- 实测环境：Windows 11 专业版 26200 x64；NVIDIA GeForce RTX 4060 Ti
  （驱动 32.0.16.1714；同机存在 ToDesk 虚拟显示/音频适配器）；rustc/cargo
  1.98.0 msvc。未实测的平台/显卡/系统版本不做承诺。
- 依赖：静态链接 crate 依赖；PE 导入表全为系统 DLL（USER32/kernel32/
  ole32/combase/gdi32/imm32/dwmapi/uxtheme/opengl32/pdh/powrprof/iphlpapi/
  psapi/netapi32/bcrypt*/api-ms-win-crt-*/VCRUNTIME140 等）——**干净机器
  需 VC++ 2015-2022 运行库（VCRUNTIME140 非 CRT 静态）**；d3d12/dxgi 由
  wgpu 运行时加载，不在导入表。发布依赖清单以 AG028 实际包为准。
- 构建合同（冻结）：
  `cargo build --release --locked --offline --manifest-path
  experimental/autographics-auto-driver/Cargo.toml --bin age-game-player
  --target x86_64-pc-windows-msvc`
  （`.sdk-source` SDK681 锁定树随 crate 只读提供；Cargo.lock 锁定全部
  registry 依赖）。跨构建字节不保证一致：AG028 以正式源码 + 本合同重建，
  并按实际字节重绑校验。
- 整链证据（PLAN-072 T02/T05/T06，QA 报告留档 plan-artifacts）：2D/3D
  两真实 AG025 闭包从公开 exe 普通启动→真实 OS 键鼠→Gameplay tick→可见
  GPU 帧（surface 捕获 sha 输入前后不同）→实际设备非零 PCM（leap/defeat
  cue，peak 0.22/0.44）→死亡→fresh Enter 新 Run 原点重生→3D orbit/zoom
  →resize 暂停自恢复且 tick 0→46 连续→干净退出（exit 0，不挂进程）。
  负控：5 项坏配置具名拒绝（窗 0）、坏件 sha 拒绝+修复再启动绿、
  double-tick 破坏由 tick/frame 比 oracle 判红。

## 5. 迁移归属（ACE 接洽）

- 本 player/`player.json`/release 构建/测试入口为旧 driver/SDK681 的新增
  消费者：迁移归属 `auto-lang/PLAN-981` 的
  SOURCE/SESSION/CONTENT/AUDIT 分母，登记于本 Spec 与 AG028 release
  manifest；不新增无人认领的旧 SDK 表面。若届时旧 VM 已正式退役，按届时
  已交 producer 具名修订，不复活被禁默认入口。本 player 不签 AGE M1、
  不等 ACE M2 整阶段。
- 后继：AG028 依赖本合同以正式源码重建最终发行物（两游戏 ZIP、校验、
  README），不依赖本 Plan 的临时 exe；`REL-DELIVERY-001` 持有规则见
  ag-examples/PLAN-028。
