use binrw::binrw;
use crate::internal_file::common::cpf::CPF;
// use crate::internal_file::xml::XML;

#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub enum PropertySet {
    CPF(CPF),
    // XML(XML),
}
