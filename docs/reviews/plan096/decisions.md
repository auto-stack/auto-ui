# PLAN-096 实施决策

计划 r3 的交付平台为 Windows Native VM 和 Windows Vue。实施树为 `D:/autostack/.wt/ui-096/auto-ui`，分支 `codex/ui-096-command-palette`，基线 `4dd9696da6a6289795ba9d6f57e603c24d99114a`。

## 依赖和工具

| 来源 | 固定版本 |
|---|---|
| auto-lang | `8384ea064a78dd3d0b1de3d9c1c47bafd4d061f0` |
| auto-web | `0cfc7c78446650631d244a9ea442d28635072958` |
| auto-down | `456c86ac886f9781e56f6153ce9aa41cd59e5c8e` |
| AutoOS host source | `cdb4883812229576b1f21e6124cd900bfad7c630` |
| Auto CLI | `auto 0.1.0+plan-890-verified-r7-t19-116-gc879b3e`，SHA256 `eddb2cce50fd8afe08cd26959c2fe52e090eb833b5c8b9a3f27538b28d947fcf` |
| 构建和浏览器 | Rust/Cargo 1.98.0，Node 25.2.1，pnpm 11.6.0，Python 3.14.2，Playwright MCP 0.0.86，Chrome 157.0.8103.80 |

参考 Launcher 源码 SHA256 `4EAEC20D6F87C52B53D37D6AB7CF2DD82E7BFC087B67C52F5F656F5565D0206E`。参考图 SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13`。主检出上的其他计划文档改动已在计划 §4 记录。依赖通过同组兄弟树解析。094 的旧依赖组缺少基线所需的 `auto_lang_core::file_session`，本计划重新固定到表中的版本，按原锁文件构建成功。

## 普通控件链

- `CommandPalette` 由 `pac.at` 依赖和 widget 相对包导入装载。Native VM 和 Vue 均通过 `Dialog`、`Input`、`Button` 和可滚动标准容器运行。Builtin 注册优先级及 `Command` 族保持原有行为。
- Vue 已有 Command 项目组件会自行过滤并持有键盘行为。现有 Native VM 的单行 Input 探针没有方向键消费入口，模态 MCP 将 ArrowDown 当成后台快捷键拒绝。选择标准 Input 并补有限的 Up/Down 绑定。Iced 在真实聚焦输入框里派发，MCP 查当前模态并通过同一实际焦点 ID 查找绑定。TextInput 组合输入时保留 Enter 和已绑定方向键给输入法。
- 首次普通 Tick 生成 UUID 并保存为实例身份。每次打开获取新会话和控件绑定。首次错误通过 Tick 的公开消息派发。提交前核对当前命令指纹、修订和结果请求，点击另带结果代次。Native VM 和 Vue 共用同一组 `.at` 模型函数。
- Vue 生成器在内联帮助函数时会将与模型字段同名的自由识别符当成 Vue ref。局部参数和模型使用不同名称。标准输入使用 shadcn Input 绑定以接收字符串和已有 IME 守卫。
- Launcher 在 provided 模式发布原 `ApplyFilter` 的排序结果。分类胶囊及进入网格的按钮由应用经默认 slot 填充。原生宿主权限和 `Launch` 保持在应用层。Vue 独立运行显示具名拒绝。公开结果标签由可见文本产生。

## 运行入口

`cargo build --locked --offline -p auto-ui-tools --bin auto-ui-tool` 准备工作树工具。设置 `AUTO_UI_TOOL_BIN` 后，用 `auto run -r vm` 运行原生独立消费示例。Vue 在移根项目执行 `auto gen -p`、`pnpm install --prefer-offline --ignore-scripts`、`pnpm exec vue-tsc --noEmit` 和 `pnpm exec vite build --mode development --minify false`，再运行现有 Playwright MCP stdio 提供方。各轮机器证据在 `D:/autostack/plan-artifacts/plan-096/`。
