# Vue 桌面宿主的应用 API 面

实现来源：auto-ui/PLAN-053；本规范由 auto-ui/PLAN-060 为 auto-os/PLAN-054
集成并验证已有兼容路径，不表示 PLAN-053 已完成。生产入口为 `VueProject::generate_desktop_host` 与
`run_vue_flow`。

- 每个应用的自包含 API 客户端写入 `src/apps/<id>/lib_api.ts`。源选择保持
  `gen/front/vue/src/lib/api.ts` 优先、`src/back/api.ts` 次之；缺失仍跳过。
  应用内已知的 API、组件与 store 模块 import 改指本应用目录；相同文件名
  不再由其他应用首胜覆盖。宿主 shadcn/UI 运行时仍共享。
- 客户端和生成的组件/store 中，以单引号、双引号或模板字符串开头的
  `/api/` URL 字面量带 `/apps/<id>/api/` 前缀，已经带前缀的内容不再改写。
  不承诺解析任意手写 TypeScript 模块依赖或动态拼接 URL。
- `desktop-backends.json` 只记录本次获准嵌入应用的根、后端 profile 与端口。
  `api: "rust"` 使用既有 Rust API launcher，并要求非零 `back_port`；
  `api: "vm"` 或未声明 api 的既有 back 契约使用 VM session proxy。
  Rust 请求经 Vite 脱去应用前缀后转发到本应用端口，VM 请求保留前缀由
  session proxy 分派。Rust 应用不回退到 VM。重复端口或已有监听具名拒绝，
  不为新桌面应用清理既有服务。Rust 启动或 proxy 绑定失败中止进入 dev-server；
  VM session 保持既有按需加载及失败时返回 503 的行为。
- 桌面后端通过既有 owned-process 设施持有，退出/取消与启动失败时清理
  本次启动的 Rust 子进程树。VM proxy 复用既有进程生命周期设施。
- 有 API 客户端的路由应用使用既有 gallery 的页面发射与 memory-router
  工厂，通过薄 Vue 包装在窗口内挂载；每次挂载新建 router，卸载释放 Vue
  app，安装既有 store context。纯前端 routes、ext、i18n 的旧跳过守卫保留。
  单视图应用的每窗口 store 生命周期不在本合同内。
- 既有窗口关闭控件的 pointer-down 不进入标题栏拖动捕获；一次普通点击须
  到达窗口关闭路径，移除窗口并卸载其挂载内容。标题栏非控件区域的拖动和
  八向缩放仍走既有 pointer-capture 路径，不扩大单视图 store 生命周期承诺。
- 发射时将模板根的屏幕尺寸约束改为窗口客户区尺寸，并允许根内容滚动；
  不依赖可选 overlay 标记，不改内部画布尺寸。
- VM 后端初始化包含模块变量与常量。参数优先按 path/body 字段/query 绑定；
  单个字符串参数未命名命中且请求体非空时，保持 standalone 的整个原始请求体
  约定，然后才处理 meta。多个参数缺参仍返回 400。
- 应用声明的 npm 依赖合并在宿主 package 同步之后执行，`file:`/`link:`
  路径以应用原生成目录为基准解析；同名 npm 包维持既有 first-wins 策略。
  合并结果写入 `desktop-npm-deps.json`，生成期与桌面 run 自愈均使用这个
  相同集合发射 Vite include；非桌面来源保持原 `VueProject::npm_deps`。

视图解析器的元素嵌套使用堆上的父节点栈，表达式/属性解析临时量在进入
下一层前释放；`else if` 链和 widget 的 inline/input/modal mint 遍历使用
迭代且保持先序。不通过增大生产 worker 栈或增加语法深度拒绝阈值处理
桌面扫描溢出；验收须覆盖实际主根及自动追加的外部根。

## Rust 服务构建与运行边界（PLAN-956）

Rust API launcher 先在 GeneratedBuildContext 的 workspace cwd/受管 flags/
目标根内完成 Cargo build，从 Cargo JSON 取得本成员可执行文件并校验/发布。
编译失败立即返回，保留 compiler-message rendered 诊断；不把既有二进制当新
成功、不启动服务。构建使用既有取消与构建预算，不消耗服务的 120 秒 ready 窗口。
随后启动发布的原生镜像，才开始 ready 检查；超时/失败仍中止 UI 初始化并回收
拥有的子进程。启动子进程失败不能吞成 Ok(None)。

运行 cwd = 应用根，与 VM 路径相同；Cargo cwd 仍是生成工作区，保留 PLAN-910
的配置隔离。相对资源（如 Reader src/assets/engine-assets）不依赖业务专属
环境变量补丁。服务生命周期内保留 target lease，托管与 legacy 启动均执行
同一构建/发布纪律。

## 声明共享 daemon 时的后端归属与去重（PLAN-083 SD-02）

- 应用的 pac `back_port` 与 `apps.manifest` 同名条目 `daemon.port` 相等时，
  该 daemon **就是**本应用的后端：桌面启动器不为它启动第二份自有后端，也不
  把该端口已有的监听视为"已被占用"而中止；路由（`desktop-proxies.json`）
  直接指向经身份核验的 daemon，`strip: true`（前端 `/apps/<id>/api/...` 去掉
  应用前缀后由 daemon 的 `/api/...` 承接）。
- 该判定只在声明与身份校验之后发生：先按 `docs/specs/runtime/featurepack-daemon-hosting.md`
  §6 取得/复用实例（跨进程单实例门、实例标记核对），失败即具名中止桌面后端
  启动——不得悄悄退回"另起一份后端"。
- 非共享的声明 daemon（`back_port != daemon.port`）同样在**后端 spawn 之前**
  ensure 并解析宿主配置：就绪即随后端子进程投递；声明非法 → 拒绝启动其后端；
  daemon Offline 仍按既有"Offline 不阻断"语义启动后端，但明确记录"后端在无
  宿主配置下启动"（运行后再补投递不可行）。
- 普通端口占用（未声明 daemon、或声明端口与应用 `back_port` 不同）保持既有
  规则：具名拒绝，不为新桌面应用清理既有服务。
- 后端配置归属：声明的宿主配置（地址键、`host.env` 非敏感 literal、受管
  凭据引用）随该应用后端子进程 spawn 投递（剔除继承同名键）；共享 daemon
  自身的服务配置只随 daemon spawn 投递。in-proc VM 宿主无法按宿主隔离凭据
  （`Env.get` 进程全局），该缺口见 runtime 合同 §5，不得用运行期改写全局变量
  假装隔离。
- 生命周期：共享 daemon 独立于任一主窗口/宿主进程（detached，归属由实例标记
  记录）；自有 Rust 后端仍由既有 owned-process 设施按原规则清理（退出/取消/
  启动失败回收本次子进程树）。
- 既有 Rust 服务构建/启动纪律（本页上文）不变；运行时资源与 cwd 语义不变。

