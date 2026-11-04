# 生成 Vue 工程 vite 配置发射（generated vite config emission）

> PLAN-050 新增合同。生成期与 run 期自愈共用唯一生产模板
> `auto-ui-tools/src/vue.rs::generate_vite_config`（compiler 侧
> `ui_gen/vue/project_files.rs` 同名副本为 test-only 旧形态，不在本合同内）。
> 分类：build ｜ 状态：active ｜ 来源计划：docs/plans/050-vite-config-optimize-deps-and-strict-port.md

## 1. 发射合同（恒真）

- 生成模板的 `server` 块恒发射 `host: '127.0.0.1'` 与 `strictPort: true`：
  端口被占 = vite 具名失败非零退出（"Port … is already in use"）；禁止
  端口静默自增（可撞后端端口呈 split-brain）与同端口跨协议栈静默绑定
  （127.0.0.1 被占绑 ::1 仍报 ready）。`localhost` URL 经 OS 双栈回落照常
  可达；tauri webview 的 `http://localhost:{port}` 不受影响。
- `optimizeDeps.include`：dependencies 键集非空时发射 `optimizeDeps:
  { include: [...] }`，键 = **写入该次运行时 package.json `dependencies`
  的同一键集**（键为 TS 字符串字面量，JSON 转义）；键集为空时整个
  `optimizeDeps` 键省略（不发 `include: []`）。不变式：include 键集 ⊆
  实装依赖，零"引用未安装包"失败面。

## 2. 两条写入路径（同源）

- **生成期**（`write_project_files`）：include 源 = 遮蔽后的 `extra_deps`
  （Plan-482 `@autodown/engine` 同级推导后的最终集合），与同一函数写入
  package.json 的集合逐字同源。
- **run 期自愈**（`run_vue`）：include 源 = 磁盘 `package.json` 的
  `dependencies` 键集（`package_json_dep_keys`）——手改与历史项目的唯一
  真源。解析失败（损坏/缺 dependencies/缺文件）= 保留原 vite.config +
  具名警告，不中断 run（与自愈写失败同级语义）。

## 3. 动机与边界

- 动机：vite dev 冷扫描只见静态可达 bare import；SFC 内动态导入链
  （如 mermaid→dayjs CJS 互操作）不被预打包导致运行期求值断裂
  （auto-blog F-2 实录，cand5 控制变量验证 include 修复全绿）。
- 静默改道语义变化为有意合同：多实例第二实例由"静默自增"改为"显式
  失败"，与消费方故障门（auto-blog BLOG-005 B-01）一致。
- `optimizeDeps.include` 只作用于 dev 预打包；build/rollup 输出配置不在
  本合同内。
