use autoval::AutoStr;

pub trait Widget {
    fn id(&self) -> AutoStr;
}