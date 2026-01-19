## AutoUI

AutoUI 是基于Auto语言的UI库


## 目标

长远目标：用Auto语言来描述所有UI界面。

这需要可以把Auto语言描述的界面翻译成以下UI框架：

1. gpui
2. iced
3. vue.js
4. 鸿蒙UI
5. Jetpack Compose
6. LVGL

这样的话，基本可以覆盖PC，WEB，Android，鸿蒙和MCU。

短期目标：用Auto语言作为描述层，用gpui/iced作为实现层，实现完整的桌面段跨平台UI库。

## 为什么要选择gpui和iced

AutoUI最初选择用gpui，但gpui当时的实现并不完善，
更改非常频繁，
且gpui的消息反馈机制和Auto语言的设计并不完全兼容，
适配起来比较麻烦，因此暂停了。参见`gpui2`和`gpui3`两个未完成的分支。

后来我发现iced的设计更简洁，且相对稳定，因此打算以iced为基础，重新实现AutoUI

## 为什么要重做

这个项目因为其他项目的原因暂停了很久，现在重新开始时，许多预设条件都变化了。

最大的变化，是现在有AI编程了。

因此我考虑把AutoUI做成与底层无关的独立层，让它可以切换底层实现。
考虑到之前的经历，最好的选择就是让AutoUI同时支持iced和gpui。

AutoUI本身的设计接近于iced，即以类似ELM的消息通讯为基础实现UI的行为。

因此，我们应当先确保iced基底的实现。


