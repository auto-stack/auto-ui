# 状态实例与Reload：主题收口后的框架接续设计

> 2026-11-04，target-state r1。接受031/033/036与918已归档的有限交付。用户本轮明确规划auto-ui与auto-lang两个仓；仅调研/立正式合同/刷新记录，暂不实施。本文不是新API或运行PASS，所有探针均待授权work。

## 1. 本轮选择

| 主合同 | 完整模块 | 本轮界限 |
|---|---|---|
| [UI041](../../plans/041-component-and-store-instance-ownership.md) | 组件model/props/handler receiver按实例归属，store按App实例共享、不同App隔离，退出不串/不泄漏 | 现语法和有限内部端口，保显式共享与既有store路由；不同时实现完整Feature/命令框架 |
| [UI042](../../plans/042-reload-schema-and-effect-readiness.md) | Reload多文件依赖、schema/状态与effect owner的有界整链研究验证 | 形成实际支持/拒绝/原子失败/后继实施合同输入；研究不冒完整HMR实现 |
| [语言930](../../../../auto-lang/docs/plans/930-generated-delivery-regression-and-source-boundary-closure.md) | 036最终收据保留的生成交付3红与独立源L1/L2守门 | 修driver过时成员定位、实证V05a实际根因，必要UI投影由UI owner参与；不接管语法/ACE/整个Cargo问题 |

教学037/038已有归档与最终收据，038研究不是迁移；新B3教学045/046/047与OS055–061沿其合同继续，不整体等待041/042/930。仅当其真实选定运行链违反既有主题/生成/实例合同才按反例与owner交接；不把新State/Reload当全部示例/产品主题迁移前置。Tauri R-6仍是036具名非阻塞后继观察，未由本轮三包自动关闭；Feature/命令目标/Service仍需以后按实际consumer另约。

## 2. 已观察代码与权威来源

已读UI Spec索引、runtime/vue-bare-store-routing、主题scope/host transport及036归档；语言Design47/38作为目标输入、现仓界/生成生命周期和交付Spec、918准入规则。观察HEAD：UI 92ee2459a1058ddddc3fc3378ecec6585ffe726c、语言aa6791649be2e2386c15b334d91fff74975cf684、Man 2bbd9542c708c174c2ac71cd9c1f26811cc10c4c。按实际工作目录定位，不以HEAD代源码hash；工作重核完整来源和WIP。

| 证据 | 静态实物 | 不能据此推断 |
|---|---|---|
| UI runtime ui/vm_bridge.rs::new_from_decls | 注释/代码仍合并child/store同名model字段入根；ensure_child_state写props到root并返回root_id | 不是所有VM对象都扁平，也不是所有独立store都不存在 |
| ensure_child_state | widget_name/state_field_names未参与实例分配；已有changed判断/RC materialize/release与bump_path | 只换对象ID而不改view/handler/写屏障会导致另一套假隔离 |
| compiler ui_gen/vue/store.rs::generate_store_composable_full | module-level ref singleton与module-scope watch | 单组件内共享合法，不代表多个App挂载应隐式共享；不得删旧共享来凑隔离 |
| runtime ui/dynamic.rs::reload | old_field_defs为空，新bridge迁移写回存在忽略错误路径 | 不能承诺类型安全迁移或失败原子性已完成 |
| check_file_changed | imports或widget_registry非空时直接Ok(None)；单源以mtime判更新 | 保守跳过避免旧破坏，不等多文件热重载实现，也不宜仅删guard |
| 语言generated-ui-consumer/driver.py V04 | member写死rust-workspace/dev-run | 910已交付成员为basename-id8；两条断言可能因找错文件红，不能据此改回旧目录 |
| V05a | 在真实bundle A中cargo build --locked -p auto-ui-tools，失败即停止 | 036已把它列原红/815投影owner；本轮未重跑，不先指定一定只是测试漂移 |

036最终archived10/10，交付UI e06b65b2/Man 2bbd954/lang 2f558fc93，原材料已删；当前看实际源/归档摘要，不要求旧scope存在。它新增渠道与包输入，未实现实例状态/Reload或关闭V04×2/V05a，不能以主题成功推定这些通过。语言当前927 reviewed10/12、928/929 drafting为独立语法/ACE线，本轮不改其合同、基线或取走实现。

## 3. UI041目标语义

- **App实例**指一次独立启动/挂载的root context，不以产品名或线程判断；两个同名App/root也隔离。同一context内部对同store定义的既有use继续共享；跨context共享只能通过显式注入/共享owner端口，内部机制由041 T-01冻结，不新增一套.at语法。
- 组件model/props属于具体实例；显式key存在时保keyed重排身份，未keyed的旧位置语义需写明，不凭类型名去重。view读取、handler receiver、computed/store方法、脏位与RC释放都用同一对象身份；改两个组件props的最后写入顺序不得改变另一实例。
- App/store/child可在同一VM中有各自对象，不机械每组件新VM。保现typed heap引用和419所有权/写屏障，避免把对象隔离变成拷贝整业务。
- Vue生成context/factory与setup订阅对应真实App owner；旧裸store.Method路由规则继续，不能以改方法归属当解决实例问题。无App上下文的旧入口若需legacy适配须明确标边界，不让新多App路径退回隐藏singleton。
- 删除/关闭一个child/App只撤自己的订阅/Tick/handler绑定与对象；已有显式共享的最后owner才释放，延迟回调不得重定向到另一同名实例。保持031主题scope与业务owner分离，不把主题选择当业务store。

首批验收为两个永久最小回归输入（新fixture，不改产品）：同型双计数/输入组件，以及同App共享store、两个App隔离的fixture；真实Vue/VM事件、keyed重排/关闭与存活实例继续操作共同证实。另选B1现有计数/表单与真实多store路由消费者作为兼容控制，选择按正式源且不借其他计划WIP。

## 4. UI042有界研究合同

初期可用现实现核根因，不等041整项；依赖实例/schema端口的新路径只接041正式结果。整链至少覆盖单文件有效修改、真实import/store/child多文件依赖修改、同名字段类型变更/增删、编译或迁移失败后旧界面继续操作、effect/Tick/订阅最后owner与再次合法修改。

观察结果分“支持并实证”“明确拒绝/需重启”“具名阻断/未测”，不能把Ok(None)或Vite更新当AutoUI HMR PASS。记录保持/迁移/局部重建/拒绝的schema判定和资源预算；candidate先完整解析/迁移/校验、发布一次，失败保旧字节/状态/effect，不通过重启App消除错误。研究可以指出当前做不到并给实证前置，不能仅靠文档设想给go。

UI042不顺手修完通用HMR。输出可直接立后继实施合同的源/依赖图/版本/schema/effect owner/原子发布端口、正负反例及支持边界；不预分未来实现号、不降低现承诺。

## 5. 语言930与跨仓写面

语言driver拥有“实际验证对象是谁、用例是否真的执行”的合同，UI拥有生成器/投影。V04按910公开member身份选择并核canonical项目身份，拒绝同basename错成员/模糊多个候选；不能只把glob第一个文件当正确对象。main.rs同源断言仍需读到实际dev/交付文件后比较，不删该义务。V05a先以冻结真实bundle复现和定位，若是UI交付投影缺陷，在同组UI owner的窄改面修，不在语言仓复制UI算法或删tool发行面。

完整V01..V06实际消费、失败恢复/工具wire、主题输入保留；jet/ark原环境门控条件保持，不因本包扩到所有平台。独立源L1 Core与L2完整workspace分别核，不能借隐藏用户仓/旧target/删成员实现。已有证据仅输入身份未变才复用，930新变化自己的审计基线按918规则重冻；其他语法/主题owner的变化不得一起无审查吸收。

共享UI runtime/compiler、语言driver/生成核心与manifest/锁短窗单写；先读已交付组合，041与930互不整项等待。042只在消费041新增实例能力的验证腿等待该接口，不先要求所有示例/OS产品迁完。

## 6. 边界、资源与后继

本轮不改业务产品clone、OS pin、用户系统主题/文件。未来永久回归fixture进入对应产品测试源码，临时探针/日志/外部consumer/target仅统一plan-artifacts/plan-NNN/<repo>/；普通构建在唯一树。实际资源/支持/命令在各T-01有界冻结，未给无限规模承诺。代码/ABI/语法或支持承诺扩大须具体重规划；常规等价内部机制不重复申请权限。

各Plan独立review同组/同树，review验证不修实现；merge冻结必要职责并集，不全AG/ACE/全机回归。必要摘要折Plan后删自有scope/树产物，wt-guard clean才移除，未知/他方不盲删，canonical在docs/specs、旧JSON只读。

## 7. 接受038实测后的教学B3正式实施批次

038 archived8/8，实测canonical将十二例分三完整模块，已正式立约：

| 合同 | 全部目录 | 已知门 |
|---|---|---|
| [UI045](../../plans/045-media-demo-theme-migration.md) | 019/020/030，3例 | 归属根links冲突、030无media back contract/mpv、020死代码接线与真实播放条件逐项核，不以scan/降级页抵必需播放 |
| [UI046](../../plans/046-image-and-file-demo-theme-migration.md) | 029/031-image-viewer/027，3例 | 031iv七a2r编译错/VM挂起硬阻断，photo_root与原文件/性能条件；静态零hex不算运行成功 |
| [UI047](../../plans/047-specialized-workbench-demo-theme-migration.md) | 018/041/026/025/024/031-paint，6例 | 018挂载/路由五红、旧测试定位、四处主题声明；编辑/表格/series/画布内容owner明确 |

三批均drafting r1、0/9，本轮暂不实施。041/042并非主题批次统一前置，930只接确命中同一生成来源/交付根因的腿，不包揽所有媒体/图像类型问题。当前教学B1+B2按canonical累计22例，B3十二仍未迁；UI037头部7/11与最终archived11/11收据不一致，由owner修簿记，不重开归档。037复审来源/首红和vue-render新渠道待办按其最终记录保留，036仅Rust delivery-root支持不被扩成所有Vue发行已交付。真实产品按OS Design03单独验收，不从Demo抄业务。
