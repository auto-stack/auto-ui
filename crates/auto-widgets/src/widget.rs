use autoval::AutoStr;
use crate::Text;
use crate::Button;

#[derive(Clone)]
pub enum Widget {
    Text(Text),
    Button(Button),
}

impl Widget {
    pub fn id(&self) -> AutoStr {
        match self {
            Widget::Text(text) => text.id(),
            Widget::Button(button) => button.id(),
        }
    }
}
