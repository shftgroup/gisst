use std::{
    ffi::{c_char, c_int, c_void},
    mem::ManuallyDrop,
};

pub type RetroDB = c_void;
pub type RetroCursor = c_void;
pub type RetroQuery = c_void;

#[repr(C)]
pub enum RType {
    Null = 0,
    Bool,
    UInt,
    Int,
    String,
    Binary,
    Map,
    Array,
}

#[repr(C)]
pub struct RStr {
    pub len: u32,
    pub buf: *mut c_char,
}

#[repr(C)]
pub struct RBin {
    pub len: u32,
    pub buf: *mut c_char,
}

#[repr(C)]
pub struct RArr {
    pub len: u32,
    pub buf: *mut RVal,
}

#[repr(C)]
pub struct RMap {
    pub len: u32,
    pub buf: *mut RPair,
}

#[repr(C)]
pub struct RPair {
    pub key: RVal,
    pub val: RVal,
}

#[repr(C)]
pub union RValInner {
    pub uint_: u64,
    pub int_: i64,
    pub str_: ManuallyDrop<RStr>,
    pub bin_: ManuallyDrop<RBin>,
    pub bool_: c_int,
    pub map: ManuallyDrop<RMap>,
    pub arr: ManuallyDrop<RArr>,
}

#[repr(C)]
pub struct RVal {
    pub value: RValInner,
    pub tag: RType,
}

#[allow(dead_code)]
#[link(name = "retro-db")]
extern "C" {
    pub fn libretrodb_new() -> *mut RetroDB;
    pub fn libretrodb_open(path: *const c_char, db: *mut RetroDB) -> c_int;
    pub fn libretrodb_close(db: *mut RetroDB);
    pub fn libretrodb_free(db: *mut RetroDB);
    pub fn libretrodb_cursor_new() -> *mut RetroCursor;
    pub fn libretrodb_cursor_open(
        db: *const RetroDB,
        cursor: *mut RetroCursor,
        query: *const RetroQuery,
    ) -> c_int;
    pub fn libretrodb_cursor_read_item(cursor: *mut RetroCursor, out: *mut RVal) -> c_int;
    pub fn libretrodb_cursor_close(cursor: *mut RetroCursor);
    pub fn libretrodb_cursor_free(cursor: *mut RetroCursor);
    pub fn rmsgpack_dom_value_print(obj: *const RVal);
    pub fn rmsgpack_dom_value_free(obj: *mut RVal);

    pub fn libretrodb_create_index(
        db: *mut RetroDB,
        name: *const c_char,
        field_name: *const c_char,
    ) -> c_int;
    pub fn libretrodb_find_entry(
        db: *const RetroDB,
        index_name: *const c_char,
        key: *const c_void,
        out: *mut RVal,
    ) -> c_int;
}
