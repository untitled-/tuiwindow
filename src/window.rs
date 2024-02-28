use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
};
use uuid::Uuid;

use crate::core::{ComponentBuffer, InputEvent, VRenderProps};

struct WindowRenderer {}

impl WindowRenderer {
    pub(crate) fn pre_render(window: &mut Window, buff: &mut Buffer, area: Rect) -> Rect {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill(1), Constraint::Length(2)],
        )
        .split(area);

        Self::footer(window).render(layout[1], buff);
        layout[0]
    }

    fn footer(window: &mut Window) -> impl Widget {
        let text_content = if window.is_window_focused() {
            "q - Exit"
        } else {
            "ESC - Window"
        };
        Paragraph::new(text_content).block(Block::new().borders(Borders::TOP))
    }
}

pub struct Window {
    id: Uuid,
    focusable_elements: Vec<Uuid>,
    event_buffer: ComponentBuffer,
    end_condition: Box<dyn Fn(&InputEvent) -> bool>,
    is_ended: bool,
    currently_focused: usize,
}

impl Window {
    pub fn new<T: Fn(&InputEvent) -> bool + 'static>(
        app: &crate::core::Component,
        end_condition: T,
    ) -> Self {
        let window_id = Uuid::new_v4();
        let event_buffer = ComponentBuffer::default();
        let mut focusable_elements = vec![window_id];
        focusable_elements.append(&mut app.flatten_ids());

        Self {
            id: window_id,
            focusable_elements,
            event_buffer,
            end_condition: Box::new(end_condition),
            is_ended: false,
            currently_focused: 0,
        }
    }

    fn is_window_focused(&self) -> bool {
        self.focusable_elements
            .get(self.currently_focused)
            .map(|fid| fid == &self.id)
            .unwrap_or(false)
    }

    fn focus_next(&mut self) {
        if self.currently_focused + 1 >= self.focusable_elements.len() {
            self.currently_focused = 0;
        } else {
            self.currently_focused += 1;
        }
    }

    fn handle_window_event(&mut self, ev: &InputEvent) {
        if self.is_window_focused() {
            self.is_ended = (self.end_condition)(ev);
        }
        match ev {
            InputEvent::Key(_) => {}
            InputEvent::FocusNext => self.focus_next(),
            InputEvent::FocusWindow => self.currently_focused = 0,
            InputEvent::FocusPrevious => todo!(),
        }
    }

    pub fn render(&mut self, app: &mut crate::core::Component, buff: &mut Buffer, area: Rect) {
        let event = get_event();

        if let Some(ev) = &event {
            self.handle_window_event(ev);
        }
        let focused_element = self.focusable_elements.get(self.currently_focused).cloned();
        self.event_buffer
            .add_event(focused_element.unwrap_or(self.id), &event);

        let area = WindowRenderer::pre_render(self, buff, area);
        app.render(
            &VRenderProps {
                focused_element,
                event,
            },
            &mut self.event_buffer,
            buff,
            area,
        )
    }

    pub fn is_finished(&self) -> bool {
        self.is_ended
    }
}

pub fn get_event() -> Option<InputEvent> {
    if let Ok(has_ev) = event::poll(Duration::from_millis(250)) {
        if has_ev {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == event::KeyEventKind::Press {
                    return match key.code {
                        KeyCode::Char(c) => Some(InputEvent::Key(c)),
                        KeyCode::Tab => Some(InputEvent::FocusNext),
                        KeyCode::Esc => Some(InputEvent::FocusWindow),
                        _ => None,
                    };
                }
                // if let Event::Mouse(mevent) = event {
                //     if let MouseEventKind::Up(_) = mevent.kind {
                //         self.clicked_position = Some(Position::new(mevent.column, mevent.row));
                //     }
                // }
            }
        }
    }
    None
}
