use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    text::Text,
    widgets::Widget,
};

use crate::{
    core::RenderId,
    core::{InputEvent, RenderFlow, VRenderProps},
    utils::SelectableHashMap,
};

use super::{
    alerts::AlertManager,
    menu::{Menu, MenuItem},
    page::Page,
    page_collection::PageCollection,
    page_context::PageContext,
};

struct WindowRenderer {}

// TODO: Too ugly, lets improve this. Make it extensible
impl WindowRenderer {
    pub(crate) fn pre_render(
        window: &mut Window,
        current_page: char,
        maybe_menu: Option<Menu>,
        maybe_focused_menu: Option<Menu>,
        buff: &mut Buffer,
        area: Rect,
    ) -> Rect {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill(1), Constraint::Length(1)],
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
        maybe_menu: Option<Menu>,
        maybe_focused_menu: Option<Menu>,
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
        } else {
            let mut extra = WindowRenderer::format_menu(maybe_focused_menu, current_page);
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

    fn format_menu(maybe_menu: Option<Menu>, current_page: char) -> Vec<String> {
        if let Some(menu) = maybe_menu {
            menu.menu_content
                .iter()
                .filter(|MenuItem { shortcut, .. }| *shortcut != current_page)
                .map(
                    |MenuItem {
                         shortcut,
                         display_name,
                         ..
                     }| format!("{}) {}", shortcut, display_name),
                )
                .collect()
        } else {
            vec![]
        }
    }
}

pub struct Window {
    id: RenderId,
    end_condition: Box<dyn Fn(&InputEvent) -> bool>,
    is_ended: bool,
    page_context_map: SelectableHashMap<RenderId, PageContext>,
    alerts: AlertManager,
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
        let window_id = RenderId::new();

        Self {
            id: window_id,
            end_condition: Box::new(end_condition),
            is_ended: false,
            page_context_map: SelectableHashMap::new(
                *app.get_current_page().get_page_id(),
                app.pages
                    .iter()
                    .map(|page| (*page.get_page_id(), PageContext::new(page)))
                    .collect(),
            ),
            alerts: AlertManager::default(),
        }
    }

    fn is_window_focused(&self) -> bool {
        self.page_context_map
            .get_current()
            .map(|p| p.is_window_focused())
            .unwrap_or(false)
    }

    fn on_page_change(&mut self, page: &PageCollection) {
        let new_page = page.get_current_page();
        if let Some(new_context) = self.page_context_map.get_mut(new_page.get_page_id()) {
            new_context.reconcile(new_page)
        }
        self.page_context_map.set_current(*new_page.get_page_id())
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
            InputEvent::FocusNext => {
                if let Some(p) = self.page_context_map.get_current_mut() {
                    p.focus_next()
                }
            }
            InputEvent::FocusWindow => {
                if let Some(p) = self.page_context_map.get_current_mut() {
                    p.reset_focus()
                }
            }
            InputEvent::FocusPrevious => {
                if let Some(p) = self.page_context_map.get_current_mut() {
                    p.focus_prev()
                }
            }
            InputEvent::Click(_) => {
                // TODO
            }
        };

        WindowEventResult::None
    }

    fn get_active_element_menu(
        focused_element: &Option<RenderId>,
        current_page: &Page,
    ) -> Option<Menu> {
        if let Some(fe) = focused_element {
            let mut found = None;
            current_page.visit(&mut |details| {
                if details.id == *fe {
                    found = details.render.get_menu();
                    false
                } else {
                    true
                }
            });
            found
        } else {
            None
        }
    }

    fn draw_overlays(&mut self, buf: &mut Buffer, area: Rect) {
        if let Some(alert) = &mut self.alerts.first_visible() {
            alert.render(area, buf)
        }
    }

    pub fn render<T: EventMapper>(
        &mut self,
        app: &mut PageCollection,
        buff: &mut Buffer,
        area: Rect,
    ) {
        let event = get_event::<T>();

        let focused_element = if let Some(page) = self.page_context_map.get_current_mut() {
            let focused_element = page.get_focused_element();
            page.event_buffer
                .add_event(focused_element.unwrap_or(self.id), &event);

            focused_element
        } else {
            None
        };
        if let Some(ev) = &event {
            match self.handle_window_event(ev, app) {
                WindowEventResult::PageChange => self.on_page_change(app),
                WindowEventResult::None => {}
            };

            if let Some(mut menu) = app.get_menu(&focused_element) {
                menu.handle_event(&mut self.alerts, ev)
            }
        }

        let current_page_style = app.get_current_page().style;
        buff.set_style(area, current_page_style);
        let area = WindowRenderer::pre_render(
            self,
            app.get_current_page().shortcut,
            app.get_menu(&focused_element),
            Self::get_active_element_menu(&focused_element, app.get_current_page()),
            buff,
            area,
        );

        // TODO: Deal with option
        let event_buffer = self
            .page_context_map
            .get_current_mut()
            .map(|p| &mut p.event_buffer)
            .unwrap();
        app.render(
            &mut VRenderProps {
                alerts: &mut self.alerts,
                focused_element,
                event,
            },
            event_buffer,
            buff,
            area,
        );

        self.draw_overlays(buff, area);
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
        match ev {
            Event::FocusGained => None,
            Event::FocusLost => None,
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::SHIFT,
                code: KeyCode::Tab,
                ..
            }) => Some(InputEvent::FocusPrevious),
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => Some(InputEvent::Key(c)),
                KeyCode::Tab => Some(InputEvent::FocusNext),
                KeyCode::Esc => Some(InputEvent::FocusWindow),
                _ => None,
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::Up(_) => Some(InputEvent::Click(Position::new(
                    mouse_event.column,
                    mouse_event.row,
                ))),
                _ => None,
            },
            Event::Paste(_) => None,
            Event::Resize(_, _) => None,
        }
    }
}

pub fn get_event<T: EventMapper>() -> Option<InputEvent> {
    if let Ok(has_ev) = event::poll(Duration::from_millis(250)) {
        if has_ev {
            if let Ok(ev) = event::read() {
                return T::to_input_event(&ev);
            }
        }
    }
    None
}
