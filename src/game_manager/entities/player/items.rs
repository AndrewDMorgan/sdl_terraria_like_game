use rand::Rng;


#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum ItemType {
    Block (usize),
    Tool (ToolType),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum ToolType {
    Breaker (Vec<usize>),
    Attacker (),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Item {
    pub item_type: Option<ItemType>,
    pub item_count: usize,
    pub texture_id: usize,
    pub name: String,
}

impl Item {
    pub fn new(texture_id: usize, item_type: Option<ItemType>, name: String, count: usize) -> Self {
        Item {
            item_type,
            item_count: count,
            texture_id,
            name
        }
    }
}

#[derive(Clone)]
pub struct ItemGenerator {
    count_range: (usize, usize),
    texture_id: usize,
    item_type: Option<ItemType>,
    name: &'static str,
}

impl ItemGenerator {
    pub const fn new(count_range: (usize, usize), texture_id: usize, item_type: Option<ItemType>, name: &'static str) -> Self {
        Self { count_range, texture_id, item_type, name }
    }

    pub fn generate_new(&self, rand_state: &mut dyn rand::RngCore) -> Item {
        let count = rand_state.random_range(self.count_range.0..=self.count_range.1);
        Item::new(self.texture_id, self.item_type.clone(), self.name.to_string(), count)
    }
}


