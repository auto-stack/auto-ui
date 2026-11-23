# 094–097执行准备度复核（2026-11-14）

结论：四份Plan由drafting r1修为**drafting r2、0/8**，当前合同足以交给执行agent从T-01进入。T-01是有边界的机制验证，不能越过它把包语法、实例身份、首次error、焦点/IME或MCP接入当作已可用。本报告是同一会话的计划静态复核，不是独立实现review，也未运行组件产品门。

复核基线：auto-ui HEAD `3cab09943079a1d1fca993fa5a4c4dd2a9fa8315`。核对当前Specs、正式包/工具源码和实际消费者；原027/028/015源摘要不变。084的交付提交`7ea71f0c`及归档8/8已核；086交付`765251b9`。未修改产品代码、canonical Spec、兄弟仓或其他任务进度。

| finding | 原合同缺口 / 风险 | r2关闭方式及验收归属 |
|---|---|---|
| R-01 | seq/session仅实例内递增，卸载重建可重号；同revision的真实内容变化未写清 | 四Plan §5增加不复用instanceToken和消费时身份校验；X-04/X-05、AC-03，配当前合法动作正例 |
| R-02 | Prompt取消可重复，成功revision更新可能先解锁；Tag仅凭值/revision确认会误认外部更新 | Prompt取消/成功待关闭锁、retryable拒绝及按钮文案props；Tag显式accepted与tags/revision同快照确认、捕获真实noteId；P-06/G-06、AC-02/03/05 |
| R-03 | Task取消待确认状态仅本地持有，重建后可能重复请求；终态与迟到拒绝竞态不明确 | cancelPending归任务owner，单run终态不可回退；真实任务自然完成优先；J-06、AC-02/03/05 |
| R-04 | Command分组可能重新聚合provided结果，破坏Launcher原全局评分；pending交互不完全确定 | provided保留全局次序并按连续组显示；local折叠两侧文本；冻结pending搜索/执行/用户关闭；保留028环绕/grid/recent；C-06、AC-02/03/05 |
| R-05 | 借用Dialog和声明error不等于实际可用；现有Pagination已证native子组件Init emit不派发 | 明确复用modal.rs、ui.focus/host.rs与已交包；T-01先证明首次配置错误正常调度一次通知和焦点，必要最小接缝由本Plan负责；X-04/05，AC-01/02/04 |
| R-06 | Playwright、native MCP、整串type、IME、系统provider可能被当作同一类证据；缺桥可能引发四份重复建设 | native正常autoui工具、Vue真实浏览器MCP与GUI分层记录，复用现成提供方；composition/原生输入桥独立断言；明确acceptance handler不能替代产品门；X-05/06/07，AC-04/06 |
| R-07 | runner退出码、部分启动失败的清理、平台缺口的聚合规则不够具体；Linux路线过时 | report.json/0-pass、1-fail、2-not_run、有限超时、finally清理和App重启；Linux沿用户在084指定的自制发行版+VMware，不用WSL冒充；X-02/06，AC-04/06/07 |
| R-08 | strategy/索引仍把084记为草案；Q-01仍写原因未知 | strategy r6、视觉指南r3及索引对账084交付；Q-01引用DEBT-006公共渲染器归因且保持未关闭；下一项推荐085 |

四份当前合同：

- [094 PromptDialog](../../plans/archived/094-prompt-dialog.md)：027重命名/新建和普通名称集合；fs操作/权限/验证归应用，Vue不增加磁盘写权限。
- [095 TaskProgressDialog](../../plans/095-task-progress-dialog.md)：组件只显示应用任务状态并请求取消；普通记录校验是首个真实消费者，Reader仅只读参考，不宣称产品已迁移。
- [096 CommandPalette](../../plans/archived/096-command-palette.md)：028保留排序/分类/recent/宿主总线，组件持交互；另有跨端普通设置消费者。
- [097 TagInput](../../plans/archived/097-tag-input.md)：015保留update_tags/vocab/正文，组件持草稿/候选；异步回包绑定真实noteId，不以active_id重定位写入。

参考图保持单一仓库源：[PNG](../../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1.png)、[提示](../../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1-prompt.md)、[指南](../../design/autoui/file-navigation-and-interaction-visual-guide.md)。1586×992 RGB，SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13`。四Plan分别引用左上/右上/左下/右下，字节未变，未重新生图；真实视觉仍需浅深色、中文/长文案、420×560和适用错误/pending截图，不能以图/MCP文本替代。

静态检查范围：四Plan的0…10章节、r2元数据、唯一编号、8项分阶段任务/8项AC/2项SD、13行具名case、JSON样例、任务与case关联、当前新增/修改链接及PNG/来源摘要；不把新增测试脚本和未来机制写成已运行。计划和设计索引中既有无关历史链接不在本次修复范围。

下一项建议[085 TreeView](../../plans/085-official-tree-view-component.md)：第一批唯一尚未实施的核心组件；已有018/026/027/041四份局部实现，无未交付组件硬依赖，并解除091的树前置。先重核当前084/086后的027/041及blueprint解析闭包，复用业务边界不变。若只选本次四项则优先094；之后097，再按实际消费者需求选择096/095。Linux/DEBT-006影响最终验收，但不是跳过T-01或重开084归档的理由。

