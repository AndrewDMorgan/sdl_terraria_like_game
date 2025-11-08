
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ItemType {
    Block (usize),
    Tool (ToolType),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ToolType {
    Breaker (Vec<usize>),
    Attacker (),
}

#[derive(serde::Serialize, serde::Deserialize)]
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

