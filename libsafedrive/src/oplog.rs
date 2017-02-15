#![allow(dead_code)]

#![allow(unused_variables)]

use std;

use ::nom::{IResult, rest, le_u8, le_u64};


#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpType {
    Initial,
    Commit,
    Rollback,
    FileCreate,
    FileDelete,
    UpdateCTime,
    UpdateATime,
    UpdateMTime,
    UpdateName,
    UpdatePermissions,
    InsertData,
    DeleteData,
    UpdateData,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    HardLink,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Op<'a> {
    pub sequence: u64,
    /// a serialized OpType
    pub operation_type: u8,
    /// a serialized FileType
    pub file_type: u8,
    pub file_id: u64,
    /// pointers to any relevant data needed for this op, as block ID values
    pub data_pointers: &'a [u8],
}

impl<'a> std::fmt::Display for Op<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Op <#{}| type:{}, file_id:{}, data_pointers:{}>", self.sequence, self.operation_type, self.file_id, self.data_pointers.len())
    }
}

pub fn op_parse<'a>(input: &'a [u8]) -> IResult<&'a [u8], Op<'a>> {
    chain!(input,
        magic: map_res!(tag!("sdop"), std::str::from_utf8)                           ~
        sequence: le_u64                                                             ~
        operation_type: le_u8                                                        ~
        file_type: le_u8                                                             ~
        file_id: le_u64                                                              ~
        data_pointers: rest                                                          ,
        || {
        Op {
            operation_type: operation_type,
            sequence: sequence,
            file_type: file_type,
            file_id: file_id,
            data_pointers: data_pointers,
        }
    })
}