use crate::{core::RenderFlow, core::RenderId};

use super::{
    menu::{Menu, MenuEvent},
    page::Page,
};

pub struct PageCollection {
    pub(crate) pages: Vec<Page>,
    prev_page: Option<usize>,
    current_page: usize,
}

impl PageCollection {
    pub fn new(pages: Vec<Page>) -> Self {
        Self {
            pages,
            prev_page: None,
            current_page: 0,
        }
    }

    pub(crate) fn get_current_page_mut(&mut self) -> &mut Page {
        // TODO: Handle this better
        self.pages.get_mut(self.current_page).unwrap()
    }

    pub(crate) fn get_current_page(&self) -> &Page {
        // TODO: Handle this better
        self.pages.get(self.current_page).unwrap()
    }

    pub(crate) fn try_change_page(&mut self, shortcut: char) -> bool {
        let maybe_new_page = self.pages.iter().enumerate().find_map(|(i, p)| {
            if p.shortcut == shortcut {
                Some(i)
            } else {
                None
            }
        });

        if let Some(page_index) = maybe_new_page {
            self.prev_page = Some(self.current_page);
            self.current_page = page_index;
            true
        } else {
            false
        }
    }

    pub fn get_menu(&self, focused_element: &Option<RenderId>) -> Option<Menu> {
        let active_component_menu = self
            .get_current_page()
            .get_active_element_menu(focused_element);

        let mut pages_menu = Menu::from_entries(
            self.pages
                .iter()
                .map(|page| (page.shortcut, &page.title, move |_ev: MenuEvent<'_>| {}))
                .collect(),
        );

        if let Some(cmenu) = active_component_menu {
            pages_menu.append(cmenu.clone());
        }

        Some(pages_menu)
    }
}

impl RenderFlow for PageCollection {
    fn render(
        &mut self,
        opts: &mut crate::core::VRenderProps,
        component_buffer: &mut crate::core::ComponentBuffer,
        buff: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
    ) {
        self.get_current_page_mut()
            .render(opts, component_buffer, buff, area)
    }

    fn get_focusable_elements(&self) -> Vec<RenderId> {
        self.get_current_page().get_focusable_elements()
    }
}
