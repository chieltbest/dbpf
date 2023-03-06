#pragma endian little

enum IndexType: u32 {
    V1 = 1,
    V2 = 2,
};

struct Entry<auto index_type> {
    u32 type;
    u32 group;
    u32 instance;
    if (index_type == IndexType::V2) {
        u32 resource;
    }
};

struct SimOutfits {
    u32 id;
    IndexType type;
    u32 count;
    
    Entry<type> entries[count];
};

SimOutfits file @ 0x00;
