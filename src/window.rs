use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{buffer::Buffer, layout::Rect};
use uuid::Uuid;

use crate::core::{ComponentBuffer, InputEvent, VRenderProps};

pub struct Window {
    id: Uuid,
    focusable_elements: Vec<Uuid>,
    event_buffer: ComponentBuffer,
}

impl Window {
    pub fn new(app: &crate::core::Component) -> Self {
        let event_buffer = ComponentBuffer::default();
        let focusable_elements = app.flatten_ids();

        Self {
            id: Uuid::new_v4(),
            focusable_elements,
            event_buffer,
        }
    }
    pub fn render(&mut self, app: &mut crate::core::Component, buff: &mut Buffer, area: Rect) {
        let event = get_event();
        let focused_element = self.focusable_elements.first().cloned();
        self.event_buffer
            .add_event(focused_element.unwrap_or(self.id), &event);
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
}

pub fn get_event() -> Option<InputEvent> {
    if let Ok(has_ev) = event::poll(Duration::from_millis(250)) {
        if has_ev {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == event::KeyEventKind::Press {
                    return match key.code {
                        KeyCode::Char(c) => Some(InputEvent::Key(c)),
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
