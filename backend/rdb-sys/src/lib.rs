use std::{
    ffi::{c_char, c_int, c_void},
    mem::ManuallyDrop,
};

pub type RetroDB = c_void;
pub type RetroCursor = c_void;
pub type RetroQuery = c_void;

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy)]
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
    pub buf: *mut u8,
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
    pub fn rmsgpack_dom_value_map_value(map: *const RVal, key: *const RVal) -> *const RVal;
    pub fn libretrodb_create_index(
        db: *mut RetroDB,
        name: *const c_char,
        field_name: *const c_char,
    ) -> c_int;
    pub fn libretrodb_find_entry(
        db: *const RetroDB,
        index_name: *const c_char,
        key: *const u8,
        out: *mut RVal,
    ) -> c_int;
}

impl std::fmt::Display for RVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            RType::Null => write!(f, "nul"),
            RType::Bool => {
                if unsafe { self.value.bool_ } == 0 {
                    write!(f, "false")
                } else {
                    write!(f, "true")
                }
            }
            RType::UInt => write!(f, "{}", unsafe { self.value.uint_ }),
            RType::Int => write!(f, "{}", unsafe { self.value.int_ }),
            RType::String => {
                let slc = unsafe {
                    std::slice::from_raw_parts(
                        self.value.str_.buf as *const u8,
                        self.value.str_.len as usize,
                    )
                };
                write!(
                    f,
                    "\"{}\"",
                    std::str::from_utf8(slc).map_err(|_| std::fmt::Error)?
                )
            }
            RType::Binary => {
                write!(f, "b'")?;
                let slc = unsafe {
                    std::slice::from_raw_parts(
                        self.value.bin_.buf as *const u8,
                        self.value.bin_.len as usize,
                    )
                };
                for chr in slc.iter() {
                    write!(f, "{:x}", chr)?;
                }
                write!(f, "'")
            }
            RType::Map => {
                write!(f, "{{")?;
                let map = unsafe { &self.value.map };
                for idx in 0..map.len {
                    let pair = unsafe { map.buf.add(idx as usize).read() };
                    write!(f, "{}: {}, ", pair.key, pair.val)?;
                }
                write!(f, "}}")
            }
            RType::Array => {
                write!(f, "[")?;
                let arr = unsafe { &self.value.arr };
                for idx in 0..arr.len {
                    let val = unsafe { arr.buf.add(idx as usize).read() };
                    write!(f, "{}, ", val)?;
                }
                write!(f, "]")
            }
        }
    }
}
impl RVal {
    pub fn map_get<K, V>(&self, key: K) -> Option<V>
    where
        K: Into<RVal>,
        V: for<'a> TryFrom<&'a RVal>,
    {
        if self.tag != RType::Map {
            return None;
        }
        let mut key: RVal = key.into();
        let ret = unsafe { rmsgpack_dom_value_map_value(self, &key) };
        unsafe { rmsgpack_dom_value_free(&mut key) };
        if ret.is_null() {
            return None;
        }
        Some(
            unsafe { &ret.read() }
                .try_into()
                .unwrap_or_else(|_e| panic!("Invalid type conversion from rval")),
        )
    }
}

impl From<String> for RVal {
    fn from(s: String) -> RVal {
        let len = s.len() as u32;
        let s = s.into_boxed_str();
        RVal {
            tag: RType::String,
            value: RValInner {
                str_: ManuallyDrop::new(RStr {
                    len,
                    buf: Box::into_raw(s) as *mut i8,
                }),
            },
        }
    }
}
impl From<&str> for RVal {
    fn from(s: &str) -> RVal {
        let s = s.to_string();
        Self::from(s)
    }
}

impl TryFrom<&RVal> for String {
    type Error = ();

    fn try_from(value: &RVal) -> Result<Self, Self::Error> {
        if value.tag != RType::String {
            return Err(());
        }
        let slc = unsafe {
            std::slice::from_raw_parts(
                value.value.str_.buf as *const u8,
                value.value.str_.len as usize,
            )
        };
        std::str::from_utf8(slc)
            .map_err(|_| ())
            .map(|s| s.to_string())
    }
}

impl TryFrom<&RVal> for &str {
    type Error = ();

    fn try_from(value: &RVal) -> Result<Self, Self::Error> {
        if value.tag != RType::String {
            return Err(());
        }
        let slc = unsafe {
            std::slice::from_raw_parts(
                value.value.str_.buf as *const u8,
                value.value.str_.len as usize,
            )
        };
        std::str::from_utf8(slc).map_err(|_| ())
    }
}

impl TryFrom<&RVal> for &[u8] {
    type Error = ();

    fn try_from(value: &RVal) -> Result<Self, Self::Error> {
        if value.tag != RType::Binary {
            return Err(());
        }
        Ok(unsafe {
            std::slice::from_raw_parts(
                value.value.bin_.buf as *const u8,
                value.value.bin_.len as usize,
            )
        })
    }
}

pub unsafe fn librdb_find_entry(db: *mut RetroDB, key: &str, val: &[u8], rval: &mut RVal) -> bool {
    let cursor = unsafe {
        let cursor = libretrodb_cursor_new();
        if libretrodb_cursor_open(db, cursor, std::ptr::null()) != 0 {
            return false;
        }
        cursor
    };
    while (libretrodb_cursor_read_item(cursor, rval) == 0) {
        /* Field not found in item? */
        if rval
            .map_get::<&str, &[u8]>(key)
            .map(|v| v == val)
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}
