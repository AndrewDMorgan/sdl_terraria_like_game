
pub fn render_font_unifont<const FONT_SIZE: u32, const FONT_SIZE_SQUARE: usize, const FONT_SPACING: usize>(
    atlas: &Vec<[u32; FONT_SIZE_SQUARE]>,
    pixels: &mut [u8],
    position: (usize, usize),
    _screen_size: (u32, u32),
    pitch: usize,
    text: &str,
) {
    // converting the text to unifont or whatever
    let coords = text
        .chars()
        .map(|ch| {
            if ch == ' ' { 0 }  // fixes an odd bug
            else { ch as u32 }
        })
        .collect::<Vec<_>>();

    for (row_index, code) in coords.iter().enumerate() {
        let texture = &atlas[*code as usize];
        for pixel_x in 0..FONT_SIZE {
            for pixel_y in 0..FONT_SIZE {
                let index = (pixel_x + pixel_y * FONT_SIZE) as usize;
                let color = texture[index];
                let a = ((color >> 24) & 0xFF) as u8;
                if a == 0 { continue; }
                let b = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let r = (color & 0xFF) as u8;
                let pixel_index = (position.0 + pixel_x as usize + row_index * FONT_SPACING) * 3 + (position.1 + pixel_y as usize) * pitch as usize;
                pixels[pixel_index] = lerp(r, pixels[pixel_index], a as f32 / 255.0);
                pixels[pixel_index + 1] = lerp(g, pixels[pixel_index + 1], a as f32 / 255.0);
                pixels[pixel_index + 2] = lerp(b, pixels[pixel_index + 2], a as f32 / 255.0);
            }
        }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    let (a, b) = (b as f32, a as f32);
    let result = a + (b - a) * t;
    result.clamp(0.0, 255.0) as u8
}

