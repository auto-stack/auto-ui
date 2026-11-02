# Theme Lab — 主题树试点（PLAN-007 §5.1）

固定试用面：官方候选主题族（Auto / Solarized，各 light/dark）、树状
extend、类型化槽引用、两个 CapsulePanel 实例隔离（含子级覆盖）、换肤后
业务状态保持（输入 `draft-42` / 计数 `2` 为源内 oracle）。

## 运行

```bash
cd examples/theme-lab
auto run              # Vue 轨（浏览器，dev server）
auto run -r vm        # VM/Iced 轨（原生窗口）
```

依赖：同仓 `examples/ui/themes/`（pac `dep themes { path: "../ui/themes" }`），
无需安装；`auto-ui-tool` 需在 PATH 或同组 target（`auto run` 自动发现）。

## 操作步骤（观察点）

1. 首帧：两个 CapsulePanel —— A 的圆角来自 `tokens.radius.control`
   （槽引用，随主题联动）；B 为 `24dp` 直接字面（不随主题变）。子面板
   覆盖 `muted` 底色（实例隔离：A/B/父级互不串色）。
2. 点 **Solarized** → 全界面切 base3/base2 色板（Vue 轨 CSS 变量、
   Iced 轨语义解析同步）；A 圆角仍随各族的 `radius.control`。
3. 点 **切换明暗** → dark 差异生效（Solarized dark = base03/base02）。
4. 输入框输入、点 **+**（count 2→3…），再切主题/模式 → 输入与计数
   保持（换肤不触碰业务状态）。
5. 点 **Auto** + **切换明暗** 回 light → 色板与状态完整往返（无残留）。

失败面：把 `design.theme` 绑定改成一个未注册名（如 `"Nope"`）→ 诊断
输出且保留旧有效主题（事务，C-11）；把主题 `modes` 去掉 `dark` 后请求
dark → 拒绝换肤（C-07）。
