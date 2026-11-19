# 087–093 计划执行准备度复核

日期：2026-11-14。范围：七份drafting计划r1→r2、共同执行/视觉指南、GPT参考图、路线与索引。源码观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。这是同一编写会话对**计划合同**的再次检查，不是实施后独立最终review，也未运行拟议产品测试。

结论：r1已经说明目标/非目标、复用来源、八任务和验收方向，但不足以让执行者不经重新设计就进入主要编码。r2补了公开参数/事件、数据样例、状态/反馈、测试前置与视觉规格。七项均可从T-01启动；T-02及集成任务仍需满足各自机械可行性与依赖pin，不能把规划通过当作能力已交。

## 1. 发现与修订

| ID / 级别 | 影响任务 / AC | r1缺口与依据 | r2处理及关闭验证 |
|---|---|---|---|
| F-01 / P2 | 全部T-01…03，AC-01…04 | 多数props/events形状和owner留给T-01，跨Plan消费可能出现同名不同负载 | 七份§5均补接口表、JSON样例与状态规则；共用指南§4区分请求/观察/业务成功。T-01从重新设计改为验证现有机制能承接 |
| F-02 / P2 | 087 T-01/02/05，AC-02/03 | 数值支持范围、先量化还是先比较范围、基准单位/步进未确定 | 固定9整数/6小数输入profile、precision0…6、舍入/范围顺序、单位表与基准步进；V-06包含1.004/max1及边界反例 |
| F-03 / P2 | 088 T-02/03/05，AC-02/03 | HSV灰黑、通道舍入、隐藏alpha、Picker与Dialog的commit层次可产生不同实现 | 固定颜色类型/坐标/舍入/灰黑色相、子提交不等于Dialog确认；V-06给独立RGBA已知值、取消/一次确认 |
| F-04 / P2 | 089 T-01…04，AC-02/03/04 | 受控/本地两种owner未选定，换图/变换多字段更新可能产生中间帧 | v1仅受控，完整view tuple事件、资源身份与父原子发布规则；V-06换图与旧事件控制；实际像素仍由V-01验证 |
| F-05 / P1 | 090 T-01/02/05，AC-02/03 | `mpv/contract.rs`按last_seek_target数值去重；直接照搬会把新一次同目标seek忽略。Cargo.toml又为mpv目标设required-features，原命令不齐 | 引入公开seekRevision区分新请求与重绘，标明None清理分支可探测复用；修命令带mpv-gpu及真实mpv_engine；V-06要求两次定位10秒都生效。此为源码推导的计划反例，未声称实跑复现 |
| F-06 / P1 | 091 T-01…05，AC-02/03/04 | 只有pending没有请求/反馈身份，父props回灌前双击与取消后迟到结果缺闭环 | 固定querySeq/resultQueryId、session/seq及confirmation反馈；同步本地防重、拒绝后重试、关闭失效；V-06对应实际provider与普通控件反例 |
| F-07 / P2 | 092 T-01…05，AC-02/03/04 | schema/values类型、false/0默认值和业务fieldErrors如何绑定不具体 | 明确字段/value record、kind映射、hasDefault、目标/schema版本、错误请求身份；V-06覆盖false/0恢复和A→B旧错误 |
| F-08 / P1 | 093 T-01…05，AC-01/02/04 | 数据树/slot绑定欠具体；`aura/types.rs::slot_element_name`只接字面量/Ident，动态字段name不是已交能力；只画静态分栏无法满足计划 | 固定layout节点表/权重/命令/完整候选与内容slot规则，记录最小UI适配范围及SD-03；V-06验证实际动态内容/ordinary/tracked/Vue及文档/undo保持；机械可行性仍是T-01前置 |
| F-09 / P2 | 全部T-03/05/06，AC-06 | 样式只有泛化文字，执行者可能各造密度、无错误/窄窗参考 | 新执行视觉指南含GPT图、尺寸、状态/窄窗规则及图中示意边界；七份V-06/AC-06挂入真实截图验收，生成图不充当产品证据 |
| F-10 / P2 | 全部T-01/05/06，AC-05/06 | 新脚本只列拟议名，复用的旧runner可能带FileDialog专用就绪，旧030测试绕过autoplay/用户磁盘 | 指南§5写T-01决策产物、旧Session专用限制、真实Vue/媒体前置、输出/退出规则；保留拟议/已执行区分，不写假测试通过 |

P1表示未经修订可能导致错误交付或验收漏跑，并非宣称已验证产品bug；修订关闭的是计划歧义。底层问题需实施时以真实反例验证并修复。

## 2. 逐计划交接

| Plan | 当前合同 | 下一可执行任务 | 仍需实际验证 / 硬依赖 |
|---|---|---|---|
| [087](../../plans/087-numeric-and-unit-fields.md) | r2，drafting，0/8 | T-01数值/profile与消息整链探针 | 两端定点与中间运算边界、真实runner |
| [088](../../plans/088-color-picker-and-dialog.md) | r2，drafting，0/8 | T-01动态色面/已知颜色/现有Paint适配 | T-03依087；不能用任意class/贴图冒充动态色面 |
| [089](../../plans/archived/089-image-viewport-and-preview-dialog.md) | r2，drafting，0/8 | T-01ImageSurface负载/受控tuple与共享资源 | 真实几何/像素、资源释放与加载回调，不依赖新解码器 |
| [090](../../plans/090-media-player-and-transport-controls.md) | r2，drafting，0/8 | T-01同目标seek/后端/源身份探针 | 真libmpv/音视频fixture/浏览器激活；自跳过不算解码 |
| [091](../../plans/091-asset-browser-and-picker.md) | r2，drafting，0/8 | T-01provider查询与确认反馈小闭环 | T-03依085/086/089；旧请求/权限变化由真实消费验证 |
| [092](../../plans/092-property-inspector.md) | r2，drafting，0/8 | T-01类型分支/消息/设置预览和Storage | T-03依087/088；不新增任意反射或脚本 |
| [093](../../plans/093-dock-layout-and-panels.md) | r2，drafting，0/8 | T-01动态slot/内容身份/resize整链 | 风险最高；T-04依084/085，不能越过slot/文档保持门 |

## 3. 视觉资产与边界

[执行与视觉指南](../../design/autoui/common-components-execution-and-visual-guide.md)引用[GPT参考图](../../design/autoui/assets/common-components-087-093-reference-v1.png)及[完整提示](../../design/autoui/assets/common-components-087-093-reference-v1-prompt.md)。使用内置生成工具，第一次网络失败后重试成功；针对字号错误值与暂停状态做一次定向编辑，最终1585×992 RGB、SHA256 `F4DA10C65FDC266F7ED574E9351A65653B66FDA631926D71AEAAF391FCCA41C1`。

人工检查：七个编号均有独立区域，颜色/RGBA、单选网格、图片工具栏、播放状态、字段分组和同窗停靠结构可读；字号128配8…72错误，暂停配Play按钮。它提供风格而非产品实现，缺少的hover/pending/错误等状态由指南文字与真实验收补齐。深色/窄窗/实际CJK字体仍须实施后拍摄，未宣称参考图已证明这些轨通过。

## 4. 本轮验证与最终状态

本轮只核验文档：编号/章节/metadata、task/AC/SD/Cases覆盖、JSON样例、依赖无环、路径/链接、图像读取和文件摘要、变更空白/冲突标记。结果和文件身份见[manifest.json](manifest.json)。现有及新建的产品测试均未运行，canonical Specs和组件源码未在本轮修改。

stage: new；Plans 087…093 r2；outcome: pass（可交给执行者从T-01开始，受各自前置约束）；next: work。后续T-07仍由独立上下文在同一实施树完成，不能引用本报告代替实现验收。
