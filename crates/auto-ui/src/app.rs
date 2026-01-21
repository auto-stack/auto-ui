// Unified App abstraction for auto-ui
//
// This module provides a backend-agnostic way to run applications.
// Developers can use `auto_ui::App::run()` and select the backend
// through feature flags.

use crate::Component;

/// Error type for App operations
pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Unified App entry point
///
/// This struct provides a unified way to run auto-ui applications with
/// different backends (Iced, GPUI, etc.) selected via Cargo features.
///
/// # Examples
///
/// ## Running with default backend (selected by feature flags)
///
/// ```no_run
/// use auto_ui::{Component, View, App};
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     type Msg = ();
///     fn on(&mut self, _msg: Self::Msg) {}
///     fn view(&self) -> View<Self::Msg> {
///         View::text("Hello, World!")
///     }
/// }
///
/// fn main() -> auto_ui::AppResult<()> {
///     App::run::<MyComponent>()
/// }
/// ```
///
/// ## Selecting a specific backend
///
/// ```no_run
/// # use auto_ui::{Component, View, App};
/// # struct MyComponent;
/// # impl Component for MyComponent {
/// #     type Msg = ();
/// #     fn on(&mut self, _msg: Self::Msg) {}
/// #     fn view(&self) -> View<Self::Msg> { View::text("") }
/// # }
/// fn main() -> auto_ui::AppResult<()> {
///     #[cfg(feature = "iced")]
///     return App::run_iced::<MyComponent>();
///
///     #[cfg(feature = "gpui")]
///     return App::run_gpui::<MyComponent>();
///
///     #[cfg(not(any(feature = "iced", feature = "gpui")))]
///     Err("No backend enabled".into())
/// }
/// ```
pub struct App;

impl App {
    /// Run the application with default backend selected via feature flags
    ///
    /// Backends are selected in this priority order:
    /// 1. "iced" feature → Iced backend
    /// 2. "gpui" feature → GPUI backend
    ///
    /// # Example
    /// ```no_run
    /// # use auto_ui::{Component, View, App};
    /// # struct MyComponent;
    /// # impl Component for MyComponent {
    /// #     type Msg = ();
    /// #     fn on(&mut self, _msg: Self::Msg) {}
    /// #     fn view(&self) -> View<Self::Msg> { View::text("") }
    /// # }
    /// fn main() -> auto_ui::AppResult<()> {
    ///     App::run::<MyComponent>()
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns an error if no backend feature is enabled.
    pub fn run<C>() -> AppResult<()>
    where
        C: Component + Default + 'static,
    {
        #[cfg(feature = "iced")]
        {
            return Self::run_iced::<C>();
        }

        // TODO: Re-enable when auto-ui-gpui is properly set up
        #[cfg(all(feature = "gpui", not(feature = "iced")))]
        {
            return Err("GPUI backend is temporarily disabled while styling system integration is in progress".into());
        }

        #[cfg(not(any(feature = "iced", feature = "gpui")))]
        {
            return Err(
                "No backend enabled. Please enable either 'iced' or 'gpui' feature in Cargo.toml. \
                 Example: auto-ui = { version = \"0.1\", features = [\"iced\"] }"
                    .into(),
            );
        }
    }

    /// Run the application explicitly with Iced backend
    ///
    /// This requires the "iced" feature to be enabled.
    ///
    /// # Example
    /// ```no_run
    /// # use auto_ui::{Component, View, App};
    /// # struct MyComponent;
    /// # impl Component for MyComponent {
    /// #     type Msg = ();
    /// #     fn on(&mut self, _msg: Self::Msg) {}
    /// #     fn view(&self) -> View<Self::Msg> { View::text("") }
    /// # }
    /// fn main() -> auto_ui::AppResult<()> {
    ///     App::run_iced::<MyComponent>()
    /// }
    /// ```
    #[cfg(feature = "iced")]
    pub fn run_iced<C>() -> AppResult<()>
    where
        C: Component + Default + 'static,
    {
        auto_ui_iced::run_app::<C>()
    }

    /// Run the application explicitly with GPUI backend
    ///
    /// This requires the "gpui" feature to be enabled.
    ///
    /// # Note
    /// GPUI backend requires manual implementation of the GPUI `Render` trait.
    /// This function will return an error directing you to the examples.
    /// See `auto-ui-gpui-examples/src/bin/counter.rs` for the proper pattern.
    ///
    /// # Example
    /// ```no_run
    /// # use auto_ui::{Component, View, App};
    /// # struct MyComponent;
    /// # impl Component for MyComponent {
    /// #     type Msg = ();
    /// #     fn on(&mut self, _msg: Self::Msg) {}
    /// #     fn view(&self) -> View<Self::Msg> { View::text("") }
    /// # }
    /// fn main() -> auto_ui::AppResult<()> {
    ///     App::run_gpui::<MyComponent>()
    /// }
    /// ```
    // TODO: Re-enable when auto-ui-gpui is properly set up as a dependency
    // #[cfg(feature = "gpui")]
    // pub fn run_gpui<C>() -> AppResult<()>
    // where
    //     C: Component + Default + 'static,
    // {
    //     auto_ui_gpui::run_app::<C>()
    // }

    // Empty function to satisfy doc comment
    #[cfg(feature = "gpui")]
    fn _run_gpui_placeholder() {}
}



