pub use crate::common::{BigString, FileName, LanguageCode, NullString};
use binrw::binrw;
use crate::common::ByteString;

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaggedString {
    pub language_code: LanguageCode,
    pub value: NullString,
    pub description: NullString,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UntaggedString {
    pub value: BigString,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Version {
    #[brw(magic = 0xFFF6u16)]
    V9,
    #[default]
    #[brw(magic = 0xFFFDu16)]
    V2,
    #[brw(magic = 0xFFFEu16)]
    V1,
    #[brw(magic = 0xFFFFu16)]
    V0,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionedTextList {
    Tagged {
        version: Version,
        #[br(temp)]
        #[bw(calc = sets.len() as u16)]
        count: u16,
        #[br(count = count)]
        sets: Vec<TaggedString>,
    },
    Untagged {
        #[br(big, temp)]
        #[bw(calc = sets.len() as u16)]
        count: u16,
        #[br(count = count)]
        sets: Vec<UntaggedString>,
    }
}

impl Default for VersionedTextList {
    fn default() -> Self {
        Self::Tagged {
            version: Version::default(),
            sets: vec![],
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextList {
    pub file_name: FileName,
    pub data: VersionedTextList,
}

impl From<UntaggedString> for TaggedString {
    fn from(value: UntaggedString) -> Self {
        Self {
            value: ByteString::from(value.value).into(),
            ..Self::default()
        }
    }
}

impl From<TaggedString> for UntaggedString {
    fn from(value: TaggedString) -> Self {
        Self {
            value: ByteString::from(value.value).into(),
        }
    }
}
