# image-preview demo（PLAN-089）

官方 `packages/image-viewer/` 的普通消费演示：受控回路
（viewRequested → baseRevision 校验 → 原子赋值 + revision++ → Tick 重投影）
+ 真实 media ticket URI（前端进程内建 `auto.image` 会话）。

- 产品门：`python tests/product/verify_image_preview.py --auto-bin <auto> --ui-tool-bin <tool> --out-dir <isolated> --backend native|vue`
- 夹具：`tests/fixtures/`（corner-marked.png 生成法见 manifest.json）
- 已知平台边界：前端 VM 进程 cwd 非项目目录——夹具与包路径的相对形态在该
  进程不解析；产品门 gate 期原地重写为绝对路径并字节恢复（PLAN-089
  decisions.md §7 P2/P6）。
