#![feature(libc)]

extern crate libc;
use std::ffi::CString;
use std::slice;
use std::ptr;

mod ffi;

#[derive(Debug, Copy, Clone)]
pub struct Env {
    env: ffi::Voidptr
}

#[derive(Debug, Copy, Clone)]
pub struct Db {
    db: ffi::Voidptr
}

// XXX: Use Arc
unsafe impl Sync for Db {}
unsafe impl Send for Db {}
unsafe impl Sync for Env {}
unsafe impl Send for Env {}

use std::marker::PhantomData;

#[macro_export]
macro_rules! obj {
    (
        $db:ident ;
        $(
            $name:ident : $x:expr
        ),*
    ) => {{
        let mut obj = $db.obj(); 
        $(
            obj.$name($x)
        );*
        ;obj
    }}
}


#[derive(Debug)]
pub struct DbObject<'a> {
    o: ffi::Voidptr,
    phantom: PhantomData<&'a ()>,
}

impl<'a> DbObject<'a> {
    // We name it attr(), to not mix it up with the set() method.
    pub fn attr<'b>(&'b mut self, path: &str, data: &'b[u8]) {
        let path = CString::new(path).unwrap();
        unsafe {ffi::setstring(self.o, path.as_bytes(), data); }
    }

    // Shortcut for attr("key", ...)
    pub fn key<'b>(&'b mut self, data: &'b[u8]) {
        unsafe {ffi::setstring(self.o, "key\0".as_bytes(), data); }
    }

    // Shortcut for attr("key_b", ...)
    // This allows setting a secondary key.
    pub fn key_b<'b>(&'b mut self, data: &'b[u8]) {
        unsafe {ffi::setstring(self.o, "key_b\0".as_bytes(), data); }
    }

    // Shortcut for attr("value", ...)
    pub fn value<'b>(&'b mut self, data: &'b[u8]) {
        unsafe {ffi::setstring(self.o, "value\0".as_bytes(), data); }
    }

    // Shortcut for attr("val", ...)
    pub fn val<'b>(&'b mut self, data: &'b[u8]) {
        self.value(data)
    }
 
    // Shortcut for attr("order", ...)
    pub fn order<'b>(&'b mut self, data: &'b[u8]) {
        unsafe {ffi::setstring(self.o, "order\0".as_bytes(), data); }
    }

    // Shortcut for attr("prefix", ...)
    pub fn prefix<'b>(&'b mut self, data: &'b[u8]) {
        unsafe {ffi::setstring(self.o, "prefix\0".as_bytes(), data); }
    }
}

impl<'a> Drop for DbObject<'a> {
    fn drop(&mut self) {
        if !self.o.is_null() {
            unsafe {ffi::sp_destroy(self.o)};
        }
    }
}

#[derive(Debug)]
pub struct DbResultObject<'a> {
    o: ffi::Voidptr,
    phantom: PhantomData<&'a ()>,
}

impl<'a> DbResultObject<'a> {
    fn get_<'b>(&'b self, key: &'b [u8]) -> Option<&'b[u8]> {
        unsafe {
            let mut sz = 0;
            let valptr = ffi::sp_getstring(self.o, key.as_ptr(), &mut sz);

            // XXX: what if we stored an empty value?
            if valptr.is_null() {
                return None;
            }

            let s = slice::from_raw_parts(valptr as *const u8, sz as usize);
            return Some(s);
        }

    }
    pub fn get_value<'b>(&'b self) -> Option<&'b[u8]> {
        self.get_(b"value\0")
    }

    pub fn get_key<'b>(&'b self) -> Option<&'b[u8]> {
        self.get_(b"key\0")
    }

    pub fn get_key_b<'b>(&'b self) -> Option<&'b[u8]> {
        self.get_(b"key_b\0")
    }
}

impl<'a> Drop for DbResultObject<'a> {
    fn drop(&mut self) {
        if !self.o.is_null() {
            unsafe {ffi::sp_destroy(self.o)};
        }
    }
}

#[derive(Debug)]
pub struct Transaction<'a> {
    tx: ffi::Voidptr,
    phantom: PhantomData<&'a ()>
}

impl<'a> Drop for Transaction<'a> {
    // Unless commit() is called, this will roll back the transaction
    fn drop(&mut self) {
        if !self.tx.is_null() {
            unsafe {ffi::sp_destroy(self.tx);}
        }
    }
}

pub struct Cursor<'a> {
    obj: ffi::Voidptr,
    phantom: PhantomData<&'a ()>
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        unsafe {ffi::sp_destroy(self.obj);}
    }
}

impl<'a> Cursor<'a> {
    pub fn next<'b>(&'b self) -> Option<DbResultObject<'b>> {
        let res = unsafe { ffi::sp_get(self.obj, ptr::null_mut()) };
        if res.is_null() {
            None
        }
        else {
            Some(DbResultObject{o: res, phantom: PhantomData})
        }
    }
}

pub trait SetGetOps {
    fn backend(&self) -> ffi::Voidptr;

    // Consumes the DbObject
    fn set<'a>(&self, obj: DbObject<'a>) {
        let mut obj = obj;
        unsafe {ffi::sp_set(self.backend(), obj.o)}; // XXX: Check error code
        obj.o = ptr::null_mut(); // sp_set drops it
    }

    // XXX: Make sure self.db == dbobject.db
    fn get<'a, 'b>(&'a self, pattern: DbObject<'b>) -> Option<DbResultObject<'a>> {
        unsafe {
            let res = ffi::sp_get(self.backend(), pattern.o);
            if res.is_null() {
                return None;
            }
            let n = DbResultObject {
                o: pattern.o,
                phantom: PhantomData
            };
            let mut pattern = pattern;
            pattern.o = ptr::null_mut(); 
            return Some(n);
        }
    }

    fn cursor<'a, 'b>(&'a self, pattern: DbObject<'b>) -> Cursor<'a> {
        unsafe {
            let cursor = ffi::sp_cursor(self.backend(), pattern.o);
            assert!(!cursor.is_null());

            // pattern is freed as part of the Cursor
            let mut pattern = pattern;
            pattern.o = ptr::null_mut();

            Cursor {obj: cursor, phantom: PhantomData}
        }
    }
}

impl<'a> Transaction<'a> {
    pub fn commit(mut self) -> i32 {
        let rc = unsafe{ffi::sp_commit(self.tx)};
        self.tx = ptr::null_mut();
        rc as i32
    }
}

impl<'a> SetGetOps for Transaction<'a> {
    fn backend(&self) -> ffi::Voidptr { self.tx }
}

impl Env {
    pub fn new() -> Env {
        Env {env: unsafe{ffi::sp_env()}}
    }

    pub fn destroy(self) {
        unsafe{ffi::sp_destroy(self.env)};
    }

    pub fn db(&mut self, dbname: &str) {
        // XXX: is dbname copied?
        let dbname = CString::new(dbname).unwrap();
        unsafe {ffi::setstring(self.env, "db\0".as_bytes(), dbname.as_bytes()) };
    }

    pub fn open(&mut self) {
        unsafe {ffi::sp_open(self.env)};
    }
    
    pub fn begin<'a>(&'a self) -> Transaction<'a> {
        let tx = unsafe{ffi::sp_begin(self.env)};
        assert!(!tx.is_null());
        Transaction {
            tx: tx,
            phantom: PhantomData
        }
    }

    pub fn get_db(&self, dbname: &str) -> Option<Db> {
        let dbstr = "db.".to_string() + dbname;
        let attr = CString::new(&dbstr[..]).unwrap();
        unsafe {
            let obj = ffi::sp_getobject(self.env, attr.as_ptr() as *const u8);
            if obj.is_null() {
                None
            }
            else {
                Some(Db {db: obj})
            }
        }
    }

    pub fn setattr(&mut self, key: &str, val: &str) {
        let key = CString::new(key).unwrap();
        let val = CString::new(val).unwrap();
        unsafe {ffi::setstring(self.env, key.as_bytes(), val.as_bytes()) };
    }

    pub fn setintattr(&mut self, key: &str, val: i64) {
        let key = CString::new(key).unwrap();
        unsafe {ffi::sp_setint(self.env, key.as_ptr(), val) };
    }
}

impl SetGetOps for Db {
    fn backend(&self) -> ffi::Voidptr { self.db }
}

impl Db {
    pub fn obj<'a>(&'a self) -> DbObject<'a> {
        let obj = unsafe { ffi::sp_object(self.db) };
        assert!(!obj.is_null());
        DbObject{o: obj, phantom: PhantomData} 
    }

}
