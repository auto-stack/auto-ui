# AutoUI 书籍阅读组件

`autodown` 的 `reading_mode` 默认关闭，常规 Markdown 走现有渲染和流式路径。开启后，`content` 传入 `autodown-reading/1` 书籍章源稿。Web 使用 `@autodown/engine`，原生 VM 需开启 `autodown` feature 并使用同版 `autodown-core`。

| 属性 | 作用 |
|---|---|
| `reading_chapter_id` | 可选的章 ID，优先由传入清单确定 |
| `reading_card_id` | 为空时通读整章，有值时投影为该卡的题目和正文 |
| `reading_target_id` | 目标段、句群或标题的持久 ID。滚动到当前投影中覆盖目标的块 |
| `onbookselect` | 选区消息，接收一个 JSON 字符串 |

选区字段为 `chapterId`、`cardId`、`paragraphId`、`targetId`、`text`、`before`、`after`。选区限于单个有锚的自然段，卡 ID 和目标 ID 分开传递。句群内选区返回句群 ID，片段越过句群时返回段 ID。前后文取自完整自然段。绑定空目标后可重新设置同一目标来再次定位。卡视图回到整章时保留目标 ID，即可回到同一自然段。

原生 VM 使用现有滚动控制器定位实际排版后的段落，富文本拖选读取同一字形布局的命中位置。浏览器使用当前组件范围内的 DOM 选区。源文本、卡片投影和持久目标以 AutoDown 仓的 `docs/specs/book-reading.md` 为准。
