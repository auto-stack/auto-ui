# PLAN-006 T-02：原件与真实Auto/Host/GPU最小整链

2026-10-30；唯一树 `D:/autostack/.wt/ui-006/auto-ui`，分支 `codex/plan-006-dev`，parent c7dceef；全链授权保持，前轮已交付核心与producer属有效进展。本轮未改SDK/两锁/其他计划WIP。

## 实现与固定合同

- `assets::import_animated`与Host公开`import-animated`明确选择新支持域；旧import仍拒动画。closure新增`node_animation`（false默认省略）；true以`autographics.node_animation3d/1`域生成id并完整绑定原GLB/外图，schema6保存和冷重开按该标记重建clip。不改变原文件或005旧closure字节/身份。
- `runtime_node_animation`冻结模型target的实际内容身份，128静态target/8槽；6值wire（4int+2float）逐域复核Run/tick、asset/node/clip、phase/loop/yaw、live模板/generation、free零行；合法后采样原始节点并纳入共同Frame。最后矩阵同时检finite/可逆/正det，overflow不可进入任一GPU pass。
- Host prepare使用`entity world * visual yaw * sampled node world * inverse(mesh base world)`，复用005原geometry/UV/texture/节点关系。原角色6mesh×静态/实例独立phase/clip/yaw；相机、位置、视觉、对象和事件同次prepare/accept/publish，颜色/shadow共用实例矩阵。旧Play profile对新动画payload明确unsupported，防止静默忽略clip。
- 实际payload/scratch纳入原64MiB CPU union；Run给每静态target和8个潜在槽预留last-good/candidate/prepare多份pose空间，隐藏不减分母。基础probe内mesh上传/解码零增量，未逐tick重解码或传mesh；全面owner/20轮仍归T-05。

## 实际检查

`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --test public_node_animation -- --test-threads=1 --nocapture`，exit0，4/4（`t02-closure-final.log` SHA256 d560afc0456d28b3496f670bb5f4a1fd6b6ce7d0ddd2cc1db5e5aab90e21784f）。后续最终矩阵与旧profile拒绝只影响Host case，定向重跑最终代码`... --test public_node_animation original_textured_character_auto_host_same_frame_pose_and_gpu_pixels -- --test-threads=1 --nocapture` exit0，1/1；`t02-final-profile.log` SHA256 2d0085e066ffe57fa7911091f422b19c5f9d1db5c4dd6ae2bb4254a0db37ffc4。

Host case实际执行公开导入→配置原既有角色0.8×1×0.8碰撞AABB（仅场景scale，GLB零转换）→保存→目录搬迁→空Registry冷重开→真实Auto app_start/paused→一fixed tick：相机relative运动、垂直位移、grounded、事件一次读取、Auto选择jump、静态及实例一次visual setter→Host采样→同tick publish→实际GPU。输入GLB SHA `7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1`不变，4clip均在重新加载payload中。

结果：tick1，static clip3/jump、phase≈0.1、yaw≈0.5；实例clip1/idle独立。Auto phase按fixed dt*6明确推进，不用wall clock。位置、camera tick与visual tick一致。CPU tick总耗时3836us，VM输出/采样/prepare走原callback及整tick限额，无超额或错帧；原100000steps/120resumes/250ms callback、2s整tick未放宽。受管源79567B（此前producer证据保持）、32credits共享负控保持。资源union `(raw identities=2, raw bytes=97732, CPU=1102359, GPU=1101700)`，均低于原域，animation Asset自身cpu=53783B。

实际像素start `057c18d3ecde1a3e16d9dffee57756e2ad9340ac2a5916768d5651779c7550d0` → tick1 `4fa6d3de7f26ee5d688ebf3a657a7212086d11a65600a8886b0591a58d178788`。另在同一frame固定相机/位置/yaw/材质/mesh，只从真实MeshInstance矩阵擦除node delta，标dirty重绘后GPU像素必须不同；该负控通过，不能只凭移动或相机变化冒充动画。19项wire变异包含status/count/Run/tick/node/asset/clip/loop/负phase/NaN/错type/Inf/越限yaw/旧generation/错slot asset/free非零/短block均拒；合法wire重新admit的worlds等于实际frame。最终entity×pose溢出也被拒。

共享解码/保存路径旧005 `cached_closure_open_ignores_display_name_but_preserves_content_binding` exit0，1/1（`t02-legacy-closure.log` SHA256 8c9d9823d8a6d00d72f6c253441b788c1f45f1773e199350f76c9994ea5cd3c8）。两包相关fmt与git diff检查干净；新产品无compiler warning，冻结SDK既有146warnings未改。

日志都在 `D:/autostack/plan-artifacts/auto-ui/plan-006/`，不提交原日志。早期探针错误保留：Rust测试String/private字段接线错误修正；原件碰撞形状未按既有character域配置引发APIError24，按真实bbox设置场景scale而非改支持域；首次擦除轨道未dirty/重绘只读旧GPU纹理，修正为标dirty实际重绘后负控通过。没有降低oracle或验收。

## handoff

T-02关闭，2/10。T-03补齐生产准入广泛正反例/总额先检后分配/保存故障；T-04原四clip GPU轮廓/独立数值、朝向与物理/阴影；T-05全事务/暂停/重启/20轮owner；T-06 fresh consumer/identity/性能/兼容；独立review与merge均pending。没有用户待答事项，未将这些后续要求算作已完成。
