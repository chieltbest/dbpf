use std::fmt::{Debug, Formatter};
use binrw::binrw;

#[binrw]
#[derive(Clone)]
pub struct String {
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub data: Vec<u8>,
}

impl Debug for String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", std::string::String::from_utf8_lossy(&self.data))
    }
}
