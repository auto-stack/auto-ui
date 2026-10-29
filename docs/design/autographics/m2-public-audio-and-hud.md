# AG M2公共声音与HUD（规划v1，2026-10-30）

本设计定义拟新增公共服务的有限支持与接线边界。当前只有旧Breakout/Squash专用声音、字体与记分；没有本设计公开API/资源格式/验收已可用的声明。新Plans全为drafting，本轮不实施。旧canonical知识仍在auto-lang，新增实现与Plans归auto-ui；Gallery内容/Plans归ag-examples。

## 1. 分仓与依赖

| 引擎/内容合同 | 本期结果 | 硬前置 |
|---|---|---|
| [auto-ui/012 公共声音](../../plans/012-autographics-public-audio-and-accepted-cues.md) | 有界音频资源/cue、真实Auto请求、接受后一次播放、生命周期与消费 | 006实际delivery |
| [auto-ui/013 公共HUD](../../plans/013-autographics-public-hud-and-presentation-state.md) | 有限GPU文本HUD、Auto状态、画面/声音/HUD共同接受 | 012实际delivery |
| [ag-examples/004 G10b](../../../../ag-examples/docs/plans/004-autographics-gallery-node-animation-and-facing.md) | 原character四clip/朝向、G10双入口和单项包 | 006实际delivery、内容003实际交付 |
| [ag-examples/005 G12a](../../../../ag-examples/docs/plans/005-autographics-gallery-audio-actions.md) | 动作/有限cue声音示例与声音单项包 | 012实际delivery、内容004实际交付（共享目录串行） |
| [ag-examples/006 G12b](../../../../ag-examples/docs/plans/006-autographics-gallery-event-hud.md) | 0..3/Playing/Won/Failed、事件音效与HUD、完整G12包 | 013实际delivery、内容005实际交付 |

引擎006→012→013共享资产/assembly/Frame/prepare协议，实施串行；Gallery004可与引擎012并行，Gallery005可与013并行，各自只消费已交付pin。Gallery共享catalog/gallery.py/package变更按004→005→006落地，素材/合同准备可独立，运行验收不追开发tip。011仍固定004＋008，不自动借006聚合输入或后继服务改写其研究结论；完整游戏整链在新能力交付后另约。

## 2. 资源与有限支持

保持既有32credits、步数/时限、256KiB用户源/96KiB桥接/352KiB合并源、8资源身份/32MiB raw、64MiB CPU/64MiB GPU、8文件/32MiB项目assets磁盘及节点/实例/几何上限。音频编码字节/解码PCM、font原字节/字体与glyph atlas、服务队列与owner开销全部入同一账，不能为新格式另给免费容量。原profile/文档支持域显式保留，新格式只能被新版本识别。

### 声音

本期是非空间化one-shot SFX，不做音乐loop、流媒体、混音台/音轨编辑、3D距离衰减或录音。支持有限Ogg/Vorbis以及RIFF/WAVE PCM16（小端、mono/stereo），无任意codec透传。samplerate限22050/44100/48000Hz，channels1/2，单资源raw<=8MiB、总时长<=10s、解码float PCM<=16MiB，所有实际payload/解码暂存先预测再受限解码。header/packet/span/整型溢出/异常codebook与decoded frames/length必须前后检查，不能直接调用无界from_cursor后才发现越限。T-02若既有codec不能满足边界，给反例回new，不放开格式或静默抬内存。

每项目最多8个cue元数据，cue引用已接受音频内容身份与明确[start_frame,end_frame)范围；原始音频可只有一个声音bank，多个cue共享不可变PCM/编码owner。bank/音频文件仍按每个实际raw SHA计入资源8，cue并不新增免费raw。gain有限[0,1]，每tick最多4个命令、Run同时最多8个voice，容量不足是具名失败，不隐式抢占/丢音/无界排队。warm触发只建有限voice，不重解码/传图；无外部URI或实时网络加载。

选定Kenney jump/coin/land原Ogg可分别公开准入；为完整G3（4模型+共图）+声音+字体留预算，可制作一个不重采样/不丢帧、与原解码样本保持明确量化误差界的PCM16 bank与3cue。5模型/图身份+bank1+font1=7，仍在8内。转换身份/源SHA/采样与cue范围/许可/误差均必须实际记录，PCM16量化误差目标<=1/32768，源样本超幅不得静默削波，不把合并bank当已生成事实或省掉某个声音。011研究和最终游戏仍需真实组合验证。

### HUD

本期是每Run至多4个固定文本槽、每槽至多32个可见ASCII字符（总<=128），有限位置/anchor/颜色与显隐；不是生产AutoUI组件、交互按钮、IME/CJK、富文本或可采样深度正确的3D UI。HUD绘制到游戏图形surface的viewport内，2D/3D均为最后的GPU overlay，不随世界相机变换；letterbox/DPI/resize与clip规则明确，不覆盖编辑面板或把文字投进世界深度。

仅支持一个冻结版本Lilita One TTF（Kenney固定来源）与其完整OFL/RFN，raw<=1MiB，SHA白名单核验后再fontdue；不是任意TTF导入。font作为项目实际资源计入8/32MiB与64MiB CPU，固定ASCII32..126字形集合、字号16..48、单atlas边<=1024且texture<=4MiB（全部计入64MiB GPU）。最多4个被当前/history/pending/外部pin保留的(font,size,device代次)atlas key，全部预测计账；DPI不自动创建无界字号cache，T-02冻结字体栅格px与逻辑坐标/屏幕缩放的映射。glyph/布局/队列预测先于分配，缺字/超长/超域/非有限位置具名拒绝，不能以空格替代未知字形假绿。字体固定是本期支持域，完整字体系统/CJK另约。

## 3. 公共调用与32credit整链

聚合输入仍为held_mask+64*unread_pressed_mask：2D沿原action id0 Left/1 Right/2 Jump/3 Restart，bit4/5为0；3D沿006的0 Left/1 Right/2 Forward/3 Back/4 Jump/5 Restart。混读/重复消费与neutral/fresh Run沿已交付语义，不能为统一bit布局重排旧2D action含义。准确函数名、selector、schema/wire号在各Plan T-02核当时注册表唯一性后冻结，本设计不预占尚未发布编号。新profile分别显式组合2D sprites或3D node-animation链，并沿006已交付的有限输入快照语义；旧profile字节/价格/功能不改，也不能在旧2D profile偷偷提供新getter。

拟提供独立音效emit、HUD槽写入（每次1credit），同时提供**新profile独有的有界presentation bundle**：每callback最多一次、最多1个visual目标＋4个声音命令，013再版本化扩至4个HUD槽，整bundle显式定价1credit。固定有限表/字段/长度/排序由Host独立核验，零项也不无限扫；CPU/步数/源预算照常收费，不能借1credit塞无界数组。新定价只属于有限新API，不能降低旧visual/event/action getter价格。

完整代表性活动回调必须实测：1 snapshot+26 move+1 grounded+1 event_next+1 pair+1 phase+1 bundle=32；实际有事件并真正写visual/声音（013再写HUD）的分支不能省略。多事件/销毁/敌人完整关卡不由这一个和式宣称可行；此门只是本期必需公共组合，未来游戏仍需独立预算验证。若连此组合都不成立则回new，不改32、旧getter价格或用Rust选择动作。

一个callback不能同时用standalone emit与bundle写同类效果，不允许两次bundle或同一visual/槽重复声明；冲突/错类型/越限应拒整个候选。bundle visual仍带静态/动态kind、完整Run/slot/generation，遵守006/004原绑定与取帧/pose验证。HUD是完整有限槽状态或显式版本化delta，T-02只能选一套并写清缺席/清空与新Run默认，不能两套混用。

## 4. 音频接受、身份与失败

用户Auto决定何时发声、音效与HUD状态；Host只校验/渲染，不从grounded/score推断声音或胜负。声音event ordinal在Run内正整数严格递增，bundle内唯一有序；不同逻辑tick复用ordinal是拒绝，不会再发声。Host对同一已接受(Run,tick,bundle,ordinal)的传输/渲染重放幂等，不重排/补播；迟到旧Run结果拒绝。Run/tick/ordinal/cue/slot必须走精确整数通路，不经f32往返；分数/非有限/精度丢失拒绝，上界在T-02按实际typed wire冻结。ordinal耗尽受控失败，不回绕。

数据/资源/voice容量都在prepare检查；坏候选不发布visual/HUD/状态，也不送声音。同tick指逻辑接受的Run/tick一致，PCM从下一可用本服务mixer quantum起效，实际调度/硬件延迟界另报，不承诺GPU present与音频硬件同纳秒。完整接受后才把有界命令交真正mixer；绘制次数、重挂载和每次capture不能变成音效时钟。声学结果不可撤回：**接受之后设备断开/输出故障**具名audio Fault，停止该Run剩余voice/队列、保最后接受的视觉/HUD并禁止自动重试；不假称已经发出的样本可回滚。恢复只由Stop/新Run重新建立，不补播旧序列。

音频manager由Host显式拥有并可跨Run复用，Stop只撤销本Run voice/queue/控制权，Host最终关闭后释放manager；不能把正常Host owner当泄漏，也不能以共享manager为名保留旧Run可发声对象。Pause/Resume/Stop的实际输出控制以本服务下一mixer quantum为边界，已交硬件缓存单列。Pause冻结既有voice播放头，不推进Auto/新请求、不积压。neutral Step明确为静音调试：仍一次逻辑tick更新visual/HUD，但该tick新音效被标suppressed_by_step并消费其ordinal，Resume不补播；已有voice继续保持暂停。Resume只恢复已有合法voice剩余尾部。Stop/Fault/关闭/新Run撤销旧voice/队列和runtime owner；从下一本服务mixer quantum起不再产生该Run PCM，已交硬件缓存不能撤回，延迟边界必须报告。Editor/显式外部pin可合法保留静态资源，但不能保留会自动继续发声的旧Run。

## 5. 验收与来源边界

引擎声音验收须从真实Auto接受链观察**实际解码/混音PCM波形**与voice/owner，至少两个可辨识cue、gain/重放/暂停/Stop/旧Run/故障正反例；只数API或play调用不够。native输出正控验证本服务真实输出stream的PCM与backend状态，不录麦克风、不捕获系统其他应用声音；设备不可用必须具名失败/明确未覆盖，不可用mock健康冒充。离线mixer与本服务native输出分层记录，声压/物理扬声器/驱动内部缓存不在可证明范围。

HUD用新鲜GPU像素/独立字形或固定样本文本oracle，0/1/2/3、Playing/Won/Failed、漏字/旧截图/错局/位置/clip/resize与同tick数据具判别性。两者都要求preaccept失败全last-good、postaccept音频故障单列、source/consumer/SDK/锁与实际包绑定。OS键鼠/置顶/抢焦点自动化继续暂停，真实GPU/音频设备/重型门机器单实例，原始PCM/PNG/log只本地保留。

异步音频线程的解码/混音CPU与VM/Host/publish时间分别记账，记录每quantum样本数/耗时/实时deadline或underrun；不能把异步开销从性能报告里隐藏，8ms pump目标与音频deadline分列。

新服务尚未实现，所有预计字节/价格/组合仅是合同约束；可行性先于依赖实现。任何必须格式/输入/预算或支持范围改变都带反例回new，不默改AC。Gallery只消费实际delivery，不修引擎、不复用上游绿报告当本项证据。最终G10/G12与首12项全部子入口Ready仍不替代完整游戏、统一引擎矩阵/发行与全AG里程碑。
