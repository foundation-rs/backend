use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::ora_source::*;

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

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnType {
    Int16, Int32, Int64, Float64, Varchar, DateTime, Blob, Clob, Long, Unsupported
}

#[derive(Debug)]
pub struct ColumnInfo {
    pub name:           String,
    pub col_type:       ColumnType,
    pub col_type_name:  String,
    pub oci_data_type:  u16,
    pub col_len:        u16,
    pub nullable:       bool,
    pub data_precision: u16,
    pub data_scale:     u16,
    pub buffer_len:     usize
}

#[derive(Debug)]
pub struct PrimaryKey {
    pub name:    String,
    pub columns: Vec<String>
}

#[derive(Debug)]
pub struct TableIndex {
    pub name:    String,
    pub unique:  bool,
    pub columns: Vec<IndexColumn>
}

#[derive(Debug)]
pub struct IndexColumn {
    pub name: String,
    pub desc: bool
}

impl From<OraTableColumn> for ColumnInfo {
    fn from(v: OraTableColumn) -> ColumnInfo {
        use std::mem::size_of;

        let name = v.column_name;
        let nullable = v.nullable == "Y";
        let data_scale = v.data_scale;
        let data_precision = v.data_precision;
        let col_len = v.data_length;

        let mut col_type_name = v.data_type;

        let (col_type, oci_data_type, buffer_len) = {
            let ctn: &str = &col_type_name.clone();
            match ctn {
                "CHAR" | "VARCHAR2" => {
                    // SQLT_CHR
                    (ColumnType::Varchar, 1, col_len as usize)
                },
                "LONG" => {
                    // SQLT_CHR
                    (ColumnType::Long, 1, 4000)
                },
                "DATE" => {
                    // SQLT_DAT
                    (ColumnType::DateTime, 1, 12)
                },
                "CLOB" => {
                    // SQLT_CLOB
                    (ColumnType::Clob, 112, 0)
                },
                "BLOB" => {
                    // SQLT_BLOB
                    (ColumnType::Blob, 113, 0)
                },
                "NUMBER" => {
                    if data_scale == 0 {
                        if data_precision == 0 || data_precision > 7 {
                            if data_precision == 0 {
                                col_type_name = "INTEGER".to_string();
                            }
                            // SQLT_NUM
                            (ColumnType::Int64, 2, size_of::<i64>())
                        } else if data_precision > 4 {
                            // SQLT_NUM
                            (ColumnType::Int32, 2, size_of::<i32>())
                        } else {
                            // SQLT_NUM
                            (ColumnType::Int16, 2, size_of::<i16>())
                        }
                    } else {
                        // SQLT_NUM
                        (ColumnType::Float64, 2, size_of::<f64>())
                    }
                },
                _ => {
                    // Unsupported
                    (ColumnType::Unsupported, 0, 0)
                }
            }
        };

        ColumnInfo { name, col_type, col_type_name, oci_data_type, col_len, nullable, data_precision, data_scale, buffer_len }
    }
}
