//! sp_cursor (Transact-SQL)
//!
//! <https://learn.microsoft.com/en-us/sql/relational-databases/system-stored-procedures/sp-cursor-transact-sql?view=sql-server-ver17>

#![allow(unused)]

use crate::tds::types::sp::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct OpType(pub u32);

impl OpType {
    pub const UPDATE: Self = Self(0x01);
    pub const DELETE: Self = Self(0x02);
    pub const INSERT: Self = Self(0x04);
    pub const REFRESH: Self = Self(0x08);
    pub const LOCK: Self = Self(0x0010);
    pub const SETPOSITION: Self = Self(0x20);
    pub const ABSOLUTE: Self = Self(0x40);
}

impl core::ops::BitOr for OpType {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct SpCursor {
    cursor: i32,
    op_type: OpType,
    row_num: i32,
    table: Option<String>,
    values: Vec<ParameterData>,
}

impl SpCursor {
    pub fn into_rpc_batch(self, all_headers: AllHeaders) -> RPCReqBatch {
        let mut builder = RPCReqBatchBuilder::default();

        let cursor = ParameterData::cursor(String::new(), self.cursor);
        let op_type = ParameterData::uint4(String::new(), self.op_type.0);
        let row_num = ParameterData::int4(String::new(), self.row_num);
        let table= ParameterData::nvarchar(String::new(), &self.table.unwrap_or("".to_string()));

        let mut parameters = vec![cursor, op_type, row_num, table];
        parameters.extend(self.values);
        
        #[cfg(feature = "tds7.4")]
        builder.enclave_package(vec![]);

        builder
        .all_headers(all_headers)
        .name_len_proc_id(NameLenProcId::ProcID(ProcId::SpCursor))
        .option_flags(OptionFlags::default())
        .parameter_data(parameters)
        .build()
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_type_bitor() {
        let op = OpType::UPDATE | OpType::DELETE;
        assert_eq!(op.0, 0x03)
    }

    #[test]
    fn sp_cursor_into_rpc_batch() {
        let batch = SpCursor {
            cursor: 1, 
            op_type: OpType::UPDATE | OpType::DELETE,
            row_num: 0,
            table: None,
            values: vec![],
        }.into_rpc_batch(AllHeaders::new(vec![]));

        assert!(matches!(
            batch.name_len_proc_id,
            NameLenProcId::ProcID(ProcId::SpCursor)
        ))

    }
}