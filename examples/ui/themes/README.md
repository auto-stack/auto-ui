# examples/ui/themes（已迁移）

官方主题包已提升为正式 UI 包资产：

- 唯一可编辑源：`packages/themes/`（包名 `themes`，版本 `0.2.0`）。
- 本目录不再保留可编辑 `.at` 镜像。旧 `dep themes { path: "../themes" }`
  的消费方请改为指向 `packages/themes`（示例见 `examples/ui/002-counter/pac.at`）。
- Auto/Solarized 两族的颜色数据、31 公共键与 light/dark 模式差异随迁移
  原样保留；来源/能力清单见 `packages/themes/theme-package.json`。
