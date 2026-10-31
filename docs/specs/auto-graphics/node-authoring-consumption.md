# AutoGraphics Node 作者产物消费与来源身份（auto-ui PLAN-019 SD-02）

Status: delivered by auto-ui PLAN-019 r1。本文件描述新迁移入口显式选择的
版本化 source-build 消费合同；旧 exact-binary 锁保持严格且不被弱化，
新模式不构成失败后的自动 fallback。

## 0. 与旧锁的关系

016/017（及更早）交付的已认证 runner 沿原 exact-binary 锁：身份比对仍
严格拒绝不匹配二进制。本合同只为 **明确选择新作者入口的迁移内容**
（PLAN-019 scene-2d/character-2d/camera-2d profile 的产物）启用固定
source-build 模式。不引入 PKI/通用安装器/远端发布平台；不为历史 exe
缺失保永久计划证据。

## 1. 产物与身份链

公共准备输出（每入口一目录）：

- `scene.agscene` —— `autographics.scene/6` 工程；脚本 profile 的
  `behavior.api` 为受信 assembly 版本串（`autographics.character/1` /
  `autographics.runtime_camera2d/1`），`behavior.source` 绑定生成用户源
  的 sha256 与 byte_length；scene-2d 的 `behavior` 为 null（纯场景
  play，无 VM）。
- `world.at` —— 仅脚本 profile；生成用户源（sha 即场景绑定值）。
- `identity.json` —— `generator`（计划/profile 标识）、`api`、
  `user_source_sha256/bytes`、逐模块 sha、`closure` 清单（name/sha256/
  bytes）、`roles` 具名映射（character/solids/camera/scene_objects→
  场景 id）。
- `closure/<module>.at` —— 输入闭包副本：消费与冷再生的唯一输入，
  B 端不借生产者源树。

身份链逐环可核：closure 文件 sha == identity 清单 == 夹具源；world sha
== identity `user_source_sha256` == 场景 `behavior.source.sha256`；
任何一环不匹配即拒绝（篡改检出实测）。

## 2. 版本化 source-build 记录

`node2d-lower` 产物目录 + `tools/verify_plan019.py --build-record <out>
--generator <bin>` 产生记录：

- `generator.sha256/bytes` —— 实际生成器二进制（绑定正在使用的 exe，
  非自填字符串）；
- `toolchain` —— rustc 版本、平台；
- `workspace_lock` —— 生成器 workspace 的 Cargo.lock sha；
- `products` —— 全部产品 scene/world/identity 的 sha。

消费者 `--verify-record` 依次核 generator sha、Cargo 锁 sha、产品
hash；任一漂移拒绝（锁漂移负控实测在产品比较前拒绝）。运行侧由内容仓
（ag-examples 008）按同一合同补 runner 二进制身份与运行报告绑定。

## 3. 冷 A→B 闭合（实测）

A（生产者）生成产品 + closure/；B 仅持 closure/ 副本与 profile 选择，
从闭包模块再生：scene/world/identity 全部与 A 冻结产物字节等价（13
产品、四组冷 A→B 实测）。篡改任一闭包模块 → manifest sha 捕获；精确
还原后同一入口立即恢复可用（实测）。坏件（半写/错 sha/过期 record）
不得混为有效产物。

## 4. 变体与失效

变体是声明入口（独立模块文件）；任何被用模块/资源/参数变化使旧 Ready
失效——identity 与场景绑定随之改变，旧绑定不匹配新源即拒绝再生等价。
再生命令：`node2d-lower --profile <p> --entry <closure-copy>/main.at
--out <dir>`（与生成时同 profile；错 profile 拒绝）。

## 5. provenance

本文件由 auto-ui PLAN-019 r1 建立并验证（收据见计划归档）；实际行为
描述沿 [node2d-basic-authoring](node2d-basic-authoring.md) 与计划 §9
T-02 冻结合同。历史锁与旧回执保持原状，不在本文件重述。
