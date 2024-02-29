use ratatui::{
    layout::{Position, Rect},
    style::Style,
};

use crate::{
    area_calculation::{unroll, UnrolledComponents},
    core::{RenderComponent, RenderFlow, RenderNode},
};

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RenderId(uuid::Uuid);

impl RenderId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for RenderId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Clone, Debug)]
pub struct Menu {
    pub(crate) menu_content: Vec<(char, String)>,
}

// TODO: Define an API for events that handle modes
// pub trait InputMode {
//     fn transform_event(event: crossterm::event::Event) -> InputEvent;
// }

impl Menu {
    pub fn from_entries<T: Into<String>>(entries: Vec<(char, T)>) -> Self {
        Self {
            menu_content: entries
                .into_iter()
                .map(|(k, description)| (k, description.into()))
                .collect(),
        }
    }
}

pub struct Page {
    id: RenderId,
    title: String,
    pub(crate) shortcut: char,
    root: RenderComponent,
    unrolled: UnrolledComponents,
    menu: Menu,
    pub(crate) style: Style,
}

impl Page {
    pub fn get_page_id(&self) -> &RenderId {
        &self.id
    }

    pub fn new<T: Into<RenderComponent>, S: Into<String>>(
        title: S,
        shortcut: char,
        root: T,
    ) -> Self {
        let root: RenderComponent = root.into();
        let layout_factories = unroll(&root);
        Self {
            id: RenderId::new(),
            shortcut,
            title: title.into(),
            root,
            menu: Menu::default(),
            unrolled: layout_factories,
            style: Style::default(),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_menu(&mut self, menu: Menu) -> &mut Self {
        self.menu = menu;
        self
    }

    pub fn with_menu_entries<T: Into<String>>(&mut self, entries: Vec<(char, T)>) -> &mut Self {
        self.menu = Menu::from_entries(entries);
        self
    }

    pub fn visit(&self, f: &mut dyn FnMut(&RenderNode) -> bool) {
        self.root.visit(f);
    }

    pub fn components_at_position(&self, pos: &Position, area: &Rect) -> Vec<&RenderId> {
        self.unrolled
            .0
            .iter()
            .filter_map(|(id, area_calc)| {
                let sub_area = area_calc(*area);
                if sub_area.contains(*pos) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl RenderFlow for Page {
    fn render(
        &mut self,
        opts: &crate::core::VRenderProps,
        component_buffer: &mut crate::core::ComponentBuffer,
        buff: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
    ) {
        self.root.render(opts, component_buffer, buff, area)
    }

    fn get_menu(&self) -> Option<Menu> {
        Some(self.menu.clone())
    }

    fn get_focusable_elements(&self) -> Vec<RenderId> {
        self.root.get_focusable_elements()
    }
}

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
}

impl RenderFlow for PageCollection {
    fn render(
        &mut self,
        opts: &crate::core::VRenderProps,
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

    fn get_menu(&self) -> Option<Menu> {
        Some(Menu::from_entries(
            self.pages
                .iter()
                .map(|page| (page.shortcut, &page.title))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::render::Render;

    use super::Page;

    struct MyWidget {}

    impl Render for MyWidget {
        fn render(
            &mut self,
            _render_props: &crate::render::RenderProps,
            _buff: &mut ratatui::prelude::Buffer,
            _area: ratatui::prelude::Rect,
        ) {
            todo!()
        }
    }
    #[test]
    fn test_creating_page() {
        let _page = Page::new("P1", 'p', MyWidget {});
        let _page2 = Page::new("P2", 'd', MyWidget {}).with_menu_entries(vec![('i', "Insert")]);
    }
}
