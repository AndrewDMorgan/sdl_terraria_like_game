use crate::{core::rendering::ui::{UiElement, UiError}, game_manager::entities::player::{font_rendering::render_font_unifont, items::Item, player::PlayerData}};
use std::rc::Rc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Inventory {
    // the user's inventory is 10 wide for hotbar, with the inventory being 4 rows of 10
    hot_bar: [Option<Item>; 10],
    inventory: [[Option<Item>; 10]; 4],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            hot_bar: {
                let mut items: [Option<Item>; 10] = [const { None }; 10];
                items[0] = Some(Item::new(1, String::from("Attack"), 1));
                items[1] = Some(Item::new(2, String::from("Break"), 2));
                items[2] = Some(Item::new(3, String::from("Build"), 1));
                items[3] = Some(Item::new(4, String::from("Light"), 128));
                items
            },
            inventory: Default::default(),
        }
    }
}

// it may be more efficent to store one instance, and just swap it between the active vector, and some sort of storage location, but idk, that sounds like work, and it should be fast enough as is
pub fn generate_player_inventory_ui_element(item_textures: Rc<Vec<[u32; 256]>>, font_atlas: Rc<Vec<[u32; 256]>>) -> UiElement<PlayerData> {
    UiElement::new((50, 50), (400, 200), Box::new(|_pixels, (_window_size, _pitch), _data, (_pos, _size)| {
        Ok(())  // todo! do things here
    }))
}

pub fn generate_player_hotbar_ui_element(item_textures: Rc<Vec<[u32; 256]>>, font_atlas: Rc<Vec<[u32; 256]>>) -> UiElement<PlayerData> {
    UiElement::new((25, 25), (400, 40), 
        Box::new(move |pixels, (window_size, pitch), data, (pos, _size)| {
            let item_textures = item_textures.clone();
            let font_atlas = font_atlas.clone();

            render_item_row(item_textures, font_atlas, &data.inventory.hot_bar, pixels, (window_size, pitch), data, (pos, _size))?;

            Ok(())
    }))
}

// renders a row of items (an example being the hotbar)
// by abstracting this out, hopefully rendering the inventory will be slightly easier
fn render_item_row(
    item_textures: Rc<Vec<[u32; 256]>>,
    font_atlas: Rc<Vec<[u32; 256]>>,
    items: &[Option<Item>; 10],
    pixels: &mut [u8],
    (window_size, pitch): ((u32, u32), usize),
    data: &PlayerData,
    (pos, _size): ((usize, usize), (usize, usize)),
) -> Result<(), UiError> {
    // going item by item
    for item_index in 0..10 {
        let pixel_position = (pos.0 + item_index * 40, pos.1);

        // drawing the backing (needs improved visuals, but I'm lazy rn and have homework due soon)
        for x in pixel_position.0..pixel_position.0 + 40 {
            for y in pixel_position.1..pixel_position.1 + 46 {
                let pixel_index = x * 3 + y * pitch as usize;
                pixels[pixel_index] = 100;
                pixels[pixel_index + 1] = 100;
                pixels[pixel_index + 2] = 100;
            }
        }

        // drawing the actual item sprite (if there is an item)
        if let Some(item) = &items[item_index] {
            let sprite = &item_textures[item.texture_id];
            for raw_pixel_x in 0..16 {
                for raw_pixel_y in 0..16 {
                    let pixel = sprite[raw_pixel_x + raw_pixel_y * 16];
                    let pixel_alpha = ((pixel >> 24) & 0xFF) as u8;
                    if pixel_alpha == 0 { continue; }
                    let pixel_blue = ((pixel >> 16) & 0xFF) as u8;
                    let pixel_green = ((pixel >> 8) & 0xFF) as u8;
                    let pixel_red = (pixel & 0xFF) as u8;

                    // scaling it up since 16x16 pixels is tiny
                    // a factor of 2 seems to work well
                    const PIXEL_SCALE_UP: usize = 2;
                    // can you tell what my opinion on never nesters is?
                    for offset_x in 0..PIXEL_SCALE_UP { for offset_y in 0..PIXEL_SCALE_UP {
                        let pixel_x = (raw_pixel_x * PIXEL_SCALE_UP + offset_x) + (pixel_position.0 + 4);  // the 4 is the padding from the edges of the cell
                        let pixel_y = (raw_pixel_y * PIXEL_SCALE_UP + offset_y) + (pixel_position.1 + 4);  // the 4 is the padding from the edges of the cell
                        pixels[pixel_x * 3 + pixel_y * pitch as usize] = pixel_red;
                        pixels[pixel_x * 3 + pixel_y * pitch as usize + 1] = pixel_green;
                        pixels[pixel_x * 3 + pixel_y * pitch as usize + 2] = pixel_blue;
                    }}
                }
            }
        }
    }

    // rendering text on top of EVERYTHING so it doesn't get covered
    for item_index in 0..10 {
        let pixel_position = (pos.0 + item_index * 40, pos.1);
        if let Some(item) = &items[item_index] {
            // drawing the item count
            render_font_unifont::<16, 256, 8>(
                &*font_atlas, pixels, (pixel_position.0 + 12, pixel_position.1 + 29), window_size, pitch, &format!("x{}", item.item_count)
            );
        }
    } Ok(())
}

