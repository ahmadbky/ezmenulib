use cursive::View;

pub struct CursiveMenu {
    title: Option<String>,
}

impl View for CursiveMenu {
    fn draw(&self, printer: &cursive::Printer) {
        todo!()
    }

    fn layout(&mut self, _: cursive::Vec2) {}

    fn needs_relayout(&self) -> bool {
        true
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        let _ = constraint;
        cursive::Vec2::new(1, 1)
    }

    fn on_event(&mut self, _: cursive::event::Event) -> cursive::event::EventResult {
        cursive::event::EventResult::Ignored
    }

    fn call_on_any<'a>(&mut self, _: &cursive::view::Selector<'_>, _: cursive::event::AnyCb<'a>) {}

    fn focus_view(
        &mut self,
        _: &cursive::view::Selector<'_>,
    ) -> Result<cursive::event::EventResult, cursive::view::ViewNotFound> {
        Err(cursive::view::ViewNotFound)
    }

    fn take_focus(
        &mut self,
        source: cursive::direction::Direction,
    ) -> Result<cursive::event::EventResult, cursive::view::CannotFocus> {
        let _ = source;

        Err(cursive::view::CannotFocus)
    }

    fn important_area(&self, view_size: cursive::Vec2) -> cursive::Rect {
        cursive::Rect::from_size((0, 0), view_size)
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
