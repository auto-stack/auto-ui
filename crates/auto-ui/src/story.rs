mod assets;
mod title_bar;

pub use assets::Assets;
pub use title_bar::AppTitleBar;

use gpui::{
    actions, div, impl_internal_actions, prelude::FluentBuilder as _, px, size, AnyElement,
    AnyView, App, AppContext, AsyncApp, Bounds, Context, Div, Entity, EventEmitter, Focusable,
    Global, Hsla, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled as _, Window, WindowBounds, WindowKind, WindowOptions,
};

use gpui_component::{
    button::Button,
    divider::Divider,
    dock::{register_panel, Panel, PanelControl, PanelEvent, PanelInfo, PanelState, TitleStyle},
    h_flex,
    label::Label,
    notification::Notification,
    popup_menu::PopupMenu,
    scroll::ScrollbarShow,
    v_flex, ActiveTheme, ContextModal, IconName, Root, TitleBar,
};

use reqwest_client::ReqwestClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SelectScrollbarShow(ScrollbarShow);

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SelectLocale(SharedString);

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SelectFont(usize);

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SelectRadius(usize);

impl_internal_actions!(
    story,
    [SelectLocale, SelectFont, SelectRadius, SelectScrollbarShow,]
);

actions!(story, [Quit, Open, CloseWindow]);

const PANEL_NAME: &str = "StoryContainer";

pub struct AppState {
    pub invisible_panels: Entity<Vec<SharedString>>,
}
impl AppState {
    fn init(cx: &mut App) {
        let state = Self {
            invisible_panels: cx.new(|_| Vec::new()),
        };
        cx.set_global::<AppState>(state);
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

pub fn create_new_window<F, E>(title: &str, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    create_new_window_sized(title, crate_view_fn, cx, 1600, 1200);
}

pub fn create_new_window_sized<F, E>(
    title: &str,
    crate_view_fn: F,
    cx: &mut App,
    width: u32,
    height: u32,
) where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    let mut window_size = size(px(width as f32), px(height as f32));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let window_bounds = Bounds::centered(None, window_size, cx);
    let title = SharedString::from(title.to_string());

    cx.spawn(async move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(640.),
                height: px(480.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            ..Default::default()
        };

        let window = cx
            .open_window(options, |window, cx| {
                let view = crate_view_fn(window, cx);
                let root = cx.new(|cx| StoryRoot::new(title.clone(), view, window, cx));
                cx.new(|cx| Root::new(root.into(), window, cx))
            })
            .expect("failed to open window");

        window
            .update(cx, |_, window, _| {
                window.activate_window();
                window.set_window_title(&title);
            })
            .expect("failed to update window");

        Ok::<_, anyhow::Error>(())
    })
    .detach();
}

struct StoryRoot {
    title_bar: Entity<AppTitleBar>,
    view: AnyView,
}

impl StoryRoot {
    pub fn new(
        title: impl Into<SharedString>,
        view: impl Into<AnyView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title_bar = cx.new(|cx| AppTitleBar::new(title, window, cx));
        Self {
            title_bar,
            view: view.into(),
        }
    }
}

impl Render for StoryRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let drawer_layer = Root::render_drawer_layer(window, cx);
        let modal_layer = Root::render_modal_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .size_full()
            .child(
                v_flex()
                    .size_full()
                    .child(self.title_bar.clone())
                    .child(div().flex_1().overflow_hidden().child(self.view.clone())),
            )
            .children(drawer_layer)
            .children(modal_layer)
            .child(div().absolute().top_8().children(notification_layer))
    }
}

impl Global for AppState {}

pub fn init(cx: &mut App) {
    gpui_component::init(cx);
    AppState::init(cx);

    let http_client =
        std::sync::Arc::new(reqwest_client::ReqwestClient::user_agent("auto-ui/story").unwrap());
    cx.set_http_client(http_client);

    register_panel(cx, PANEL_NAME, |_, _, info, window, cx| {
        let story_state = match info {
            PanelInfo::Panel(value) => StoryState::from_value(value.clone()),
            _ => {
                unreachable!("Invalid PanelInfo: {:?}", info)
            }
        };

        let view = cx.new(|cx| {
            let (title, description, closable, zoomable, story) = story_state.to_story(window, cx);
            let mut container =
                StoryContainer::new(window, cx).story(story, story_state.story_klass);

            cx.on_focus_in(
                &container.focus_handle,
                window,
                |this: &mut StoryContainer, _, _| {
                    println!("StoryContainer focus in: {}", this.name);
                },
            )
            .detach();

            container.name = title.into();
            container.description = description.into();
            container.closable = closable;
            container.zoomable = zoomable;
            container
        });
        Box::new(view)
    });
}

actions!(story, [ShowPanelInfo]);

pub fn section(title: impl IntoElement, cx: &App) -> Div {
    use gpui_component::ActiveTheme;
    let theme = cx.theme();

    h_flex()
        .items_center()
        .gap_4()
        .p_4()
        .w_full()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(theme.border)
        .flex_wrap()
        .justify_around()
        .child(div().flex_none().w_full().child(title))
}

pub struct StoryContainer {
    focus_handle: gpui::FocusHandle,
    name: SharedString,
    title_bg: Option<Hsla>,
    description: SharedString,
    width: Option<gpui::Pixels>,
    height: Option<gpui::Pixels>,
    story: Option<AnyView>,
    story_klass: Option<SharedString>,
    closable: bool,
    zoomable: Option<PanelControl>,
}

#[derive(Debug)]
pub enum ContainerEvent {
    Close,
}

pub trait Story: Focusable + Render {
    fn klass() -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn title() -> &'static str;
    fn description() -> &'static str {
        ""
    }
    fn closable() -> bool {
        true
    }
    fn zoomable() -> Option<PanelControl> {
        Some(PanelControl::default())
    }
    fn title_bg() -> Option<Hsla> {
        None
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable>;
}

impl EventEmitter<ContainerEvent> for StoryContainer {}

impl StoryContainer {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            name: "".into(),
            title_bg: None,
            description: "".into(),
            width: None,
            height: None,
            story: None,
            story_klass: None,
            closable: true,
            zoomable: Some(PanelControl::default()),
        }
    }

    pub fn panel<S: Story>(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let name = S::title();
        let description = S::description();
        let story = S::new_view(window, cx);
        let story_klass = S::klass();
        let focus_handle = story.focus_handle(cx);

        let view = cx.new(|cx| {
            let mut story = Self::new(window, cx).story(story.into(), story_klass);
            story.focus_handle = focus_handle;
            story.closable = S::closable();
            story.zoomable = S::zoomable();
            story.name = name.into();
            story.description = description.into();
            story.title_bg = S::title_bg();
            story
        });

        view
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: gpui::Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn story(mut self, story: AnyView, story_klass: impl Into<SharedString>) -> Self {
        self.story = Some(story);
        self.story_klass = Some(story_klass.into());
        self
    }

    fn on_action_panel_info(
        &mut self,
        _: &ShowPanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        struct Info;
        let note = Notification::new(format!("You have clicked panel info on: {}", self.name))
            .id::<Info>();
        window.push_notification(note, cx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoryState {
    pub story_klass: SharedString,
}

impl StoryState {
    fn to_value(&self) -> serde_json::Value {
        serde_json::json!({
            "story_klass": self.story_klass,
        })
    }

    fn from_value(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }

    fn to_story(
        &self,
        _w: &mut Window,
        _cx: &mut App,
    ) -> (
        &'static str,
        &'static str,
        bool,
        Option<PanelControl>,
        AnyView,
    ) {
        macro_rules! story {
            ($klass:tt) => {
                (
                    $klass::title(),
                    $klass::description(),
                    $klass::closable(),
                    $klass::zoomable(),
                    $klass::view(window, cx).into(),
                )
            };
        }

        match self.story_klass.to_string().as_str() {
            // "ButtonStory" => story!(ButtonStory),
            // "CalendarStory" => story!(CalendarStory),
            // "DropdownStory" => story!(DropdownStory),
            // "IconStory" => story!(IconStory),
            // "ImageStory" => story!(ImageStory),
            // "InputStory" => story!(InputStory),
            // "ListStory" => story!(ListStory),
            // "ModalStory" => story!(ModalStory),
            // "PopupStory" => story!(PopupStory),
            // "ProgressStory" => story!(ProgressStory),
            // "ResizableStory" => story!(ResizableStory),
            // "ScrollableStory" => story!(ScrollableStory),
            // "SwitchStory" => story!(SwitchStory),
            // "TableStory" => story!(TableStory),
            // "TextStory" => story!(TextStory),
            // "TooltipStory" => story!(TooltipStory),
            // "WebViewStory" => story!(WebViewStory),
            // "AccordionStory" => story!(AccordionStory),
            // "SidebarStory" => story!(SidebarStory),
            // "FormStory" => story!(FormStory),
            _ => {
                unreachable!("Invalid story klass: {}", self.story_klass)
            }
        }
    }
}

impl Panel for StoryContainer {
    fn panel_name(&self) -> &'static str {
        "StoryContainer"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        self.name.clone().into_any_element()
    }

    fn title_style(&self, cx: &App) -> Option<TitleStyle> {
        if let Some(bg) = self.title_bg {
            Some(TitleStyle {
                background: bg,
                foreground: cx.theme().foreground,
            })
        } else {
            None
        }
    }

    fn closable(&self, _cx: &App) -> bool {
        self.closable
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        self.zoomable
    }

    fn visible(&self, cx: &App) -> bool {
        !AppState::global(cx)
            .invisible_panels
            .read(cx)
            .contains(&self.name)
    }

    fn set_zoomed(&mut self, zoomed: bool, _window: &mut Window, _cx: &mut App) {
        println!("panel: {} zoomed: {}", self.name, zoomed);
    }

    fn set_active(&mut self, active: bool, _window: &mut Window, _cx: &mut App) {
        println!("panel: {} active: {}", self.name, active);
    }

    fn popup_menu(&self, menu: PopupMenu, _window: &Window, _cx: &App) -> PopupMenu {
        menu //.track_focus(&self.focus_handle)
            .menu("Info", Box::new(ShowPanelInfo))
    }

    fn toolbar_buttons(&self, _window: &mut Window, _cx: &mut App) -> Option<Vec<Button>> {
        Some(vec![
            Button::new("info")
                .icon(IconName::Info)
                .on_click(|_, window, cx| {
                    window.push_notification("You have clicked info button", cx);
                }),
            Button::new("search")
                .icon(IconName::Search)
                .on_click(|_, window, cx| {
                    window.push_notification("You have clicked search button", cx);
                }),
        ])
    }

    fn dump(&self, _cx: &App) -> PanelState {
        let mut state = PanelState::new(self);
        let story_state = StoryState {
            story_klass: self.story_klass.clone().unwrap(),
        };
        state.info = PanelInfo::panel(story_state.to_value());
        state
    }
}

impl EventEmitter<PanelEvent> for StoryContainer {}
impl Focusable for StoryContainer {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for StoryContainer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("story-container")
            .size_full()
            .overflow_y_scroll()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_action_panel_info))
            .when(self.description.len() > 0, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .p_4()
                        .child(Label::new(self.description.clone()).text_size(px(16.0)))
                        .child(Divider::horizontal().label("This is a divider")),
                )
            })
            .when_some(self.story.clone(), |this, story| {
                this.child(
                    v_flex()
                        .id("story-children")
                        .w_full()
                        .flex_1()
                        .p_4()
                        .child(story),
                )
            })
    }
}
