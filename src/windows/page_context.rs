use crate::{
    core::RenderId,
    core::{ComponentBuffer, RenderFlow},
    utils::CyclicList,
};

use super::page::Page;

pub struct PageContext {
    id: RenderId,
    page_id: RenderId,
    focusable_elements: CyclicList<RenderId>,
    pub(crate) event_buffer: ComponentBuffer,
}

impl PageContext {
    pub fn new(page: &Page) -> Self {
        let root_id = RenderId::new();
        Self {
            id: root_id,
            page_id: *page.get_page_id(),
            focusable_elements: Self::build_focusable_elements(&root_id, page),
            event_buffer: ComponentBuffer::default(),
        }
    }

    fn build_focusable_elements(window_id: &RenderId, page: &Page) -> CyclicList<RenderId> {
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

    pub(crate) fn get_focused_element(&self) -> Option<RenderId> {
        self.focusable_elements.current().cloned()
    }

    pub(crate) fn reset_focus(&mut self) {
        self.focusable_elements.reset()
    }
}
