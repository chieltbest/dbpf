use std::io::{Read, Seek, Write};
use binrw::{BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt, Endian, NamedArgs, args};
use crate::common::FileName;
use crate::internal_file::behaviour_function::Goto::{Error, False, Instr, True};
use crate::internal_file::behaviour_function::Signature::*;

#[binrw]
#[brw(repr = u16)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Default)]
pub enum Signature {
    V0 = 0x8000,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    #[default]
    V9,
}

#[binrw]
#[br(import {opcode: u16})]
pub enum Function {
    Unknown(#[br(calc = opcode)]
            #[bw(ignore)]
            u16,
            [u8; 16]),
}

impl Function {
    pub fn opcode(&self) -> u16 {
        match self {
            Function::Unknown(opcode, _) => *opcode
        }
    }
}

#[derive(NamedArgs, Clone, Debug)]
pub struct GotoBinArgs {
    signature: Signature,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum Goto {
    #[default]
    Error,
    True,
    False,
    Instr(u16),
}

impl BinRead for Goto {
    type Args<'a> = GotoBinArgs;

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
        Ok(if args.signature < V7 {
            match reader.read_type::<u8>(endian)? {
                0xFD => Error,
                0xFE => True,
                0xFF => False,
                n => Instr(n as u16),
            }
        } else {
            match reader.read_type::<u16>(endian)? {
                0xFFFC => Error,
                0xFFFD => True,
                0xFFFE => False,
                n => Instr(n),
            }
        })
    }
}

impl BinWrite for Goto {
    type Args<'a> = GotoBinArgs;

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, args: Self::Args<'_>) -> BinResult<()> {
        if args.signature < V7 {
            match self {
                Error => writer.write_type(&0xFDu8, endian)?,
                True => writer.write_type(&0xFEu8, endian)?,
                False => writer.write_type(&0xFFu8, endian)?,
                Instr(n) => writer.write_type(&(*n as u8), endian)?,
            }
        } else {
            match self {
                Error => writer.write_type(&0xFFFCu16, endian)?,
                True => writer.write_type(&0xFFFDu16, endian)?,
                False => writer.write_type(&0xFFFEu16, endian)?,
                Instr(n) => writer.write_type(n, endian)?,
            }
        }
        Ok(())
    }
}

#[binrw]
#[brw(import {signature: Signature})]
#[derive(Clone, Debug, Default)]
pub struct Instruction {
    // #[br(temp)]
    // #[bw(calc = function.opcode())]
    pub opcode: u16,
    #[brw(args {signature})]
    pub true_target: Goto,
    #[brw(args {signature})]
    pub false_target: Goto,
    #[brw(if(signature >= V5))]
    pub node_version: u8,
    #[br(count = if signature >= V3 { 16 } else { 8 })]
    // #[bw(assert(operands.len() == if signature >= V3 { 16 } else { 8 }))]
    pub operands: Vec<u8>,

    // pub function: Function,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default)]
pub struct BehaviourFunction {
    pub name: FileName,
    pub signature: Signature,
    #[br(temp)]
    #[bw(calc = instructions.len() as u16)]
    num_instructions: u16,
    pub tree_type: u8,
    pub num_parameters: u8,
    pub num_locals: u8,
    pub header_flags: u8,
    pub tree_version: i32,

    #[brw(if(matches!(signature, V9)))]
    pub cache_flags: u8,

    #[br(count = num_instructions, args { inner: args ! {signature}})]
    #[bw(args { signature: * signature })]
    pub instructions: Vec<Instruction>,
}
