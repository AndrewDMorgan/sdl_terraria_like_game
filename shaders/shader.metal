
float lerp(float left, float right, float alpha) {
    return left * (1.0 - alpha) + right * alpha;
}


struct Text {
    ulong2 info;
    uchar characters[32];
};

kernel void ComputeShader (
    constant uint&   pitch             [[ buffer(0 ) ]],
    constant uint&   width             [[ buffer(1 ) ]],
    constant uint&   height            [[ buffer(2 ) ]],

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

    device   uchar* pixels [[ buffer(16) ]],
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
    float2 position_float = float2(gid.x - camera_position.x, gid.y - camera_position.y);
    uint2 position = uint2(uint(position_float.x), uint(position_float.y));
    if (camera_position.x <= gid.x && camera_position.y <= gid.y) {
        // getting the corrected screen space position
        float x_coord = metal::floor(position.x * inv_zoom * 0.125);  // 1 / 8
        float y_coord = metal::floor(position.y * inv_zoom * 0.125);  // 1 / 8
        if (x_coord < tile_map_width && y_coord < tile_map_height) {
            uint tile_index = x_coord + y_coord * float(tile_map_width);
            uint offset = (uint(position.x * inv_zoom) % 8) + (uint(position.y * inv_zoom) % 8) * 8;
            
            // getting the lighting (fourth layer)
            ulong light_value = tile_map[tile_index * 4 + 3];
            float3 light_color = float3(
                ((light_value >> 0 ) & 0xFF) * 0.00392156862,
                ((light_value >> 8 ) & 0xFF) * 0.00392156862,
                ((light_value >> 16) & 0xFF) * 0.00392156862
            );
            
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
    float2 half_size = float2(width, height) * 0.5;
    float2 camera_position_corrected = float2(metal::round((gid_f.x - half_size.x) * inv_zoom * 8.0) * 0.125, metal::round((gid_f.y - half_size.y) * inv_zoom * 8.0) * 0.125);
    for (uint i = 0; i < num_entities; i++) {
        ulong2 entity = entity_data[i];
        //uint rotation   =(entity.y >> 16) & 0xFFFF;
        float offset_x    = float(short(ushort(entity.y))) * 0.01;  // using &0xFFFF shouldn't be needed as the cast already does it implicitly
        float offset_y    = float(short(ushort(entity.x >> 48))) * 0.01;
        //uint depth      = (entity.x >> 0 ) & 0xF;
        if (camera_position_corrected.x >= offset_x && camera_position_corrected.x < offset_x + 8.0 &&
            camera_position_corrected.y >= offset_y && camera_position_corrected.y < offset_y + 8.0
        ) {
            uint texture_id = entity.y >> 32;
            uint index_offset = uint(camera_position_corrected.x - offset_x) + uint(camera_position_corrected.y - offset_y) * 8;
            uchar4 texture_color = entity_textures[texture_id * 64 + index_offset];
            float alpha = texture_color.w * 0.00392156862;
            color = float3(
                lerp(color.x, texture_color.x * 0.00392156862, alpha),
                lerp(color.y, texture_color.y * 0.00392156862, alpha),
                lerp(color.z, texture_color.z * 0.00392156862, alpha)
            );
        }
    }

    uint index = gid.y * pitch + gid.x * 3;

    pixels[index + 0] = uchar(color.x * 255.0); // R
    pixels[index + 1] = uchar(color.y * 255.0); // G
    pixels[index + 2] = uchar(color.z * 255.0); // B
}

