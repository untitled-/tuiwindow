use std::error::Error;
use std::thread;
use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, Paragraph, Widget, Wrap},
};
#[macro_use]
extern crate vtree;
use vtree::{
    api::{Menu, Page, PageCollection},
    core::InputEvent,
    tui::TuiCrossterm,
    window::DefaultEventMapper,
};
use vtree::{
    core::{FocusableRender, Render, RenderComponent, RenderProps},
    window::Window,
};

#[derive(Default)]
struct AnotherWidget {}

impl FocusableRender for AnotherWidget {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
        let block = Block::new()
            .borders(Borders::all())
            .style(if render_props.is_focused {
                Style::new().fg(Color::Red)
            } else {
                Style::new()
            });
        Widget::render(
            List::new(vec!["Element 1", "Element 2"]).block(block),
            area,
            buff,
        );
    }

    fn get_menu(&self) -> Option<Menu> {
        Some(Menu::from_entries(vec![('/', "Search")]))
    }
}

#[derive(Default)]
struct TestWidget {
    text_content: String,
}

impl FocusableRender for TestWidget {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
        if let Some(InputEvent::Key(c)) = render_props.event {
            self.text_content.push(c)
        }
        Paragraph::new(format!(
            "Hello world! Focused? {}: {}",
            render_props.is_focused, self.text_content
        ))
        .block(
            Block::new()
                .borders(Borders::all())
                .style(if render_props.is_focused {
                    Style::new().fg(Color::Red)
                } else {
                    Style::new()
                }),
        )
        .wrap(Wrap { trim: false })
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

fn main() -> Result<(), Box<dyn Error>> {
    thread::sleep(Duration::from_millis(500));

    let mut tui = TuiCrossterm::new()?;
    let terminal = tui.setup()?;
    // our stuff:

    let mut app = PageCollection::new(vec![
        Page::new(
            "Page 1",
            '1',
            row_widget!(
                AnotherWidget::default(),
                column_widget!(StaticWidget {}, TestWidget::default())
            ),
        ),
        Page::new(
            "Page 2",
            '2',
            column_widget!(
                AnotherWidget::default(),
                column_widget!(StaticWidget {}, TestWidget::default())
            ),
        ),
    ]);

    let mut window = Window::new(&app, |ev| match ev {
        vtree::core::InputEvent::Key(c) => *c == 'q',
        _ => false,
    });

    while !window.is_finished() {
        terminal.draw(|f| {
            let area = f.size();
            let buff = f.buffer_mut();

            // draw
            window.render::<DefaultEventMapper>(&mut app, buff, area)
        })?;
    }

    // TuiCrossterm::tear_down(terminal)
    Ok(())
}
