use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, Paragraph, Widget, Wrap},
};
use std::error::Error;
#[macro_use]
extern crate tuiwindow;
use tuiwindow::hooks::AsyncResource;
use tuiwindow::{
    api::{Menu, Page, PageCollection},
    core::InputEvent,
    tui::TuiCrossterm,
    window::DefaultEventMapper,
};
use tuiwindow::{
    core::RenderComponent,
    render::{FocusableRender, Render, RenderProps},
    window::Window,
};

#[derive(Default)]
struct SlowWidget {
    task: AsyncResource<u64>,
}

fn fib_cpu_intensive(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fib_cpu_intensive(n - 1) + fib_cpu_intensive(n - 2),
    }
}

impl FocusableRender for SlowWidget {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
        let maybe_result = self.task.with_thread_spawning(|| fib_cpu_intensive(40));

        let block = Block::new()
            .borders(Borders::all())
            .style(if render_props.is_focused {
                Style::new().fg(Color::Red)
            } else {
                Style::new()
            });
        Widget::render(
            Paragraph::new(format!("Very slow: {:?}", maybe_result)).block(block),
            area,
            buff,
        );
    }
}

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

#[derive(Default)]
struct StaticWidget {}

impl Render for StaticWidget {
    fn render(&mut self, _render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
        Paragraph::new("I'm static")
            .block(Block::new().borders(Borders::all()))
            .render(area, buff)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut tui = TuiCrossterm::new()?;
    let terminal = tui.setup()?;

    // define the collection of pages:

    let mut app = PageCollection::new(vec![
        Page::new(
            "Page 1",
            '1',
            row_widget!(
                SlowWidget::default(),
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
        )
        .with_style(Style::default().bg(Color::White).fg(Color::Black)),
    ]);

    let mut window = Window::new(&app, |ev| match ev {
        tuiwindow::core::InputEvent::Key(c) => *c == 'q',
        _ => false,
    });

    while !window.is_finished() {
        terminal.draw(|f| {
            let area = f.size();
            let buff = f.buffer_mut();

            let mut second_buff = buff.clone();
            // draw
            window.render::<DefaultEventMapper>(&mut app, &mut second_buff, area);

            buff.merge(&second_buff);
        })?;
    }

    Ok(())
}
