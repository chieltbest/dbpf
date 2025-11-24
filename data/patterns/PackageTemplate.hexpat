// SPDX-FileCopyrightText: 2023 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma endian little

#include <std/sys.pat>

enum CompressionType: u16 {
    Uncompressed = 0x0000,
    Streamable = 0xfffe,
    Internal = 0xffff,
    Deleted = 0xffe0,
    ZLib = 0x5a42,
};

bitfield DataSize {
    size: 31;
    ext_compressed: 1;
};

bitfield Flags {
    constant_type: 1;
    constant_group: 1;
    constant_instance: 1;
    unused: 29;
};

bitfield GroupId {
    id: 31;
    is_custom: 1;
};

struct IndexEntry<auto entry_flags> {
    if (!entry_flags.constant_type) {
        u32 type_id;
    }
    if (!entry_flags.constant_group) {
        GroupId group_id;
    }
    if (!entry_flags.constant_instance) {
        u32 instance_id_high;
    }
    
    u32 instance_id_low;
    
    u32 data_location;
    DataSize data_size;
    u32 size_decompressed;
    
    if (data_size.ext_compressed) {
        CompressionType c_type;
        u16 commited;
    }
    
    u8 data[data_size.size] @ data_location;
};

struct Index<auto entry_count> {
    Flags flags;
    if (flags.constant_type) {
        u32 constant_type_id;
    }
    if (flags.constant_group) {
        GroupId constant_group_id;
    }
    if (flags.constant_instance) {
        u32 constant_instance_id;
    }
    
    IndexEntry<flags> entries[entry_count];
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
    u32 index_location_low;
    u32 index_size;
    
    u32 hole_index_entry_count;
    u32 hole_index_location;
    u32 hole_index_size;
    
    u32 index_minor;
    
    u64 index_location;
    
    if (index_location == 0) {
        u64 index_loc = index_location_low;
    } else {
        u64 index_loc = index_location;
    }
    
    Index<index_entry_count> index @ index_loc;
    
    //u32 unused[6];
    padding[24];
};

Header header @ 0x00;
