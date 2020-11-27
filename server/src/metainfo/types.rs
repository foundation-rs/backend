use std::borrow::Borrow;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use super::ora_source::*;
use oracle::{self, SqlType};

pub struct MetaInfo {
    pub schemas:  HashSet<SchemaInfo>,
}

#[derive(Debug, Eq)]
pub struct SchemaInfo {
    pub name:    String,
    pub tables:  HashSet<TableInfo>,
}

impl Hash for SchemaInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl PartialEq for SchemaInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Borrow<str> for SchemaInfo {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct TableInfo {
    pub name:        String,
    pub is_view:     bool,
    pub temporary:   bool,
    pub num_rows:    i32,
    pub columns:     Vec<ColumnInfo>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes:     Vec<TableIndex>
}

impl TableInfo {

    pub(crate) fn set_primary_key(&self, pk: PrimaryKey) {
        self.get_primary_key_as_mutable().replace(pk);
    }

    pub(crate) fn push_table_index(&self, index: TableIndex) {
        self.get_indexes_as_mutable().push(index);
    }

    // hashset don't have get_mut member, but we change only primary key, remaining name unchanged
    fn get_primary_key_as_mutable(&self) -> &mut Option<PrimaryKey> {
        let reference = &(self.primary_key);
        let cont_pointer = reference as *const Option<PrimaryKey>;
        let mut_pointer = cont_pointer as *mut Option<PrimaryKey>;
        unsafe { &mut *mut_pointer }
    }

    // hashset don't have get_mut member, but we change only indexes vector, remaining name unchanged
    fn get_indexes_as_mutable(&self) -> &mut Vec<TableIndex> {
        let reference = &(self.indexes);
        let cont_pointer = reference as *const Vec<TableIndex>;
        let mut_pointer = cont_pointer as *mut Vec<TableIndex>;
        unsafe { &mut *mut_pointer }
    }

}

impl Hash for TableInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl PartialEq for TableInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for TableInfo {}

impl Borrow<str> for TableInfo {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct ColumnInfo {
    pub name:           String,
    pub col_type:       SqlType,
    pub oci_data_type:  oracle::TypeDescriptor,
    pub col_type_name:  String, // type name in typescript
    pub nullable:       bool
}

#[derive(Debug)]
pub struct PrimaryKey {
    pub name:    String,
    pub column_indices: Vec<usize>
}

#[derive(Debug)]
pub struct TableIndex {
    pub name:    String,
    pub unique:  bool,
    pub columns: Vec<IndexColumn>
}

#[derive(Debug)]
pub struct IndexColumn {
    pub column_index: usize,
    pub desc: bool
}

impl TryFrom<OraTableColumn> for ColumnInfo {
    type Error = &'static str;

    fn try_from(v: OraTableColumn) -> Result<Self, Self::Error> {
        let name = v.column_name.to_lowercase();
        let nullable = v.nullable == "Y";

        let data_scale = v.data_scale;
        let data_precision = v.data_precision;
        let col_len = v.data_length;
        let ora_type_name = v.data_type.as_str();

        let (col_type, oci_data_type, col_type_name) = {
            match ora_type_name {
                "CHAR" | "VARCHAR2" => {
                    (SqlType::Varchar, ((SqlType::Varchar, col_len as usize)).into(), "string".to_owned())
                },
                "LONG" => {
                    (SqlType::Varchar, SqlType::Long.into(), "string".to_owned())
                },
                "DATE" => {
                    (SqlType::DateTime, SqlType::DateTime.into(), "string".to_owned())
                },
                /*
                "CLOB" => {
                    // SQLT_CLOB
                    (SqlType::Clob, 112, 0)
                },
                "BLOB" => {
                    // SQLT_BLOB
                    (SqlType::Blob, 113, 0)
                },
                 */
                "NUMBER" => {
                    let col_type =
                        if data_scale == 0 {
                            if data_precision == 0 || data_precision > 7 {
                                SqlType::Int64
                            } else if data_precision > 4 {
                                SqlType::Int32
                            } else {
                                SqlType::Int16
                            }
                        } else {
                            SqlType::Float64
                        };
                    (col_type, col_type.into(), "number".to_owned())
                },
                _ => {
                    // Unsupported
                    return Err("Unsupported SQL type")
                }
            }
        };

        Ok( ColumnInfo { name, col_type, oci_data_type, col_type_name, nullable } )
    }
}
