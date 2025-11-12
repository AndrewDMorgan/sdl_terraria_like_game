use crate::game_manager::{entities::{manager::{EntityManager, ItemDrop}, player::{font_rendering::render_font_unifont, items::{Item, ItemGenerator, ItemType, ToolType}, player::{KeyBindings, PlayerData}, player_ui::PlayerUiManager}}, game::GameError, world::tile_map};
use crate::core::{event_handling::event_handler::EventHandler, rendering::ui::{UiElement, UiError}, timer::Timer};
use std::rc::Rc;

use rand::Rng;

pub static TILE_DROPS: &[TileDrop] = &[
    TileDrop {
        parent_tile: &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 47, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 46],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 5, Some(ItemType::Block(1)), "Dirt", 512)],
        droped_textures: &[7],
    },
    TileDrop {
        parent_tile: &[30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 6, Some(ItemType::Block(45)), "Stone", 512)],
        droped_textures: &[8],
    },
    TileDrop {
        parent_tile: &[88],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 7, Some(ItemType::Block(88)), "Light", 512)],
        droped_textures: &[9],
    },
    TileDrop {
        parent_tile: &[173],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 8, Some(ItemType::Block(173)), "Torch", 512)],
        droped_textures: &[10],
    },
    TileDrop {
        parent_tile: &[137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 135],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 9, Some(ItemType::Block(137)), "Snow", 512)],
        droped_textures: &[11],
    },
    TileDrop {
        parent_tile: &[118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 136],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 10, Some(ItemType::Block(118)), "Ice", 512)],
        droped_textures: &[12],
    },
    TileDrop {
        parent_tile: &[89 , 90 , 91 , 92 , 93 , 94 , 95 , 96 , 97 , 98 , 99 , 100, 101, 102, 134],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 11, Some(ItemType::Block(89)), "Sand", 512)],
        droped_textures: &[13],
    },
    TileDrop {
        parent_tile: &[103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 133],
        drop_chances: &[1.0],
        drops_items: &[ItemGenerator::new((1, 1), 12, Some(ItemType::Block(103)), "Sandstone", 512)],
        droped_textures: &[14],
    },
];

pub struct TileDrop {
    parent_tile: &'static [u32],
    drop_chances: &'static [f32],
    drops_items: &'static [ItemGenerator],  // the id of the item (could really be any u32, hopefully this won't require sequential ids so that in theory additions are easy and safe)
    droped_textures: &'static [u32],
}

impl TileDrop {
    pub const fn new(parent_tile: &'static [u32], drop_chances: &'static [f32], drops_items: &'static [ItemGenerator], droped_textures: &'static [u32]) -> Self {
        Self { parent_tile, drop_chances, drops_items, droped_textures }
    }

    // (item, texture id)
    pub fn get_dropped_tile_info(&self, rand_state: &mut dyn rand::RngCore) -> Vec<(Item, u32)> {
        let mut dropped_tiles = Vec::new();
        for (i, chance) in self.drop_chances.iter().enumerate() {
            if rand_state.random_range::<f32, _>(0.0..1.0) < *chance {
                dropped_tiles.push((self.drops_items[i].generate_new(rand_state), self.droped_textures[i]));
            }
        } dropped_tiles
    }
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct Inventory {
    // the user's inventory is 10 wide for hotbar, with the inventory being 4 rows of 10
    selected_item: usize,
    hot_bar: [Option<Item>; 10],
    inventory: [[Option<Item>; 10]; 4],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            selected_item: 0,
            hot_bar: {
                let mut items: [Option<Item>; 10] = [const { None }; 10];
                items[0] = Some(Item::new(1, Some(ItemType::Tool(ToolType::Attacker())), String::from("Attack"), 1, 1));
                items[1] = Some(Item::new(2, Some(ItemType::Tool(ToolType::Breaker(vec![]))), String::from("Break"), 2, 1));
                items[2] = Some(Item::new(3, Some(ItemType::Block(1)), String::from("Build"), 1, 1));
                items[3] = Some(Item::new(4, Some(ItemType::Block(88)), String::from("Light"), 128, 1));
                items[4] = Some(Item::new(4, Some(ItemType::Block(173)), String::from("Torch"), 64, 1));
                items
            },
            inventory: {
                 [[const { None }; 10], [const { None }; 10], [const { None }; 10], [const { None }; 10]]
            },
        }
    }

    fn clicked_inventory(&self, mouse_position: (u32, u32), inventory_open: bool) -> bool {
        if inventory_open { mouse_position.0 >= 25 && mouse_position.0 <= 40 * 10 + 50 && mouse_position.1 <= 50 * 4 + 150 }
        else { mouse_position.0 >= 25 && mouse_position.0 <= 40 * 10 + 50 && mouse_position.1 <= 80 }
    }
    
    pub fn left_click_item(
        &mut self,
        tile_x: usize,
        tile_y: usize,
        tile_map: &mut tile_map::TileMap,
        event_handler: &EventHandler,
        ui_manager: &mut PlayerUiManager,
        entity_manager: &mut EntityManager,
        rand_state: &mut dyn rand::RngCore
    ) -> Result<(), GameError> {
        let inventory_open = ui_manager.ui_elements.iter().any(|e| e.identifier == "Inventory");
        if self.clicked_inventory(event_handler.mouse.position, inventory_open) { return Ok(()); }
        match & self.hot_bar[self.selected_item] {
            Some(Item { item_type: Some(ItemType::Tool(ToolType::Breaker(_can_break))), .. }) => {
                let tile = tile_map.get_tile(tile_x, tile_y, 0);
                let tile_texture_id = TILE_DROPS.iter().find(|tile_drop| tile_drop.parent_tile.iter().any(|t| *t == tile));
                if let Some(tile) = tile_texture_id {
                    let drops = tile.get_dropped_tile_info(rand_state);
                    for drop in drops {
                        entity_manager.new_drop(ItemDrop::Tile(drop.1, drop.0), ((tile_x + 1) * 8 + 2) as u32, ((tile_y + 1) * 8 + 2) as u32);
                    }
                }
                tile_map.change_tile(tile_x, tile_y, 0, 0)?;
            },
            _ => {},
        }
        Ok(())
    }

    pub fn right_click_item(
        &mut self,
        tile_x: usize,
        tile_y: usize,
        tile_map: &mut tile_map::TileMap,
        event_handler: &EventHandler,
        ui_manager: &mut PlayerUiManager,
        _entity_manager: &mut EntityManager
    ) -> Result<(), GameError> {
        let inventory_open = ui_manager.ui_elements.iter().any(|e| e.identifier == "Inventory");
        if self.clicked_inventory(event_handler.mouse.position, inventory_open) { return Ok(()); }
        match self.hot_bar[self.selected_item] {
            Some(Item { item_type: Some(ItemType::Block(id)), .. }) => {
                tile_map.change_tile(tile_x, tile_y, 0, id as u32)?;
            },
            _ => {},
        }
        Ok(())
    }

    pub fn add_item(&mut self, item: ItemDrop) -> Option<ItemDrop> {
        // checking if it's somewhere in the inventory already
        if !self.inventory.iter_mut().any(|row| {
            row.iter_mut().any(|slot_item| {
                match &item {
                    ItemDrop::Tile(_, item) => {
                        match slot_item {
                            Some( Item {
                                item_type: Some(ItemType::Block(block_id)),
                                item_count,
                                max_item_count,
                                ..
                            } ) if *item_count < *max_item_count && match &item.item_type {
                                Some(item_type) => {
                                    item_type == &ItemType::Block(*block_id)
                                },
                                _ => false,
                            } => {
                                *item_count += 1;
                                true
                            },
                            _ => { false },
                        }
                    },
                    _ => { false }
                }
            })
        }) {
            if self.inventory.iter_mut().any(|row| {
                row.iter_mut().any(|slot_item| {
                    if slot_item.is_none() {
                        match &item {
                            ItemDrop::Tile(_texture_id, item) => {
                                *slot_item = Some(item.clone());
                                true
                            }
                        }
                    } else { false }
                })
            }) {
                return None;
            } return Some(item);
        } return None;
    }

    pub fn update_key_events(
        &mut self,
        timer: &Timer,
        event_handler: &EventHandler,
        tile_map: &mut tile_map::TileMap,
        screen_size: (u32, u32),
        key_bindings: &super::player::KeyBindings,
        ui_manager: &mut PlayerUiManager,
        entity_manager: &mut EntityManager,
        player_position: &(f32, f32),
    ) -> Result<(), GameError> {
        for (index, key) in [
            sdl2::keyboard::Keycode::NUM_1, sdl2::keyboard::Keycode::NUM_2,
            sdl2::keyboard::Keycode::NUM_3, sdl2::keyboard::Keycode::NUM_4,
            sdl2::keyboard::Keycode::NUM_5, sdl2::keyboard::Keycode::NUM_6,
            sdl2::keyboard::Keycode::NUM_7, sdl2::keyboard::Keycode::NUM_8,
            sdl2::keyboard::Keycode::NUM_9, sdl2::keyboard::Keycode::NUM_0,
        ].iter().enumerate() {
            if event_handler.keys_pressed.contains(key) {
                self.selected_item = index;
            }
        }

        let mut i = 0;
        while i < entity_manager.drops.len() {
            let (_drop, pos_x, pos_y, alive_duration) = &mut entity_manager.drops[i];
            let dif_x = *pos_x as f32 - player_position.0;
            let dif_y = *pos_y as f32 - player_position.1;
            *alive_duration -= timer.delta_time;
            if *alive_duration <= 0.0 {
                entity_manager.drops.remove(i);  // despawned
                continue;
            }
            if dif_x*dif_x + dif_y*dif_y < 100.0 {
                let (drop, p1, p2, p3) = entity_manager.drops.remove(i);
                let item = self.add_item(drop);
                if let Some(item) = item {
                    entity_manager.drops.insert(i, (item, p1, p2, p3));
                    i += 1;
                }
                continue;
            }
            i += 1;
        }

        let raw_keys_down = event_handler.keys_pressed.iter().map(|k| **k).collect::<Vec<_>>();
        if KeyBindings::check_true(&key_bindings.inventory, &raw_keys_down, &event_handler.mods_pressed) {
            // Checking if the inventory is or isn't open
            let inventory_open = ui_manager.ui_elements.iter().any(|e| e.identifier == "Inventory");
            match inventory_open {
                true => {
                    ui_manager.ui_elements.retain(|e| e.identifier != "Inventory");
                },
                false => {
                    // opening a new inventory ui element
                    ui_manager.ui_elements.push(
                        generate_player_inventory_ui_element(
                            ui_manager.item_textures.clone(), ui_manager.text_character_atlas.clone()
                        )
                    );
                }
            }
        }

        Ok(())
    }
}

// it may be more efficent to store one instance, and just swap it between the active vector, and some sort of storage location, but idk, that sounds like work, and it should be fast enough as is
pub fn generate_player_inventory_ui_element(item_textures: Rc<Vec<[u32; 256]>>, font_atlas: Rc<Vec<[u32; 256]>>) -> UiElement<PlayerData> {
    UiElement::new(String::from("Inventory"), (50, 150), (400, 200), 
        Box::new(move |pixels, (window_size, pitch), data, (pos, size)| {
            let item_textures = item_textures.clone();
            let font_atlas = font_atlas.clone();
            for i in 0..4 {
                render_item_row(item_textures.clone(),
                                font_atlas.clone(),
                                &data.inventory.inventory[i],
                                pixels,
                                (window_size, pitch),
                                ((pos.0, pos.1 + i * 50), size),
                                None
                )?;
            }
            Ok(())  // todo! do things here
        }
    ))
}

pub fn generate_player_hotbar_ui_element(item_textures: Rc<Vec<[u32; 256]>>, font_atlas: Rc<Vec<[u32; 256]>>) -> UiElement<PlayerData> {
    UiElement::new(String::from("Hotbar"), (25, 25), (400, 40), 
        Box::new(move |pixels, (window_size, pitch), data, (pos, _size)| {
            let item_textures = item_textures.clone();
            let font_atlas = font_atlas.clone();

            render_item_row(item_textures,
                            font_atlas,
                            &data.inventory.hot_bar,
                            pixels,
                            (window_size, pitch),
                            (pos, _size),
                            Some(data.inventory.selected_item)
            )?;

            Ok(())
        }
    ))
}

// renders a row of items (an example being the hotbar)
// by abstracting this out, hopefully rendering the inventory will be slightly easier
fn render_item_row(
    item_textures: Rc<Vec<[u32; 256]>>,
    font_atlas: Rc<Vec<[u32; 256]>>,
    items: &[Option<Item>; 10],
    pixels: &mut [u8],
    (window_size, pitch): ((u32, u32), usize),
    (pos, _size): ((usize, usize), (usize, usize)),
    selected: Option<usize>,
) -> Result<(), UiError> {
    // going item by item
    for item_index in 0..10 {
        let pixel_position = (pos.0 + item_index * 40, pos.1);

        // drawing the backing (needs improved visuals, but I'm lazy rn and have homework due soon)
        let color = if selected == Some(item_index) { (200, 200, 200) } else { (100, 100, 100) };
        for x in pixel_position.0..pixel_position.0 + 40 {
            for y in pixel_position.1..pixel_position.1 + 46 {
                let pixel_index = x * 3 + y * pitch as usize;
                pixels[pixel_index] = color.0;
                pixels[pixel_index + 1] = color.1;
                pixels[pixel_index + 2] = color.2;
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

