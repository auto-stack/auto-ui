# iced_wgpu 本地补丁（auto-ui）

对 crates.io `iced_wgpu-0.14.0` 的最小手术，经根 Cargo.toml
`[patch.crates-io]` 挂载（与 `crates/patches/iced_widget` 同机制同哲学：
良性降级，不崩进程）。升级 iced/wgpu 时需重做/重审本补丁。

## 补丁内容（PLAN-098 T-03，DEBT-008 形态②根修）

### 1. uncaptured wgpu 错误非 panic 化（window/compositor.rs，device 创建处）

上游默认 uncaptured error handler 直接 panic。DEBT-008「进程静默退出」
形态②实录（preserve3/reader_vm_1794792834.log）：

```
wgpu error: Validation Error
  In Texture::create_view
    Texture with 'iced_wgpu.offscreen.source_texture' label is invalid
```

补丁语义：device 建立即安装非 panic handler（`[WGPU-DEGRADE]` 日志 +
eprintln），校验/设备错误只记日志、调用面自行降级——进程存活，MCP 面
继续服务。

### 2. `Renderer::screenshot` 零尺寸早退（lib.rs）

`iced::window::screenshot` 的 offscreen 路径对 0×0 viewport 裸
`create_texture` → 校验错误（PLAN-019 前科；仓内 `service_snapshot_
requests` 的记录值守卫与真实 surface 态有脱_sync 窗口）。补丁语义：
宽/高为 0 直接返回空 `Vec<u8>` + 日志。

### 3. `Renderer::screenshot` 等待有界化（lib.rs）

上游 `device.poll(Wait { timeout: None })` = 主线程可被无界 GPU 等待
阻塞（DEBT-008 模式①同族阻塞面）。补丁语义：2s 超时返回空帧（poll
Err 分支提前 return，不得 `get_mapped_range`——map 未完成会 panic；
未完成 map 的 buffer 随 drop 走 wgpu 正常清理）。

## 维护

- 升级 iced 时：diff 新版 iced_wgpu 与本目录三处补丁点，确认上游是否
  已修（若上游自带容错，删本补丁回归 crates.io 版）。
- 本补丁不改任何正常路径行为：handler 仅替换 panic 分支；零尺寸/超时
  分支上游本会崩/永等。
