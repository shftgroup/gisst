use std::ffi::{c_char, c_int, c_void};

pub type RetroDB = c_void;
pub type RetroCursor = c_void;
pub type RetroQuery = c_void;
pub type RVal = c_void;

#[allow(dead_code)]
#[link(name = "retro-db")]
extern "C" {
    pub fn libretrodb_open(path: *const c_char, db: *mut RetroDB) -> c_int;
    pub fn libretrodb_close(db: *const RetroDB);
    pub fn libretrodb_free(db: *const RetroDB);
    pub fn libretrodb_cursor_open(
        db: *const RetroDB,
        cursor: *mut RetroCursor,
        query: *const RetroQuery,
    ) -> c_int;
    pub fn libretrodb_cursor_read_item(cursor: *const RetroCursor, out: *mut RVal) -> c_int;
    pub fn libretrodb_cursor_close(cursor: *const RetroCursor);
    pub fn libretrodb_cursor_free(cursor: *const RetroCursor);
    pub fn rmsgpack_dom_value_print(obj: *const RVal);
    pub fn rmsgpack_dom_value_free(obj: *const RVal);

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
