pub mod cpf;
pub mod xml;

use binrw::{binrw, NullString};

#[binrw]
#[brw(little)]
struct FileName {
    #[brw(pad_size_to = 0x40)]
    #[bw(assert(name.0.len() < 0x40))]
    name: NullString,
}
