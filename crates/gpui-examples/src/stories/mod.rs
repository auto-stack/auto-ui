// Stories module - all Story implementations

mod button_story;
mod checkbox_story;
mod select_story;
mod welcome_story;

pub use button_story::ButtonStory;
pub use checkbox_story::CheckboxStory;
pub use select_story::SelectStory;
pub use welcome_story::WelcomeStory;

/// Register all default stories with the Gallery
pub fn register_default_stories() {
    // Stories are registered directly in gallery.rs
    // This module is for organization and exports
}
