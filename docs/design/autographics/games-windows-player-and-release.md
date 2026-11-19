# AGE两游戏Windows独立运行与二进制发行设计

2026-11-08，v1；用户已明确本轮继续先规划，确认交付合同后再实施。没有构建、启动、安装或发布授权。

最新状态（2026-11-08）：UI072已交付5fd25be8、archived10/10且清理完成；AG028 executing2/11，正式release打包/最终ZIP试玩仍在途。下文保原设计合同，不能把早期“仅规划”快照当当前状态；本次只更新状态，不构建或运行。

## 1. 目标与现状

原2D/3D固定关卡玩法、普通Auto作者源、资源与程序化Host/GPU/PCM oracle已由AG014/016交付，并在AG025候选5a53b789/lang30b396d72/SDK681同版验证。现有公开`--inspect`在创建事件循环前加载并渲染一帧后返回；原ZIP是B侧源码重建闭包，需要Rust工具链/registry缓存。它们不是用户解压双击即可持续游玩的预编译游戏。

首版交付目标：Windows x64便携发行候选，两个独立目录/ZIP；无需Rust、Cargo、Python、源码仓、联网下载或编辑器操作。必须真实持续渲染、原生键鼠输入、设备音频、胜败/新局与正常退出成立。受支持Windows具体build/GPU/驱动与运行库以实测清单为准，不凭构建成功承诺任意Windows/任意显卡。

## 2. 职责拆分

- [auto-ui/072](../../plans/archived/072-age-games-standalone-windows-player.md)：公共standalone player、winit持续窗口、Play/GameInput/相机/音频/渲染与生命周期；提供正式release二进制/配置合同及来源。游戏业务仍为原Auto，无Rust重写规则。
- [ag-examples/028](../../../../ag-examples/docs/plans/archived/028-age-games-windows-binary-and-resource-release.md)：两游戏资源闭包、相对路径、发行打包/许可清单/校验值、最终ZIP仓外试玩验收与用户移交；依赖072真实交付。
- 071/026/027继续M1原生债务线，不成为本首版便携游戏的整包前置。两游戏发布候选不冒AGE M1完整签章；若公共接口交集变更需稳定SHA单写，实证受影响面归各owner。

## 3. 运行数据流与先证

便携exe旁`player.json`（拟新增版本化启动描述）→允许的game_id/相对scene路径/profile/display参数→Storage::load_project受检项目/资源/行为闭包→现有Host/Play→真实GameInput/Tick/相机/原生audio→只含游戏视口和HUD的持续OS窗口。Root以exe所在目录确定，绝不能依赖cwd、生产者绝对路径、环境变量或下载源码。配置、scene与资源sha必须互相绑定；不允许未知profile自动退回sample。

072先用现有两游戏各一条从真实OS输入到GPU/音频/失败重开的完整路径决定最小复用：比较现有Host最小呈现模式与薄Player壳两种以内方案，复用同一Play/资源准入/renderer/音频；禁止新增第二物理/游戏解释器。不能只把editor窗口藏菜单或把--inspect改长帧数当完成。源协议若需变化先有限修订，不在包装阶段引入新预算/玩法。

## 4. 产品行为

启动即进原游戏Playing，可看见操作说明，无须编辑器Open/Run/选择对象。2D方向键/空格/终局fresh Enter；3D WASD/方向键/空格、鼠标环绕/滚轮、终局fresh Enter，沿原映射与对角归一/相机夹界。失焦/最小化不留held、捕获、音频重触发或无界追帧；一Tick恰一次动作快照、原固定dt及服务预算，不用渲染帧数代逻辑推进。窗口重绘/resize不重复Tick。HUD与声音来自原Auto输出，不用Rust根据胜负另画状态。

原Native音频确有CPAL后端，不以离线PCM对照代实际声音输出。支持无音频设备时准确可理解的错误/显式静音选项（如需）；静音不能使音频验收PASS。坏资源/缺脚本/未知配置/不支持GPU初始化须在公开入口解释并非零终态，不静默开空白窗。支持范围内surface重建/resize按已有机制；不在本包承诺新完整跨device恢复架构。

## 5. 发行目录与身份

每个游戏单独携带同一正式player的可执行文件（允许字节相同、文件名不同，必须如实记录公共runtime身份）：

```
Platformer2D/                         Platformer3D/
  Platformer2D.exe                     Platformer3D.exe
  player.json                          player.json
  data/scene.agscene                    data/scene.agscene
  data/scripts/<sha>.at                 data/scripts/<sha>.at
  data/assets/<sha>.<ext>               data/assets/<sha>.<ext>
  THIRD-PARTY-NOTICES.txt               THIRD-PARTY-NOTICES.txt
  licenses/                            licenses/
  README-zh-CN.txt                      README-zh-CN.txt
  release-manifest.json                 release-manifest.json
```

资源可采用原内容寻址布局，无需自创加密pak；3D GLB外部Textures相对URI闭包须完整。需要的非系统DLL按实际PE导入/干净环境运行结果及许可随包，不能猜测“Rust静态所以无DLL”。操作系统组件/显卡驱动为平台前置，清单写实测条件，不复制系统目录凑包。原源码离线包仍按原工具保留，不能改名冒预编译便携包。

发行ZIP拟`platformer-2d-windows-x64-<build-id>.zip`与3d同构；build-id绑定实际UI/AG/lang/SDK、工具链、target、release配置及原内容hash，附SHA256SUMS/文件大小、资源/运行库/许可证列表、已验平台与已知限制。二进制哈希受环境影响，验证最终实际字节，不默认跨机器可重现。用户数据写用户可写目录；QA显式覆盖到本Plan scope，不能写安装资源或源码树。

## 6. 验收与执行顺序

072先证→公共player完整闭环/定向正负控→独立review/落主。028可先准备清单/包装/旧证据映射；待072交付后以最终release ZIP解压到无仓库、无Rust/Python/源码、无网络依赖的Windows用户环境，从不同cwd/含空格中文路径分别双击运行。用真实输入玩出失败、新局、获胜、新局；同时用原独立oracle复核规则/预算/画面/PCM/HUD。原生试玩不以预录输入轨迹或inspect退出0顶替。实际操作留证需手工或明确限定会话授权，不将原债务线的授权扩大到全局。

资源遗漏/损坏、脚本错sha、改配置/profile、缺DLL、只inspect一帧、抓不到输入、音频无输出、后台卡键/重开状态泄漏/退出不回收都必须能被验收拒绝。最后一个有效ZIP与清单/报告绑定；修复后打包字节变更，重验其失效面，不仅测试未打包开发树。

## 7. 材料、交付和长期保留

全部构建/QA/外部解压/打包暂存仅在`D:/autostack/plan-artifacts/plan-072/auto-ui/`与`plan-028/ag-examples/`，不另建散落发行根，不把exe/ZIP提交Git。072的runtime交接包从正式源码/构建manifest可在028 scope复建；不能让028依赖merge后被删的072临时exe。

028最终两ZIP、校验表与用户操作说明为本任务交付物，必须在最终回复给出实际可访问路径。现行规则要求merge全清，但允许“具名未解决问题持有（owner+理由+关闭条件）”。若用户尚未取走或未指定长期发布接收根，登记`REL-DELIVERY-001`：owner=本发行交付协调者（用户确认接收方），问题=最终二进制发行物仍待用户验收/接管，持有仅最终ZIP/清单/README及hash，不保target/SDK副本/巨大日志。关闭=用户/指定接收方实际取得相同SHA256字节并记录接收路径/时间，再安全释放该scope。该项必须明确报告为“用户移交待确认”，不能虚报整个scope已删除，不能把需要交给用户的ZIP在回复前清掉。若用户先明确授权永久发行接收目录，届时按用户指令移交并对账；本轮不推断任意盘符/远端发布权限。

只构建本地可分发候选，不自动上架、推送release/tag或申请代码签名。正式远端渠道、产品命名/版本、是否签名后续确认；当前目标不以商业上架审核通过为验收。

## 8. ACE迁移接洽

player是旧driver/SDK的新增产品入口，不得漏出迁移分母。072/028冻结实际入口/构建/资源/包consumer，在各自Spec和release manifest记迁移归属`auto-lang/PLAN-981`的SOURCE/SESSION/CONTENT/AUDIT，最终安装与旧binary清理由986/987接收。不得让981反过来成为本发布的整阶段前置，也不得因此允许隐藏旧VM fallback。若执行时旧VM已正式退役，T01须改用届时已交ACE producer；未就绪则具名修订，不复活被禁默认入口。未交付草案不当作现存发布物；实际新增消费者交付后追加到正式分母。
