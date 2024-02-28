use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};
use uuid::Uuid;

use crate::{
    api::{Menu, PageCollection},
    core::{ComponentBuffer, InputEvent, RenderFlow, VRenderProps},
};

struct WindowRenderer {}

// TODO: Too ugly, lets improve this
impl WindowRenderer {
    pub(crate) fn pre_render(
        window: &mut Window,
        current_page: char,
        maybe_menu: &Option<Menu>,
        maybe_focused_menu: &Option<Menu>,
        buff: &mut Buffer,
        area: Rect,
    ) -> Rect {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill(1), Constraint::Length(2)],
        )
        .split(area);

        Self::footer(
            window,
            current_page,
            maybe_menu,
            maybe_focused_menu,
            layout[1],
            buff,
        );
        layout[0]
    }

    fn footer(
        window: &mut Window,
        current_page: char,
        maybe_menu: &Option<Menu>,
        maybe_focused_menu: &Option<Menu>,
        area: Rect,
        buff: &mut Buffer,
    ) {
        let mut items: Vec<String> = if window.is_window_focused() {
            vec!["q) Exit"].into_iter().map(String::from).collect()
        } else {
            vec!["ESC) Window"].into_iter().map(String::from).collect()
        };

        if window.is_window_focused() {
            let mut extra = WindowRenderer::format_menu(maybe_menu, current_page);
            items.append(&mut extra);
        }
        let footer_layout = Layout::new(
            Direction::Horizontal,
            items.iter().map(|_| Constraint::Fill(1)),
        )
        .split(area);

        for (i, c) in items.iter().zip(footer_layout.iter()) {
            Text::raw(i).render(*c, buff);
        }
    }

    fn format_menu(maybe_menu: &Option<Menu>, current_page: char) -> Vec<String> {
        if let Some(menu) = maybe_menu {
            menu.menu_content
                .iter()
                .filter(|(k, _)| *k != current_page)
                .map(|(k, v)| format!("{}) {}", k, v))
                .collect()
        } else {
            vec![]
        }
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

enum WindowEventResult {
    PageChange,
    None,
}

impl Window {
    pub fn new<F: Fn(&InputEvent) -> bool + 'static>(
        app: &PageCollection,
        end_condition: F,
    ) -> Self {
        let window_id = Uuid::new_v4();
        let event_buffer = ComponentBuffer::default();
        let mut focusable_elements = vec![window_id];
        focusable_elements.append(&mut app.get_focusable_elements());

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

    fn on_page_change(&mut self, page: &PageCollection) {
        let event_buffer = ComponentBuffer::default();
        let mut focusable_elements = vec![self.id];
        focusable_elements.append(&mut page.get_focusable_elements());
        self.focusable_elements = focusable_elements;
        self.event_buffer = event_buffer;
        self.currently_focused = 0;
    }

    fn handle_window_event(
        &mut self,
        ev: &InputEvent,
        pages: &mut PageCollection,
    ) -> WindowEventResult {
        if self.is_window_focused() {
            self.is_ended = (self.end_condition)(ev);
        }
        match ev {
            InputEvent::Key(c) => {
                if self.is_window_focused() && pages.try_change_page(*c) {
                    return WindowEventResult::PageChange;
                }
            }
            InputEvent::FocusNext => self.focus_next(),
            InputEvent::FocusWindow => self.currently_focused = 0,
            InputEvent::FocusPrevious => todo!(),
        };

        WindowEventResult::None
    }

    pub fn render<T: EventMapper>(
        &mut self,
        app: &mut PageCollection,
        buff: &mut Buffer,
        area: Rect,
    ) {
        let event = get_event::<T>();

        if let Some(ev) = &event {
            match self.handle_window_event(ev, app) {
                WindowEventResult::PageChange => self.on_page_change(app),
                WindowEventResult::None => {}
            };
        }
        let focused_element = self.focusable_elements.get(self.currently_focused).cloned();
        self.event_buffer
            .add_event(focused_element.unwrap_or(self.id), &event);

        let area = WindowRenderer::pre_render(
            self,
            app.get_current_page().shortcut,
            &app.get_menu(),
            &None, //TODO: extract menu from focused element
            buff,
            area,
        );
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

pub trait EventMapper {
    fn to_input_event(ev: &crossterm::event::Event) -> Option<InputEvent>;
}

#[derive(Default)]
pub struct DefaultEventMapper {}

impl EventMapper for DefaultEventMapper {
    fn to_input_event(ev: &crossterm::event::Event) -> Option<InputEvent> {
        if let Event::Key(key) = ev {
            return match key.code {
                KeyCode::Char(c) => Some(InputEvent::Key(c)),
                KeyCode::Tab => Some(InputEvent::FocusNext),
                KeyCode::Esc => Some(InputEvent::FocusWindow),
                _ => None,
            };
        }
        None
    }
}

pub fn get_event<T: EventMapper>() -> Option<InputEvent> {
    if let Ok(has_ev) = event::poll(Duration::from_millis(250)) {
        if has_ev {
            if let Ok(ev) = event::read() {
                return T::to_input_event(&ev);
            }
            // if let Ok(Event::Key(key)) = event::read() {
            //     if key.kind == event::KeyEventKind::Press {
            //         return match key.code {
            //             KeyCode::Char(c) => Some(InputEvent::Key(c)),
            //             KeyCode::Tab => Some(InputEvent::FocusNext),
            //             KeyCode::Esc => Some(InputEvent::FocusWindow),
            //             _ => None,
            //         };
            //     }
            //     // if let Event::Mouse(mevent) = event {
            //     //     if let MouseEventKind::Up(_) = mevent.kind {
            //     //         self.clicked_position = Some(Position::new(mevent.column, mevent.row));
            //     //     }
            //     // }
            // }
        }
    }
    None
}
