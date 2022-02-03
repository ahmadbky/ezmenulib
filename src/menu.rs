struct MainFormatting<'a> {
    title: Option<&'a str>,
    pos: TitlePos,
    prefix: &'a str,
    new_line: bool,
    defaults: bool,
}

impl Default for MainFormatting<'_> {
    fn default() -> Self {
        Self {
            title: None,
            pos: Default::default(),
            prefix: ": ",
            new_line: false,
            defaults: true,
        }
    }
}

pub enum TitlePos {
    Top,
    Bottom,
}

impl Default for TitlePos {
    fn default() -> Self {
        Self::Top
    }
}
