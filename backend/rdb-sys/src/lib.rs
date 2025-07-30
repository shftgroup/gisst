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
    pub key: ManuallyDrop<RVal>,
    pub val: ManuallyDrop<RVal>,
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

impl Default for RVal {
    fn default() -> Self {
        Self {
            tag: RType::Null,
            value: RValInner { int_: 0 },
        }
    }
}

impl Drop for RVal {
    fn drop(&mut self) {
        unsafe {
            rmsgpack_dom_value_free(self);
        }
    }
}

#[allow(dead_code)]
#[link(name = "retro-db")]
unsafe extern "C" {
    pub safe fn libretrodb_new() -> *mut RetroDB;
    pub fn libretrodb_open(path: *const c_char, db: *mut RetroDB) -> c_int;
    pub fn libretrodb_close(db: *mut RetroDB);
    pub fn libretrodb_free(db: *mut RetroDB);
    pub safe fn libretrodb_cursor_new() -> *mut RetroCursor;
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
                        self.value.bin_.buf.cast_const(),
                        self.value.bin_.len as usize,
                    )
                };
                for chr in slc {
                    write!(f, "{chr:x}")?;
                }
                write!(f, "'")
            }
            RType::Map => {
                write!(f, "{{")?;
                let map = unsafe { &self.value.map };
                for idx in 0..map.len {
                    let pair = unsafe { ManuallyDrop::new(map.buf.add(idx as usize).read()) };
                    write!(f, "{}: {}, ", &*pair.key, &*pair.val)?;
                }
                write!(f, "}}")
            }
            RType::Array => {
                write!(f, "[")?;
                let arr = unsafe { &self.value.arr };
                for idx in 0..arr.len {
                    let val = unsafe { ManuallyDrop::new(arr.buf.add(idx as usize).read()) };
                    write!(f, "{}, ", &*val)?;
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
        // This one makes a copy so it can be safely dropped
        let key: RVal = key.into();
        self.map_get_rval(&key)
    }
    /// # Panics
    /// If the retrieved value cannot be converted into an `RVal`
    #[must_use]
    pub fn map_get_rval<V>(&self, key: &RVal) -> Option<V>
    where
        V: for<'a> TryFrom<&'a RVal>,
    {
        if self.tag != RType::Map {
            return None;
        }
        let ret = unsafe { rmsgpack_dom_value_map_value(self, key) };
        if ret.is_null() {
            return None;
        }
        Some({
            let ret: ManuallyDrop<RVal> = std::mem::ManuallyDrop::new(unsafe { ret.read() });
            (&*ret)
                .try_into()
                .unwrap_or_else(|_e| panic!("Invalid type conversion from rval"))
        })
    }
}

impl From<String> for RVal {
    fn from(s: String) -> RVal {
        let len = u32::try_from(s.len()).unwrap_or(0);
        let s = s.into_boxed_str();
        RVal {
            tag: RType::String,
            value: RValInner {
                str_: ManuallyDrop::new(RStr {
                    len,
                    buf: Box::into_raw(s).cast::<i8>(),
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
            .map(std::string::ToString::to_string)
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
                value.value.bin_.buf.cast_const(),
                value.value.bin_.len as usize,
            )
        })
    }
}

pub enum RDBError {
    IO,
    Path,
}

pub struct RDB(*mut RetroDB);
unsafe impl Send for RDB {}
unsafe impl Sync for RDB {}
impl RDB {
    /// Opens the database; it will be closed automatically on drop.
    /// # Errors
    /// `RDBError::Path` if there is no database at the given path.
    /// `RDBError::IO` if the database can't be loaded successfully.
    pub fn open(path: &std::path::Path) -> Result<Self, RDBError> {
        let path = path.as_os_str();
        let path = std::ffi::CString::new(path.as_encoded_bytes()).map_err(|_| RDBError::Path)?;
        let db: *mut RetroDB = libretrodb_new();
        if unsafe { libretrodb_open(path.as_ptr(), db) == 0 } {
            Ok(Self(db))
        } else {
            Err(RDBError::IO)
        }
    }
    #[must_use]
    pub fn open_cursor(&self) -> Option<Cursor> {
        unsafe {
            let cursor = libretrodb_cursor_new();
            if libretrodb_cursor_open(self.0, cursor, std::ptr::null()) != 0 {
                return None;
            }
            Some(Cursor(cursor))
        }
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn find_entry<K, V>(&self, key: K, val: V) -> Option<RVal>
    where
        K: Into<RVal>,
        V: for<'a> TryFrom<&'a RVal> + std::cmp::PartialEq,
    {
        self.find_entry_by::<K, V>(key, |kv| kv == val)
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn find_entry_by<K, V>(&self, key: K, test: impl Fn(V) -> bool) -> Option<RVal>
    where
        K: Into<RVal>,
        V: for<'a> TryFrom<&'a RVal> + std::cmp::PartialEq,
    {
        let mut cursor = self.open_cursor()?;
        let key: RVal = key.into();
        while let Some(rval) = cursor.next() {
            if rval.map_get_rval::<V>(&key).is_some_and(&test) {
                return Some(rval);
            }
        }
        None
    }
}
impl Drop for RDB {
    fn drop(&mut self) {
        unsafe {
            libretrodb_close(self.0);
            libretrodb_free(self.0);
        }
    }
}

pub struct Cursor(*mut RetroCursor);
impl Cursor {
    fn next(&mut self) -> Option<RVal> {
        let mut rval = RVal::default();
        if unsafe { libretrodb_cursor_read_item(self.0, &raw mut rval) == 0 } {
            Some(rval)
        } else {
            None
        }
    }
}
impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe {
            libretrodb_cursor_close(self.0);
            libretrodb_cursor_free(self.0);
        }
    }
}
