use ratatui::{
    layout::{Position, Rect},
    style::Style,
};

use crate::{
    area_calculation::{unroll, UnrolledComponents},
    core::RenderId,
    core::{RenderComponent, RenderFlow, RenderNode},
};

use super::menu::{Menu, MenuEvent};

pub struct Page {
    id: RenderId,
    pub(crate) title: String,
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

    pub fn with_menu_entries<T: Into<String>, F: Fn(MenuEvent) + 'static>(
        &mut self,
        entries: Vec<(char, T, F)>,
    ) -> &mut Self {
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

    pub(crate) fn get_active_element_menu(
        &self,
        focused_element: &Option<RenderId>,
    ) -> Option<Menu> {
        if let Some(fe) = focused_element {
            let mut found = None;
            self.visit(&mut |details| {
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
}

impl RenderFlow for Page {
    fn render(
        &mut self,
        opts: &mut crate::core::VRenderProps,
        component_buffer: &mut crate::core::ComponentBuffer,
        buff: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
    ) {
        self.root.render(opts, component_buffer, buff, area)
    }

    fn get_menu(&self) -> Option<&Menu> {
        Some(&self.menu)
    }

    fn get_focusable_elements(&self) -> Vec<RenderId> {
        self.root.get_focusable_elements()
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
        let _page2 = Page::new("P2", 'd', MyWidget {}).with_menu_entries(vec![(
            'i',
            "Insert",
            |_: super::MenuEvent<'_>| {},
        )]);
    }
}
