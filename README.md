# tui-window

A minimal page and focus manager for [Ratatui](https://ratatui.rs/) and
[Crossterm](https://github.com/crossterm-rs/crossterm) (though it supports other
backends that Ratatui also supports).

`tui-window` provides a very minimal setup to allow you quickly build simple,
page/page based TUI (Text-Based-User-Interface) applications.

It is loosely inspired in how HTML works: Declare a tree of widgets, and have
tui-window manage application-level concerns like **focus management**, **input
redirection** to specific widgets, etc.

## Features

- Build TUI layouts declaratively. Define a tree of components and only calculate layouts when you need a
  fine-grain control on the render process.
- An opinionated handle of focus-management: An order of focusable widgets is
  calculated (e.g. press `Tab` to focus to the next element).
- Create "pages" (collections of trees of widgets) and navigate easily between
  them.
- Supports native Ratatui widgets.
- Utilities to initialize Ratatui and Crossterm with panic handling out of the
  box.

## Status

This library is actively under development. Feel free to suggest improvements
(just keep in mind that the scope of this library is purposely small).

## Getting started

Install `tui-window` into your project using Cargo:

```bash
cargo add tuiwindow
```

Build a few widgets by implementing `Render` -for widgets that don't receive focus- or `FocusableRender` for widgets that should receive focus (deriving `Default` is not required):

```rust

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
```

Define the structure of your application (you can style whole pages):

```rust
    let mut app = PageCollection::new(vec![
        Page::new(
            "Page 1", // the page's title
            '1', // a shortcut for navigating to this page
            row_widget!( // macro for evenly-distributing your widgets in rows
                SlowWidget::default(),
                column_widget!(StaticWidget {}, TestWidget::default())
            ),
        ),
        Page::new(
            "Page 2",
            '2',
            column_widget!( // macro for evenly-distributing your widgets in
                AnotherWidget::default(),   //columns
                column_widget!(StaticWidget {}, TestWidget::default())
            ),
        )
        .with_style(Style::default().bg(Color::White).fg(Color::Black)),
    ]);
```

Put it all together in your `main` function, setting rendering using the
provided helper `TuiCrossterm`:

```rust
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
        // define the termination condition for the app:
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
```
