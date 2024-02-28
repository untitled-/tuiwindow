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
    api::{Menu, Page, PageCollection},
    core::{ComponentBuffer, InputEvent, RenderFlow, VRenderProps},
    utils::{CyclicList, SelectableHashMap},
};

struct WindowRenderer {}

struct PageContext {
    id: Uuid,
    page_id: Uuid,
    focusable_elements: CyclicList<Uuid>,
    event_buffer: ComponentBuffer,
}

impl PageContext {
    fn new(page: &Page) -> Self {
        let root_id = Uuid::new_v4();
        Self {
            id: root_id,
            page_id: *page.get_page_id(),
            focusable_elements: Self::build_focusable_elements(&root_id, page),
            event_buffer: ComponentBuffer::default(),
        }
    }

    fn build_focusable_elements(window_id: &Uuid, page: &Page) -> CyclicList<Uuid> {
        let mut focusable_elements = vec![*window_id];
        focusable_elements.append(&mut page.get_focusable_elements());
        CyclicList::new(focusable_elements)
    }

    pub(crate) fn reconcile(&mut self, page: &Page) {
        if self.page_id != *page.get_page_id() {
            self.focusable_elements = Self::build_focusable_elements(&self.id, page);
            self.event_buffer = ComponentBuffer::default();
            self.page_id = *page.get_page_id();
        }
    }

    pub(crate) fn is_window_focused(&self) -> bool {
        self.focusable_elements
            .current()
            .map(|fid| fid == &self.id)
            .unwrap_or(false)
    }

    pub(crate) fn focus_next(&mut self) {
        self.focusable_elements.move_next();
    }

    pub(crate) fn focus_prev(&mut self) {
        self.focusable_elements.move_previous();
    }

    pub(crate) fn get_focused_element(&self) -> Option<Uuid> {
        self.focusable_elements.current().cloned()
    }

    pub(crate) fn reset_focus(&mut self) {
        self.focusable_elements.reset()
    }
}

// TODO: Too ugly, lets improve this. Make it extensible
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
    end_condition: Box<dyn Fn(&InputEvent) -> bool>,
    is_ended: bool,
    page_context_map: SelectableHashMap<Uuid, PageContext>,
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

        Self {
            id: window_id,
            // focusable_elements,
            // event_buffer,
            end_condition: Box::new(end_condition),
            is_ended: false,
            // currently_focused: 0,
            page_context_map: SelectableHashMap::new(
                *app.get_current_page().get_page_id(),
                app.pages
                    .iter()
                    .map(|page| (*page.get_page_id(), PageContext::new(page)))
                    .collect(),
            ),
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
        };

        WindowEventResult::None
    }

    fn get_active_element_menu(
        focused_element: &Option<Uuid>,
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

        let focused_element = if let Some(page) = self.page_context_map.get_current_mut() {
            let focused_element = page.get_focused_element();
            page.event_buffer
                .add_event(focused_element.unwrap_or(self.id), &event);

            focused_element
        } else {
            None
        };

        let area = WindowRenderer::pre_render(
            self,
            app.get_current_page().shortcut,
            &app.get_menu(),
            &Self::get_active_element_menu(&focused_element, app.get_current_page()), //TODO: extract menu from focused element
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
            &VRenderProps {
                focused_element,
                event,
            },
            event_buffer,
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
