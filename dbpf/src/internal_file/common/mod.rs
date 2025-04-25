pub mod cpf;
pub mod xml;

use binrw::{binrw, NullString};
use crate::common;

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default)]
pub struct FileName {
    #[brw(pad_size_to = 0x40)]
    #[bw(assert(name.0.len() < 0x40))]
    pub name: NullString,
}

#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, Default)]
pub enum LanguageCode {
    #[default]
    English = 1,
    Abo = 2,
    Abn = 3,
    Abm = 4,
    Abl = 5,
    Abk = 6,
    Abj = 7,
    Abi = 8,
    Abh = 9,
    Abg = 10,
    Abf = 11,
    Abe = 12,
    Abd = 13,
    Abc = 14,
    Abb = 15,
    Aba = 16,
    Aaz = 17,
    Aay = 18,
    Aax = 19,
    Aaw = 20,
    Aav = 21,
    Aau = 22,
    Aat = 23,
    Aas = 24,
    Aar = 25,
    Aaq = 26,
    Aap = 27,
    Aao = 28,
    Aan = 29,
    Aam = 30,
    Aal = 31,
    Aak = 32,
    Aaj = 33,
    Aai = 34,
    Aah = 35,
    Aag = 36,
    Aaf = 37,
    Aae = 38,
    Aad = 39,
    Aac = 40,
    Aab = 41,
    Aaa = 42,
    Dutch = 43,
}

pub type Id = common::String;
