
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ItemType {
    //
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub item_type: Option<ItemType>,
    pub item_count: usize,
    pub texture_id: usize,
    pub name: String,
}

impl Item {
    pub fn new(texture_id: usize, name: String, count: usize) -> Self {
        Item {
            item_type: None,
            item_count: count,
            texture_id,
            name
        }
    }
}

