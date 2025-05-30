use binrw::binrw;
use crate::common::{FileName, NullString};

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AudioReference {
    pub file_name: FileName,
    pub reference: NullString,
}
