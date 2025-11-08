
float lerp(float left, float right, float alpha) {
    return left * (1.0 - alpha) + right * alpha;
}

float3 lerp_f3(float3 left, float3 right, float alpha) {
    return left * (1.0 - alpha) + right * alpha;
}

float3 ToColor(ulong value) {
    return float3(
        ((value      ) & 0xFF) * 0.00392156862,
        ((value >> 8 ) & 0xFF) * 0.00392156862,
        ((value >> 16) & 0xFF) * 0.00392156862
    );
}

struct Text {
    uchar characters[32];
    ulong2 info;
};

kernel void ComputeShader (
    constant uint&   pitch             [[ buffer(0 ) ]],  // from sdl2 for padding
    constant uint&   width             [[ buffer(1 ) ]],  // width of screen
    constant uint&   height            [[ buffer(2 ) ]],  // height of screen

    constant uchar4* entity_textures   [[ buffer(3 ) ]],  // texture data for entities
    constant uchar4* tile_textures     [[ buffer(4 ) ]],  // texture data for tiles
    constant uchar4* particle_textures [[ buffer(5 ) ]],  // texture data for particles

    constant uint&   tile_map_width    [[ buffer(6 ) ]],  // size of the tile map in tiles (width, height)
    constant uint&   tile_map_height   [[ buffer(7 ) ]],  // size of the tile map in tiles (width, height)
    constant ulong*  tile_map          [[ buffer(8 ) ]],  // tile map data (screen space, not all global tiles)
    constant float4& camera_position   [[ buffer(9 ) ]],  // camera position (screen space offset) and scale and rotation

    constant uint&   num_entities      [[ buffer(10) ]],  // number of entities
    constant ulong2* entity_data       [[ buffer(11) ]],  // entity data

    constant uint&   num_particles     [[ buffer(12) ]],  // number of particles
    constant ulong2* particle_data     [[ buffer(13) ]],  // particle data

    constant uint&   num_texts         [[ buffer(14) ]],  // number of text entries
    constant Text*   text_buffer       [[ buffer(15) ]],  // text data

    constant bool*   texture_atlas     [[ buffer(16) ]],  // font
    constant uint&   default_font_size [[ buffer(17) ]],  // font size

    device   uchar* pixels [[ buffer(18) ]],
    uint2 gid [[ thread_position_in_grid ]]
) {
    if (gid.x >= width || gid.y >= height) return;

    // this is gonna get kinda crazy.... but this will have to render ui, text, entities, and tiles.....  and even particles....
    //    the tile map will have 3 layers: background (walls like in terraria), tiles, and annything forground
    //      related (mostly to make sure the layer is reserved incase it's later needed)    plus one layer for lighting (rgb strength)
    //    tiles are a 64 bit value, stored in an array buffer, with the first 32 being the tile id (aka texture index),
    //      and the others being tile data (would very by tile probably)
    //    
    //    the entities are a 128 bit value, with the first 32 being the texture id (similar to tiles),
    //      the next 16 being rotation ( f16 of [0, pi) ), and 16 for x + 16 for y (screen space offsets, uint), with 44 bits for applicable data, and the
    //      the 4 being depth (to correctly layer them; hopefully 4 bits is enough, but idk)
    //    
    //    the particles are represented by a 128 bit value:
    //        32 -> texture id
    //        16 -> rotation (f16 of [0, pi) )
    //        16 -> x offset (screen space, uint)
    //        16 -> y offset (screen space, uint)
    //        32 -> lighting (r, g, b being 8, 8, 8, and a being 8 for strength or alpha)
    //        12 -> applicable data
    //        4  -> depth (to correctly layer them; hopefully 4 bits is enough, but idk)
    //    
    //    text buffer for rendering text is a 128 bit value and a buffer of 32 u8:
    //        16 -> x offset (screen space; top left of text, uint)
    //        16 -> y offset (screen space; top left of text, uint)
    //        16 -> rotation
    //        32 -> color (r, g, b each being 8, 8, 8, 8 for alpha)
    //        32 -> applicable data (not sure what would go here yet, but it's reserved anyways, so use if needed)
    //        8 bits for font size
    //        8 bits for the length of the character buffer
    //    
    //    the ui is gonna be different, and probably painful....
    //      I might make multiple shaders for dedicated menu, but game ui would still need to be here
    //        (at least it will be less variable).
    //        -- however, only one shader should be called each frame, so on overlapping ui like in gameplay,
    //           this shader has to do it all; sending data to and from the gpu multiple times is a terrible idea for performance
    //      I might pass in a struct with uint flags representing the ui state, or maybe parse the inventory/ui into a
    //        less variable format that's easier to render (the performance of the ui really only comes from text and rendering, not state tracking or parsing)

    // todo! add the camera offset transform and stuff ig

    // how in the world has this managed to not only work but to not segfault?
    // how is the memory actually aligned right on my first attempt???
    float inv_zoom = camera_position.z;
    float3 color = float3(0.8, 0.8, 0.9);
    float2 gid_f = float2(gid.x, gid.y);
    // making sure the position isn't outside the tilemap
    float3 light_color = float3(1.0, 1.0, 1.0);
    float2 position_float = float2(gid.x - camera_position.x, gid.y - camera_position.y);
    uint2 position = uint2(uint(position_float.x), uint(position_float.y));
    if (camera_position.x <= gid.x && camera_position.y <= gid.y) {
        // getting the corrected screen space position
        float px_zoomed = position.x * inv_zoom;
        float py_zoomed = position.y * inv_zoom;
        float px_fract = px_zoomed * 0.125;  // 1 / 8
        float py_fract = py_zoomed * 0.125;  // 1 / 8
        uint x_coord = metal::floor(px_fract) + 1;
        uint y_coord = metal::floor(py_fract) + 1;
        if (x_coord < tile_map_width && y_coord < tile_map_height) {
            uint tile_index = x_coord + y_coord * tile_map_width;
            uint offset = (uint(px_zoomed) % 8) + (uint(py_zoomed) % 8) * 8;
            
            // interpolating the light (if defined)
            #define INTERPOLATE_LIGHT
            #ifdef INTERPOLATE_LIGHT
                float3 top_left     = ToColor(tile_map[(x_coord + y_coord * tile_map_width) * 4 + 3]);
                float3 top_right    = ToColor(tile_map[(x_coord + 1 + y_coord * tile_map_width) * 4 + 3]);
                float3 bottom_left  = ToColor(tile_map[(x_coord + (y_coord + 1) * tile_map_width) * 4 + 3]);
                float3 bottom_right = ToColor(tile_map[(x_coord + 1 + (y_coord + 1) * tile_map_width) * 4 + 3]);
                float interp_x = metal::pow(metal::fract(px_fract), 2.0);
                float interp_y = metal::pow(metal::fract(py_fract), 2.0);
                float3 light_color_1 = lerp_f3(
                    lerp_f3(top_left, top_right, interp_x),
                    lerp_f3(bottom_left, bottom_right, interp_x),
                    interp_y
                );

                // interpolating the light
                top_right    = ToColor(tile_map[(x_coord - 1 + y_coord * tile_map_width) * 4 + 3]);
                bottom_left  = ToColor(tile_map[(x_coord + (y_coord - 1) * tile_map_width) * 4 + 3]);
                bottom_right = ToColor(tile_map[(x_coord - 1 + (y_coord - 1) * tile_map_width) * 4 + 3]);
                interp_x = metal::pow(1.0 - metal::fract(px_fract), 2.0);
                interp_y = metal::pow(1.0 - metal::fract(py_fract), 2.0);
                float3 light_color_2 = lerp_f3(
                    lerp_f3(top_left, top_right, interp_x),
                    lerp_f3(bottom_left, bottom_right, interp_x),
                    interp_y
                );
                light_color = (light_color_1 + light_color_2) * 0.5;
                color *= light_color;
            #else
                light_color = ToColor(tile_map[(x_coord + y_coord * tile_map_width) * 4 + 3]);
                color *= light_color;
            #endif
            
            // going through the 3 layers ( the first is the forground )
            for (int i = 2; i >= 0; i--) {
                uint tile_value = tile_map[tile_index * 4 + i];
                // casting tile_value to uint from ulong should just cut off the extra bits of info, which isn't necessary here at least for now
                uint tile_text_index = tile_value * 64 + offset;
                float alpha = tile_textures[tile_text_index].w * 0.00392156862;
                color = float3(
                    lerp(color.x, tile_textures[tile_text_index].x * 0.00392156862 * light_color.x, alpha),
                    lerp(color.y, tile_textures[tile_text_index].y * 0.00392156862 * light_color.y, alpha),
                    lerp(color.z, tile_textures[tile_text_index].z * 0.00392156862 * light_color.z, alpha)
                );
            }
        }
    }
    
    // rendering entities
    // is this slow? YES   does it work? for now
    // todo! optimize this (possibly move it to the cpu end to prevent the O(screen * entities))
    //     the extra overhead of doing per pixel far outweighs the gpu's power
    // actually, this works really well, and isn't too bad
    float2 half_size = float2(width, height) * 0.5;
    float2 camera_position_corrected = float2(metal::round((gid_f.x - half_size.x) * inv_zoom * 8.0) * 0.125, metal::round((gid_f.y - half_size.y) * inv_zoom * 8.0) * 0.125);
    for (uint i = 0; i < num_entities; i++) {
        ulong2 entity = entity_data[i];
        //uint rotation   =(entity.y >> 16) & 0xFFFF;
        // the 0.01 scales it correctly to remove the error caused by limited bits
        float offset_x    = float(short(ushort(entity.y))) * 0.01;  // using &0xFFFF shouldn't be needed as the cast already does it implicitly
        //uint depth      = (entity.x >> 0 ) & 0xF;

        // hopefully this is faster having them broken up by reducing the average number of instructions used
        // but idk, cause it'll probably also add other instructions in doing so and the offset calculations
        // are fairly cheap
        if (camera_position_corrected.x < offset_x || camera_position_corrected.x >= offset_x + 8.0) {
            continue;
        }
        float offset_y    = float(short(ushort(entity.x >> 48))) * 0.01;
        if (camera_position_corrected.y < offset_y || camera_position_corrected.y >= offset_y + 8.0) {
            continue;
        }

        uint texture_id = entity.y >> 32;
        uint index_offset = uint(camera_position_corrected.x - offset_x) + uint(camera_position_corrected.y - offset_y) * 8;
        uchar4 texture_color = entity_textures[texture_id * 64 + index_offset];
        float alpha = texture_color.w * 0.00392156862;
        color = float3(
            lerp(color.x, texture_color.x * 0.00392156862 * light_color.x, alpha),
            lerp(color.y, texture_color.y * 0.00392156862 * light_color.y, alpha),
            lerp(color.z, texture_color.z * 0.00392156862 * light_color.z, alpha)
        );
    }

    // drawing text
    for (uint i = 0; i < num_texts; i++) {
        Text text = text_buffer[i];
        // the casts should implicitly cut any bits beyond 8 off (so & 0xFF shouldn't be needed)
        uchar buffer_size = text.info.x;
        uchar font_size   = text.info.x >> 8;
        ushort x_offset   = text.info.y >> 48;
        ushort y_offset   = text.info.y >> 32;
        // checking the bounds
        if (gid.x < x_offset || gid.x >= x_offset + buffer_size * (font_size + 2) ||
            gid.y < y_offset || gid.y >= y_offset + font_size)
        {
            continue;
        }
        ushort pixel_x_coord = (gid.x - x_offset) % (font_size + 2);
        if (pixel_x_coord >= font_size) {
            continue;  // a spacing gap
        }

        uchar4 text_color = uchar4(
            uchar(text.info.y >> 8),
            uchar(text.info.y),
            uchar(text.info.x >> 56),
            uchar(text.info.x >> 48)
        );
        // now drawing the text........ this is gonna be annoying
        // which text drawing algorithm shall we choose? None? Welp, apparently that's not a valid algorithm :(
        ushort x_coord = float(pixel_x_coord) / float(font_size) * default_font_size;
        ushort y_coord = (gid_f.y - y_offset) / float(font_size) * default_font_size;
        uchar character = text.characters[(gid.x - x_offset) / (font_size + 2)];
        uint texture_index = x_coord + y_coord * default_font_size + character * default_font_size * default_font_size;
        bool atlas_lookup = texture_atlas[texture_index];
        if (!atlas_lookup) {
            continue;
        }
        color = float3(
            text_color.x * 0.00392156862,
            text_color.y * 0.00392156862,
            text_color.z * 0.00392156862
        );
    }

    uint index = gid.y * pitch + gid.x * 3;

    /*if (gid.x == width / 2) {
        color = float3(0., 0., 0.);
    }*/

    pixels[index + 0] = uchar(color.x * 255.0); // R
    pixels[index + 1] = uchar(color.y * 255.0); // G
    pixels[index + 2] = uchar(color.z * 255.0); // B
}

