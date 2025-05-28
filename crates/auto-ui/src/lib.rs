mod app;
pub use app::*;
mod layout;
pub use layout::*;
pub mod story;

pub mod bridge;
pub mod dyna;
pub mod trans;

mod widgets;
pub use widgets::*;

#[cfg(test)]
mod tests {
    // create a test for add
    #[test]
    fn test_add() {
        let s = "a";
    }
}
