use std::{collections::HashMap, hash::Hash};

pub struct CyclicList<T> {
    elements: Vec<T>,
    currently_selected: usize,
}

impl<T> CyclicList<T> {
    pub fn new(elements: Vec<T>) -> Self {
        Self {
            elements,
            currently_selected: 0,
        }
    }
    pub fn move_next(&mut self) {
        if self.currently_selected + 1 >= self.elements.len() {
            self.currently_selected = 0;
        } else {
            self.currently_selected += 1;
        }
    }

    pub fn move_previous(&mut self) {
        if self.currently_selected == 0 {
            self.currently_selected = self.elements.len() - 1;
        } else {
            self.currently_selected -= 1;
        }
    }

    pub fn current(&self) -> Option<&T> {
        self.elements.get(self.currently_selected)
    }

    pub fn reset(&mut self) {
        self.currently_selected = 0;
    }
}

pub struct SelectableHashMap<K: Eq + PartialEq + Hash, V> {
    contents: HashMap<K, V>,
    current: K,
    default: K,
}

#[allow(dead_code)]
impl<K: Eq + PartialEq + Hash + Copy, V> SelectableHashMap<K, V> {
    /// Create a new selectable hash map using a key as the default selected element
    pub fn new(selected: K, contents: HashMap<K, V>) -> Self {
        Self {
            contents,
            current: selected,
            default: selected,
        }
    }

    pub fn get_current(&self) -> Option<&V> {
        self.contents.get(&self.current)
    }

    pub fn get_current_mut(&mut self) -> Option<&mut V> {
        self.contents.get_mut(&self.current)
    }

    pub fn set_current(&mut self, key: K) {
        // TODO: Validate the key exists and return a Result
        self.current = key;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.contents.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.contents.get_mut(key)
    }

    /// Sets the currently selected element to the default one
    /// provided during creation
    pub fn reset(&mut self) {
        self.current = self.default
    }
}
