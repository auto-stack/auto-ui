use gpui::*;

#[derive(Debug, Clone, Copy, Default)]
pub enum SizeScale {
    L,
    #[default]
    M,
    S,
}

pub fn icon_size_for(size: SizeScale) -> Rems {
    match size {
        SizeScale::L => Rems(3.),
        SizeScale::M => Rems(2.),
        SizeScale::S => Rems(1.),
    }
}

pub trait Sizer<T: Styled> {
    fn text_size_for(self, size: SizeScale) -> T;
    fn py_for(self, size: SizeScale) -> T;
    fn pl_for(self, size: SizeScale) -> T;
    fn pr_for(self, size: SizeScale) -> T;
    fn h_for(self, size: SizeScale) -> T;
    fn min_w_for(self, size: SizeScale) -> T;
}

impl<T: Styled> Sizer<T> for T {
    fn text_size_for(self, size: SizeScale) -> T {
        match size {
            SizeScale::L => self.text_lg(),
            SizeScale::M => self.text_base(),
            SizeScale::S => self.text_sm(),
        }
    }

    fn py_for(self, size: SizeScale) -> T {
        match size {
            SizeScale::L => self.py_5(),
            SizeScale::M => self.py(px(2.)),
            SizeScale::S => self.py(px(1.)),
        }
    }

    fn pl_for(self, size: SizeScale) -> T {
        match size {
            SizeScale::L => self.pl_5(),
            SizeScale::M => self.pl(px(2.)),
            SizeScale::S => self.pl(px(1.)),
        }
    }

    fn pr_for(self, size: SizeScale) -> T {
        match size {
            SizeScale::L => self.pr_5(),
            SizeScale::M => self.pr(px(2.)),
            SizeScale::S => self.pr(px(1.)),
        }
    }

    fn h_for(self, size: SizeScale) -> T {
        match size {
            SizeScale::L => self.h_11(),
            SizeScale::M => self.h_8(),
            SizeScale::S => self.h_6(),
        }
    }

    fn min_w_for(self, size: SizeScale) -> T {
        match size {
            SizeScale::L => self.min_w_32(),
            SizeScale::M => self.min_w_24(),
            SizeScale::S => self.min_w_16(),
        }
    }
}

