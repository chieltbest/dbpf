#pragma endian little

enum Type: u32 {
    UInt    = 0xEB61E4F7,
    String = 0x0B8BEA18,
    Float  = 0xABC78708,
    Bool   = 0xCBA908E1,
    Int   = 0x0C264712,
};

struct String {
    u32 size;
    char data[size];
};

struct Item {
    Type type;
    String name;
    
    if (type == Type::UInt) u32 prop;
    if (type == Type::String) String prop;
    if (type == Type::Float) float prop;
    if (type == Type::Bool) bool prop;
    if (type == Type::Int) u32 prop;
};

struct CPF {
    u32 type_id;
    u16 version;
    u32 count;
    
    Item entries[count];
};

CPF cpf @ 0x00;
