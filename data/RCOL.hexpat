#pragma endian little

#include <std/io.pat>

struct String {
    u8 len;
    char data[len];
};

struct FileName {
    String res;
    u32 block_id;
    u32 version;
    String name;
};

// resource definitions

enum TextureFormat : u32 {
    RawARGB = 1,
    RawRGB = 2,
    Unknown2 = 3,
    DXT1 = 4,
    DXT3 = 5,
    Grayscale = 6,
    Alt32bit = 7,
    DXT5 = 8,
    Alt24bit = 9,
};

struct TextureResourceData {
    u8 data_type;
    
    if (data_type == 0) {
        u32 data_size;
        u8 data[data_size];
    }
    if (data_type == 1) {
        String lifo_name;
    }
};

struct TextureResourceTexture<auto inner_texture_version, auto mip_count> {
    if (inner_texture_version == 7) {
        u32 loop_count = mip_count;
    } else /*if (inner_texture_version == 9)*/ {
        u32 loop_count;
    }
    
    TextureResourceData entries[loop_count];
    
    u32 creator_id;
    //if (inner_texture_version == 9) {
        u32 format_flag; // 0x14210000
    //}
};

struct TextureResource<auto texture_version> {
    FileName file_name;
    
    u32 texture_width;
    u32 texture_height;
    
    TextureFormat image_format;
    u32 mip_levels;
    float purpose;
    u32 num_textures;
    u32 unknown4;
    
    if (texture_version == 9) {
        String file_name_repeat;
    }
    
    u32 texture_version_copy = texture_version;
    
    TextureResourceTexture<texture_version_copy, mip_levels> textures[num_textures];
};

struct MaterialProperty {
    String name;
    String value;
};

struct MaterialDefinition<auto version> {
    FileName file_name;
    
    String material_description;
    String material_type;
    
    u32 property_count;
    MaterialProperty properties[property_count];
    
    if (version > 8) {
        u32 name_count;
        String names[name_count];
    }
};

// RCOL definition

struct FileLink<auto version> {
    u32 group_id;
    u32 instance_id;
    if (version) {
        u32 resource_id;
    }
    u32 type_id;
};

struct ResourceItem {
    u32 type_id;
};

struct ResourceEntry {
    String name;
    u32 type_id;
    u32 version;
    
    if (type_id == 0x1C4A276C) {
        TextureResource<version> res;
    }
    if (type_id == 0x49596978) {
        MaterialDefinition<version> res;
    }
};

struct ResourceCollection {
    u32 version;
    
    u32 link_count;
    FileLink<true> links[link_count];
    
    u32 item_count;
    ResourceItem index[item_count];
    
    ResourceEntry entries[item_count];
};

ResourceCollection file @ 0x00;