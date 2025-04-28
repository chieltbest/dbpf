use crate::common::FileName;
use binrw::NullString;
use crate::common::LanguageCode;
use binrw::binrw;

#[binrw]
#[derive(Clone, Debug, Default)]
pub struct String {
    pub language_code: LanguageCode,
    pub value: NullString,
    pub description: NullString,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default)]
pub struct TextList {
    pub file_name: FileName,
    // TODO 0xFFFD magic?
    pub format_code: u16,
    #[br(temp)]
    #[bw(calc = sets.len() as u16)]
    count: u16,
    #[br(count = count)]
    pub sets: Vec<String>,
}
