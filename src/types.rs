#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ClickType {
    Single,
    Double,
}

#[derive(PartialEq, Copy, Clone)]
pub enum ClickPosition {
    Mouse,
    Coord,
}

#[derive(PartialEq, Copy, Clone)]
pub enum AppMode {
    Bot,
    Humanlike,
}
