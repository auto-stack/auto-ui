use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::input::TextInput;
use autogui::widget::toolbar::*;
use autogui::widget::util::*;
use autogui::widget::button::Button;
use autogui::widget::workspace::Workspace;
use autogui::widget::table::Table;
use autogui::widget::table::{ColConfig, WidthMode, Format, Align, Row};
use autoval::Value;
use autoval::Grid;
use gpui::*;

struct RootView {
    workspace: View<Workspace>,
}

struct CenterContent {
    table: View<Table>,
}

impl Render for CenterContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            // .w_3_4()
            .p_5()
            .h_full()
            // .h(px(500.0))
            // .max_h(px(640.))
            .gap_4()
            .child(self.table.clone())
            // .child(Button::primary("History").on_click(cx.listener(|this, ev, cx| {
                // let history = this.table.read(cx).get_update_history_as_string();
                // println!("{}", &history);
            // })))
    }
}

impl RootView {
    fn new(cx: &mut ViewContext<Self>) -> Self {

        let col_config = vec![
            ColConfig{
                idx: 0,
                id: "id".into(),
                title: "ID".into(),
                width: WidthMode::Pixels(100.),
                align: Align::Start,
                format: Format::Hex,
                options: vec![],
            },
            ColConfig{
                idx: 1,
                id: "name".into(),
                title: "Name".into(),
                width: WidthMode::Pixels(250.),
                align: Align::Start,
                format: Format::Text,
                options: vec![],
            },
            ColConfig {
                idx: 2,
                id: "callback".into(),
                title: "Callback".into(),
                width: WidthMode::Pixels(80.),
                align: Align::Start,
                format: Format::checkbox(),
                options: vec![],
            },
            ColConfig{
                idx: 3,
                id: "desc".into(),
                title: "Desc".into(),
                width: WidthMode::Auto,
                align: Align::Start,
                format: Format::Text,
                options: vec![],
            },
        ];

        let data = vec![
            vec![Value::Int(0x10), Value::Str("DiagnosticSessionControl".to_string()), Value::Bool(false), Value::Str("诊断会话控制".to_string())],
            vec![Value::Int(0x14), Value::Str("ClearDiagnosticInformation".to_string()), Value::Bool(true), Value::Str("清除诊断信息".to_string())],
            vec![Value::Int(0x19), Value::Str("ReadDTCInformation".to_string()), Value::Bool(false), Value::Str("读取DTC信息".to_string())],
            vec![Value::Int(0x22), Value::Str("ReadDataByIdentifier".to_string()), Value::Bool(true), Value::Str("读取数据".to_string())],
            vec![Value::Int(0x23), Value::Str("ReadMemoryByAddress".to_string()), Value::Bool(false), Value::Str("读取内存".to_string())],
            vec![Value::Int(0x27), Value::Str("SecurityAccess".to_string()), Value::Bool(false), Value::Str("安全访问".to_string())],
            vec![Value::Int(0x28), Value::Str("CommunicationControl".to_string()), Value::Bool(false), Value::Str("通信控制".to_string())],
            vec![Value::Int(0x2A), Value::Str("ReadDataByPeriodicIdentifier".to_string()), Value::Bool(true), Value::Str("读取数据（周期标识符）".to_string())],
            vec![Value::Int(0x2C), Value::Str("DynamicallyDefineDataIdentifier".to_string()), Value::Bool(false), Value::Str("动态定义数据标识符动态定义数据标识符".to_string())],
            vec![Value::Int(0x2E), Value::Str("WriteDataByIdentifier".to_string()), Value::Bool(true), Value::Str("写入数据".to_string())],
            vec![Value::Int(0x2F), Value::Str("InputOutputControlByIdentifier".to_string()), Value::Bool(false), Value::Str("输入输出控制".to_string())],
            vec![Value::Int(0x31), Value::Str("RoutineControl".to_string()), Value::Bool(true), Value::Str("例程控制".to_string())],
            vec![Value::Int(0x3D), Value::Str("WriteMemoryByAddress".to_string()), Value::Bool(true), Value::Str("写入内存".to_string())],
            vec![Value::Int(0x3E), Value::Str("TesterPresent".to_string()), Value::Bool(true), Value::Str("诊断设备在线".to_string())],
            vec![Value::Int(0x85), Value::Str("ControlDTCSetting".to_string()), Value::Bool(true), Value::Str("控制DTC设置".to_string())],
        ];

        let center = cx.new_view(|cx| CenterContent {
            table: cx.new_view(|cx| Table::new(cx, "my_table".to_string(), col_config, data, vec![])),
        });

        Self {
            workspace: cx.new_view(|cx| Workspace::new(cx).child(center)),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.workspace.clone())
    }
}

fn main() {
    SimpleApp::new().run(false, |cx| cx.new_view(|cx| RootView::new(cx)));
}
