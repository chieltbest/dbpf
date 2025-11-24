// SPDX-FileCopyrightText: 2023 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma endian little

struct DBPFDirectoryEntry {
    u32 type_id;
    u32 group_id;
    u32 instance_id_low;
    u32 instance_id_high;
    u32 decompressed_size;
};

struct DBPFDirectory<auto dir_size> {
    DBPFDirectoryEntry entries[dir_size / 20];
};

struct IndexEntry {
    u32 type_id;
    u32 group_id;
    
    u32 instance_id_low;
    u32 instance_id_high;
    
    u32 data_location;
    u32 size;
    
    if (type_id == 0xE86B1EEF) {
        DBPFDirectory<size> data @ data_location;
    } else {
        u8 data[size] @ data_location;
    }
};

struct Index<auto entry_count> {
    IndexEntry entries[entry_count];
};

struct Version {
    u32 major;
    u32 minor;
};

struct Header {
    char magic[4];
    
    Version file_version;
    Version user_version;
    
    u32 flags;
    
    u32 created;
    u32 modified;
    
    u32 index_version;
    u32 index_entry_count;
    u32 index_location;
    u32 index_size;
    
    u32 hole_index_entry_count;
    u32 hole_index_location;
    u32 hole_index_size;
    
    u32 index_minor;
    
    Index<index_entry_count> index @ index_location;
    
    //u32 unused[8];
    padding[32];
};

Header header @ 0x00;

