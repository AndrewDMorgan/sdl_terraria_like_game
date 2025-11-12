use rand::Rng;

#[derive(bincode::Encode, bincode::Decode, Clone, PartialEq)]
pub enum ItemType {
    Block (usize),
    Tool (ToolType),
}

#[derive(bincode::Encode, bincode::Decode, Clone, PartialEq)]
pub enum ToolType {
    Breaker (Vec<usize>),
    Attacker (),
}

#[derive(bincode::Encode, bincode::Decode, Clone)]
pub struct Item {
    pub item_type: Option<ItemType>,
    pub item_count: usize,
    pub texture_id: usize,
    pub name: String,
    pub max_item_count: usize,
}

impl Item {
    pub fn new(texture_id: usize, item_type: Option<ItemType>, name: String, count: usize, max_item_count: usize) -> Self {
        Item {
            item_type,
            item_count: count,
            texture_id,
            name,
            max_item_count,
        }
    }
}

#[derive(Clone)]
pub struct ItemGenerator {
    count_range: (usize, usize),
    texture_id: usize,
    item_type: Option<ItemType>,
    name: &'static str,
    max_item_count: usize,
}

impl ItemGenerator {
    pub const fn new(count_range: (usize, usize), texture_id: usize, item_type: Option<ItemType>, name: &'static str, max_item_count: usize) -> Self {
        Self { count_range, texture_id, item_type, name, max_item_count }
    }

    pub fn generate_new(&self, rand_state: &mut dyn rand::RngCore) -> Item {
        let count = rand_state.random_range(self.count_range.0..=self.count_range.1);
        Item::new(self.texture_id, self.item_type.clone(), self.name.to_string(), count, self.max_item_count)
    }
}


