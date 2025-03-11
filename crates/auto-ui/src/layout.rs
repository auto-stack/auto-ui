use gpui::{div, Div, Styled, FlexDirection};

pub trait XYLayout {
    fn w_begin(self) -> Self;
    fn w_end(self) -> Self;
    fn h_begin(self) -> Self;
    fn h_end(self) -> Self;
    fn w_center(self) -> Self;
    fn h_center(self) -> Self;
    fn center(self) -> Self;
}

impl <E: Styled> XYLayout for E {
    fn w_begin(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.justify_start()
            } else {
                self.items_start()
            }
        } else {
            self.flex_row().justify_start()
        }
    }

    fn w_end(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.justify_end()
            } else {
                self.items_end()
            }
        } else {
            self.flex_row().justify_end()
        }
    }
    
    fn h_begin(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.justify_start()
            } else {
                self.items_start()
            }
        } else {
            self.flex().flex_col().items_start()
        }
    }

    fn h_end(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.justify_end()
            } else {
                self.items_end()
            }
        } else {
            self.flex().flex_col().items_end()
        }
    }

    fn w_center(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.justify_center()
            } else {    
                self.items_center()
            }
        } else {
            self.flex().flex_grow().items_center().justify_center()
        }
    }
    
    fn h_center(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.justify_center()
            } else {
                self.items_center()
            }
        } else {
            self.flex().flex_grow().items_center().justify_center()
        }
    }

    fn center(mut self) -> Self {
        if let Some(direction) = self.style().flex_direction {
            if direction == FlexDirection::Row {
                self.h_full().justify_center().items_center()
            } else {
                self.w_full().items_center().justify_center()
            }
        } else {
            self.flex().flex_grow().items_center().justify_center()
        }
    }

}



#[inline]
pub fn center() -> Div {
    div().flex().flex_grow().size_full().items_center().justify_center()
}

#[inline]
pub fn row() -> Div {
    div().flex().flex_row()
}

#[inline]
pub fn col() -> Div {
    div().flex().flex_col()
}

