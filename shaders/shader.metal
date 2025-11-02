
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
    constant float4& camera_position   [[ buffer(9 ) ]],  // camera position (screen space offset) and rotation and scale

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

    uint index = gid.y * pitch + gid.x * 3;
    
    // the color channels are between 0 and 1
    // color = float3( float(gid.x) / float(width), float(gid.y) / float(height), 0.5);
    float3 color = float3(
        tile_textures[(gid.x / 6) % 8 + ((gid.y / 6) % 8) * 8 + 64 * 1].x / 255.0,
        tile_textures[(gid.x / 6) % 8 + ((gid.y / 6) % 8) * 8 + 64 * 1].y / 255.0,
        tile_textures[(gid.x / 6) % 8 + ((gid.y / 6) % 8) * 8 + 64 * 1].z / 255.0
    );
    if (!tile_textures[(gid.x / 6) % 8 + (gid.y / 6) % 8 * 8 + 64 * 1].w) {
        color = float3(0.8, 0.8, 0.9);
    }
    pixels[index + 0] = uchar(color.x * 255.0); // R
    pixels[index + 1] = uchar(color.y * 255.0); // G
    pixels[index + 2] = uchar(color.z * 255.0); // B
}

