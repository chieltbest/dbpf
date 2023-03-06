#pragma endian little

bitfield OffsetX {
    offset: 12;
    msb: 4;
};

bitfield OffsetY {
    offset: 12;
    mipmaps: 4;
};

struct FSHEntryHeader {
    u8 record_id;
    u24 block_size;
    u16 width;
    u16 height;
    
    u16 rot_center_x;
    u16 rot_center_y;
    
    OffsetX offset_x;
    OffsetY offset_y;
    
    u8 data[block_size];
};

struct DirectoryEntry {
    char name[4];
    u32 offset;
    
    FSHEntryHeader header @ offset;
};

struct FranksShapeFile {
    char magic[4];
    u32 file_size;
    u32 num_entries;
    u32 dir_id;
    
    DirectoryEntry entries[num_entries];
};

FranksShapeFile file @ 0x00;
