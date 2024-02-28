pub mod core;
#[macro_use]
pub mod macros;
pub mod tui;
pub mod window;

#[cfg(test)]
mod tests {

    use std::error::Error;
    use std::thread;
    use std::time::Duration;

    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{Block, Borders, Paragraph, Widget},
    };

    use crate::tui::TuiCrossterm;
    use crate::{
        core::{Component, FocusableRender, Render, RenderProps},
        window::Window,
    };

    struct TestWidget {}

    impl FocusableRender for TestWidget {
        fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
            Paragraph::new(format!("Hello world! Focused? {}", render_props.is_focused))
                .block(Block::new().borders(Borders::all()))
                .render(area, buff)
        }
    }

    struct StaticWidget {}

    impl Render for StaticWidget {
        fn render(&mut self, _render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
            Paragraph::new("I'm static")
                .block(Block::new().borders(Borders::all()))
                .render(area, buff)
        }
    }

    #[test]
    fn tui_test() -> Result<(), Box<dyn Error>> {
        thread::sleep(Duration::from_millis(500));

        let mut tui = TuiCrossterm::new()?;
        let terminal = tui.setup()?;
        // our stuff:
        let mut app: Component = row_widget!(TestWidget {}, StaticWidget {});
        let mut window = Window::new(&app, |ev| match ev {
            crate::core::InputEvent::Key(c) => *c == 'q',
            _ => false,
        });
        terminal.draw(|f| {
            let area = f.size();
            let buff = f.buffer_mut();

            // draw
            window.render(&mut app, buff, area)
        })?;
        thread::sleep(Duration::from_secs(2));

        // TuiCrossterm::tear_down(terminal)
        Ok(())
    }

    #[test]
    fn test_asoc() {}
}
