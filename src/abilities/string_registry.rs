use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct StringRegistry<T> {
    name_to_id: HashMap<String, T>,
    id_to_name: Vec<String>,
}

impl<T: From<u32> + Copy> StringRegistry<T> {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }

    pub fn get_or_insert(&mut self, name: &str) -> T {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        let id = T::from(self.id_to_name.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        id
    }

    pub fn get(&self, name: &str) -> Option<T> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_name(&self, id: T) -> Option<&str>
    where
        T: Into<u32>,
    {
        let index = id.into() as usize;
        self.id_to_name.get(index).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.id_to_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id_to_name.is_empty()
    }
}

impl<T: From<u32> + Copy + Into<u32>> StringRegistry<T> {
    pub fn get_name_or_unknown(&self, id: T) -> &str {
        self.get_name(id).unwrap_or("<unknown>")
    }
}
