#![feature(libc)]

extern crate libc;

use libc::{c_void, c_int, c_char};
use std::ffi::CString;
use std::slice;

pub type Voidptr = *mut c_void;

#[link(name = "sophia")]
extern {
    fn sp_env() -> Voidptr;
    fn sp_object(a: Voidptr) -> Voidptr;
    fn sp_open(a: Voidptr) -> c_int;
    fn sp_drop(a: Voidptr) -> c_int;
    fn sp_destroy(a: Voidptr) -> c_int;
    fn sp_error(a: Voidptr) -> c_int;
    fn sp_asynchronous(a: Voidptr) -> Voidptr;
    fn sp_poll(a: Voidptr) -> Voidptr;
    fn sp_setobject(a: Voidptr, b: *const c_char, c: Voidptr) -> c_int;
    fn sp_setstring(a: Voidptr, b: *const c_char, c: *const c_void, d: c_int) -> c_int;
    fn sp_setint(a: Voidptr, b: *const c_char, c: i64) -> c_int;

    fn sp_getobject(a: Voidptr, b: *const u8) -> Voidptr;
    fn sp_getstring(a: Voidptr, b: *const u8, c: *mut c_int) -> Voidptr;
    fn sp_getint(a: Voidptr, b: *mut u8) -> i64;
    fn sp_set(a: Voidptr, b: Voidptr) -> c_int;
    fn sp_update(a: Voidptr, b: Voidptr) -> c_int;
    fn sp_delete(a: Voidptr, b: Voidptr) -> c_int;
    fn sp_get(a: Voidptr, b: Voidptr) -> Voidptr;
    fn sp_cursor(a: Voidptr, b: Voidptr) -> Voidptr;
    fn sp_begin(a: Voidptr) -> Voidptr;
    fn sp_prepare(a: Voidptr) -> c_int;
    fn sp_commit(a: Voidptr) -> c_int;
}

pub struct Env {
    env: Voidptr
}

pub struct Db {
    db: Voidptr
}

impl Env {
    pub fn new() -> Env {
        Env {env: unsafe{sp_env()}}
    }

    pub fn destroy(self) {
        unsafe{sp_destroy(self.env)};
    }

    pub fn db(&mut self, dbname: &str) {
        let db = CString::new("db").unwrap();
        let dbname = CString::new(dbname).unwrap();
        unsafe {sp_setstring(self.env, db.as_ptr(), dbname.as_ptr() as *const c_void, 0) };
    }

    pub fn open(&mut self) {
        unsafe {sp_open(self.env)};
    }

    pub fn get_object(&mut self, attr: &str) -> Voidptr {
        let attr = CString::new(attr).unwrap();
        unsafe {sp_getobject(self.env, attr.as_ptr() as *const u8)}
    }

    pub fn get_db(&mut self, dbname: &str) -> Option<Db> {
        let dbstr = "db.".to_string() + dbname;
        let attr = CString::new(&dbstr[..]).unwrap();
        unsafe {
            let obj = sp_getobject(self.env, attr.as_ptr() as *const u8);
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
        unsafe {sp_setstring(self.env, key.as_ptr(), val.as_ptr() as *const c_void, 0) };
    }
}

impl Db {
    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        unsafe {
            let mut o = sp_object(self.db);
            sp_setstring(o, "key\0".as_ptr() as *const c_char, key.as_ptr() as Voidptr, key.len() as i32);
            sp_setstring(o, "value\0".as_ptr() as *const c_char, value.as_ptr() as Voidptr, value.len() as i32);
            sp_set(self.db, o);
        }
    }

    // XXX: Return a value object
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        unsafe {
            let mut o = sp_object(self.db);
            sp_setstring(o, "key\0".as_ptr() as *const c_char, key.as_ptr() as Voidptr, key.len() as i32);
            let mut o2 = sp_get(self.db, o);
            if o2.is_null() {
                return None;
            }
            let mut sz = 0;
            let valptr = sp_getstring(o2, "value\0".as_ptr(), &mut sz);

            // XXX: What if we have not stored a value?
            if valptr.is_null() {
                sp_destroy(o2);
                return None;
            }

            let s = slice::from_raw_parts(valptr as *const u8, sz as usize);
            let vec = s.to_vec();
            sp_destroy(o2);
            return Some(vec);
        }
    }
}
