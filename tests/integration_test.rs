use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::error::Error;
use std::thread;
use std::time::Duration;

#[cfg(feature = "macros")]
#[macro_use]
extern crate tuiwindow_macros;

use tuiwindow::core::RenderComponent;
#[macro_use]
extern crate tuiwindow;
use tuiwindow::{
    core::InputEvent,
    render::{FocusableRender, Render, RenderProps},
    tui::TuiCrossterm,
    windows::{
        page::Page,
        page_collection::PageCollection,
        window::{DefaultEventMapper, Window},
    },
};

struct TestWidget {}

impl FocusableRender for TestWidget {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
        Paragraph::new(format!("Hello world! Focused? {}", render_props.is_focused))
            .block(Block::new().borders(Borders::all()))
            .render(area, buff)
    }
}

#[derive(Default)]
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
    let mut app = PageCollection::new(vec![Page::new(
        "Page",
        'p',
        row_widget!(TestWidget {}, StaticWidget::default()),
    )]);
    let mut window = Window::new(&app, |ev| match ev {
        InputEvent::Key(c) => *c == 'q',
        _ => false,
    });
    terminal.draw(|f| {
        let area = f.size();
        let buff = f.buffer_mut();

        // draw
        window.render::<DefaultEventMapper>(&mut app, buff, area)
    })?;
    thread::sleep(Duration::from_secs(1));

    // TuiCrossterm::tear_down(terminal)
    Ok(())
}
