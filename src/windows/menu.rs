use std::rc::Rc;

use crate::core::InputEvent;

use super::alerts::AlertManager;

pub struct MenuEvent<'a> {
    pub alerts: &'a mut AlertManager,
}

pub type MenuItemEventHandler = Box<dyn Fn(MenuEvent)>;

#[derive(Clone)]
pub struct MenuItem {
    pub(crate) shortcut: char,
    pub(crate) display_name: String,
    pub(crate) handler: Rc<MenuItemEventHandler>,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("shortcut", &self.shortcut)
            .field("display_name", &self.display_name)
            .field("handler", &format_args!("{}", "<>"))
            .finish()
    }
}

#[derive(Clone, Default, Debug)]
pub struct Menu {
    pub(crate) menu_content: Vec<MenuItem>,
}

impl From<Vec<Menu>> for Menu {
    fn from(value: Vec<Menu>) -> Self {
        Menu {
            menu_content: value.iter().flat_map(|m| m.menu_content.clone()).collect(),
        }
    }
}

impl Menu {
    pub fn from_entries<T: Into<String>, F: Fn(MenuEvent) + 'static>(
        entries: Vec<(char, T, F)>,
    ) -> Self {
        Self {
            menu_content: entries
                .into_iter()
                .map(|(k, description, handler)| MenuItem {
                    shortcut: k,
                    display_name: description.into(),
                    handler: Rc::new(Box::new(handler)),
                })
                .collect(),
        }
    }

    pub fn append(&mut self, mut other: Self) {
        self.menu_content.append(&mut other.menu_content);
    }

    pub fn handle_event(&mut self, alert_manager: &mut AlertManager, event: &InputEvent) {
        if let InputEvent::Key(key) = event {
            for menu_item in &self.menu_content {
                if menu_item.shortcut == *key {
                    (menu_item.handler)(MenuEvent {
                        alerts: alert_manager,
                    })
                }
            }
        }
    }
}
