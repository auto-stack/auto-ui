# 拆分路线下一批：坏源恢复、原子Reload与Vue真实实例隔离

2026-11-05，target-state r1。只规划，未实施；当前行为仍以canonical Specs为准。

## 1. 正式合同与顺序

| 主责 | 合同 | 启动/交付关系 |
|---|---|---|
| auto-lang，auto-ui参与 | [940解析推进与坏源恢复](../../../../auto-lang/docs/plans/940-parser-progress-and-malformed-source-recovery.md) | 接042历史观察；T-01先安全监督复现和真实Core/UI归因，绝不裸跑51GB输入。根因属于UI则修UI，不反向拉实现到语言仓 |
| auto-ui | [056原子多文件Reload](../../plans/056-atomic-multifile-reload-and-schema-lifecycle.md) | 接041/042；合法快照/迁移准备可独立推进，危险坏源腿与最终交付接受940正式结果 |
| auto-ui | [057 Vue真实双App/Store隔离](../../plans/057-vue-app-store-isolation-and-owner-lifecycle.md) | 接041/050/051，可与940/056独立；store/context共享符号短窗单写 |

这是三个完整模块，不把Feature/命令/Service、全部发行渠道或AGE新语法自动纳入；OS主题与教学Demo不再整体等待Reload。

## 2. 当前行为与不可越界项

042归档仅证单文件与拒绝矩阵：imports/registry形态guard整体跳过；同mtime内容改变漏检；reload旧schema为空，同名类型变化未判型；绕过guard会丢child/store/handler。940解决坏源受控返回，056才改变现役Reload事务。禁止仅删除guard、拼旧/新桥、忽略迁移写错误后宣称成功。

041已交付实例机制，051又修改MCP/page-named派发边界；两者现行Spec优先于旧041设计草稿。Vue目前仅生成契约覆盖，无双App浏览器实证。057验证真实生成模块同页双挂载、同App共享/跨App默认隔离/显式共享，并按实际watch/timer/SSE owner关闭；不把字符串断言当运行隔离。

056的来源root/import/store/child/handlers/schema/effects同快照；准备无外部effect，错误保旧，合法下一编辑恢复。同名类型变化或不透明类型默认拒绝，活跃parked任务不能迁移时具名保旧等静止，不伪造可回滚外部副作用。完整Feature/Node/外部Python/web模块热替换、公开MCP组件reload API、Vue/Vite HMR不在本包。

## 3. 观察基线与内容身份

静态核查语言HEAD 5f67374675e1ea9c5278e39d0dd24849faa7e8dc；UI观察HEAD e3f0933d2384921aa49df8007cf9730ad7723692（其他会话继续合入，work需重核）。HEAD不替代实际内容：

| 仓/源码 | SHA256 |
|---|---|
| lang crates/auto-lang-core/src/parser.rs | 463cda800422a35c4f845a529c4ecdc900b3b8e89e8d34b24fff60e1f71c33fd |
| lang crates/auto-lang-core/src/parser_bridge.rs | d9356056fd36a19931e79cc789d11483e806718238e2dd3ab9d7b831fd746122 |
| lang docs/specs/auto-parser-api/project.md | e14bf0c489bf20123c3a195658c5fb4c024365bd2553fca94543b69bb40db4fd |
| UI crates/auto-ui-runtime/src/ui/dynamic.rs | 23b5314ba8fd17cf4834046fe0cf9a4e0653a48a65a2075ce4b693311ff5851e |
| UI crates/auto-ui-runtime/src/dynamic_build.rs | daa4f34fda98232df557070279e547cf7a2483dc4cfaa5be20d33f662c7b868d |
| UI crates/auto-ui-runtime/src/ui/iced/renderer/host.rs | c04238690991b6b3d501802a608e3452889fcaff2323198711f3b17d902b2bab |
| UI crates/auto-ui-dialect/src/grammar/widget.rs | 0ea292c8c1201cce9de2d4bf2620673230b3f118e754094c0d30748fe12c3b63 |
| UI crates/auto-ui-compiler/src/ui_gen/vue/store.rs | 4856bf1abeb03bfced34b2afda4e6a7b17022c321fac9f81a35d5411b57fda15 |
| UI docs/specs/runtime/reload-readiness-and-lifecycle.md | 125c1b08d40472113ae3ea343ecf73b12745095cac8fba04b8105ec4b7bce7bf |

资料里的截断widget是示意，不是保存的逐字51GB输入；940 T-01从合法fixture构造确切字节及hash，小样本受监督运行，资源停止不能记语法受控拒绝。10秒/512MiB仅试验保护上限，不是语言支持限制。无执行证据。

## 4. 其他在途与交付边界

UI052已archived5/5、生成器交付64aef780；其修复不能证明UI053多API surface/栈问题已闭。UI053现drafting0/0是有事实的待完善草案，由OS054原owner补齐执行合同，不在本轮改其AC或接管WIP。OS054仍executing4/10、接052正式源后继续；OS055执行中，真实系统事件待窗口许可，不由Reload试验修改用户主题。OS060现已归档9/9，OS061仍草案。

UI054/055为AGE独立路线，语言934–939为SQ/CA/ACE路线，不自动并入本拆分合同或设全局前置。Tauri R-6、Vue新发行渠道、Web schema/锁/source staging与Man派发差额按各owner保留，未经调研不在本轮虚构统一治理实施包。

## 5. 验收与交付

每份合同已有T/AC/V/SD、输入/入口/区分性oracle、支持边界及work/review/merge归属。实施与独立final review同权威树顺序执行；全部新运行待授权。受影响完整职责，不以全ACE/全AGE/全语言代替选门。资料仅plan-artifacts/plan-NNN/<repo>，merge折摘要、fresh guard后删自有scope和树，保他方WIP。
