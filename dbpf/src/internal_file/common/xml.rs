/*use std::fmt::{Debug, Formatter};
use binrw::{binrw, helpers::until_eof};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct XML {
    #[br(parse_with = until_eof)]
    data: Vec<u8>,
}

impl Debug for XML {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.data))
    }
}*/
