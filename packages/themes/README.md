# AutoUI 官方主题包（正式）

> PLAN-031 T-04 / SD-03 的单一可编辑源。包名 `themes`，正式版本 `0.2.0`
> （自 PLAN-007 试点 `examples/ui/themes` 0.1.0 迁移）。

## 内容

- `themes.at`：唯一颜色数据源。`pub theme Auto` + `pub theme Solarized`
  （`extend("Auto", …)`），各含 31 公共键与 light/dark 模式差异；
  `Solarized` 的 light 差异按族自足显式重声明。
- `pac.at`：Auto 包描述（name/version/scene/render/title）。
- `theme-package.json`：受管来源/能力清单——owner、包版本、schema、文件
  sha256、family 支持模式与默认。**不是第二份颜色数据**。

## 消费

开发期（本仓示例）：

```
dep themes { path: "../../packages/themes" }
use themes.themes: Auto, Solarized
```

应用经 `extend("Auto", …)` 合成自己的主题，不复制配色表。

## 不变量

- 修改 `themes.at` 后必须同步 `theme-package.json` 的文件 sha256（编译器
  测试 `theme_package_manifest` 会失败）。
- `examples/ui/themes` 只保留迁移 README，不再有可编辑 `.at` 镜像。
- 外部发行快照（`vendor/themes` 相对 dep）由 UI 受管 pipeline 生成，
  带 owner/version/hash 校验，不可编辑、不是第二权威源。
