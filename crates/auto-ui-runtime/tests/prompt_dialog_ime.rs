use iced::keyboard::key::Named;
use iced::{Element, Event, Font, Pixels, Size};
use iced_test::runtime::{
    core::{clipboard::Null, input_method, renderer::Headless, widget::operation},
    user_interface::{Cache, UserInterface},
};

#[test]
fn preedit_enter_waits_for_commit_or_cancel_in_the_standard_native_input() {
    let id = iced::widget::Id::from("prompt-name");
    let input: Element<'_, String> = iced::widget::text_input("Name", "Inbox")
        .id(id.clone())
        .on_input(|value| format!("edit:{value}"))
        .on_submit("submit".into())
        .into();
    let mut renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
        Font::DEFAULT,
        Pixels(12.0),
        None,
    ))
    .expect("native input test renderer");
    let mut ui = UserInterface::build(
        input,
        Size::new(420.0, 200.0),
        Cache::default(),
        &mut renderer,
    );
    ui.operate(
        &mut renderer,
        &mut operation::focusable::focus::<()>(id.clone()),
    );
    ui.operate(
        &mut renderer,
        &mut operation::text_input::select_all::<()>(id),
    );

    let mut drive = |events: &[Event]| {
        let mut messages = Vec::new();
        ui.update(
            events,
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut Null,
            &mut messages,
        );
        messages
    };
    let enter = iced_test::simulator::tap_key(Named::Enter, None).collect::<Vec<_>>();
    let escape = iced_test::simulator::tap_key(Named::Escape, None).collect::<Vec<_>>();
    assert!(drive(&[
        Event::InputMethod(input_method::Event::Opened),
        Event::InputMethod(input_method::Event::Preedit("读".into(), None)),
    ])
    .is_empty());
    assert!(
        drive(&enter).is_empty(),
        "composition Enter triggered a submission"
    );
    assert_eq!(
        drive(&[Event::InputMethod(input_method::Event::Commit(
            "读书".into()
        ))]),
        ["edit:读书"]
    );
    assert_eq!(drive(&enter), ["submit"]);

    drive(&[Event::InputMethod(input_method::Event::Preedit(
        "丢弃".into(),
        None,
    ))]);
    assert!(drive(&escape).is_empty());
    drive(&[Event::InputMethod(input_method::Event::Closed)]);
    assert_eq!(
        drive(&enter),
        ["submit"],
        "cancelled preedit lost the edit focus"
    );
}
