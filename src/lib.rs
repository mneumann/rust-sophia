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
        unsafe {ffi::setstring(self.env, "db\0".as_bytes(), dbname.as_bytes(), 0) };
    }

    pub fn open(&mut self) {
        unsafe {ffi::sp_open(self.env)};
    }

    /*
    fn get_object(&mut self, attr: &str) -> Voidptr {
        let attr = CString::new(attr).unwrap();
        unsafe {ffi::sp_getobject(self.env, attr.as_ptr() as *const u8)}
    }
    */

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
        unsafe {ffi::setstring(self.env, key.as_bytes(), val.as_bytes(), 0) };
    }

    pub fn setintattr(&mut self, key: &str, val: i64) {
        let key = CString::new(key).unwrap();
        unsafe {ffi::sp_setint(self.env, key.as_ptr(), val) };
    }
}

pub struct Object<'a> {
    obj: ffi::Voidptr,
    key: &'a [u8]
}

impl<'a> Object<'a> {
    pub fn get_value<'b>(&'b mut self) -> Option<&'b[u8]> {
        unsafe {
            let mut sz = 0;
            let valptr = ffi::sp_getstring(self.obj, "value\0".as_ptr(), &mut sz);

            // XXX: what if we stored an empty value?
            if valptr.is_null() {
                return None;
            }

            let s = slice::from_raw_parts(valptr as *const u8, sz as usize);
            return Some(s);
        }
    }
}

impl<'a> Drop for Object<'a> {
    fn drop(&mut self) {
        unsafe {ffi::sp_destroy(self.obj);}
    }
}

pub struct Cursor {
    obj: ffi::Voidptr,
}

pub struct CursorObject<'a> {
    obj: ffi::Voidptr,
    cur: &'a Cursor
}

impl<'a> CursorObject<'a> {
    pub fn get_key<'b>(&'b mut self) -> Option<&'b[u8]> {
        unsafe {
            let mut sz = 0;
            let keyptr = ffi::sp_getstring(self.obj, "key\0".as_ptr(), &mut sz);

            if keyptr.is_null() {
                return None;
            }

            let s = slice::from_raw_parts(keyptr as *const u8, sz as usize);
            return Some(s);
        }
    }

    pub fn get_value<'b>(&'b mut self) -> Option<&'b[u8]> {
        unsafe {
            let mut sz = 0;
            let valptr = ffi::sp_getstring(self.obj, "value\0".as_ptr(), &mut sz);

            // XXX: what if we stored an empty value?
            if valptr.is_null() {
                return None;
            }

            let s = slice::from_raw_parts(valptr as *const u8, sz as usize);
            return Some(s);
        }
    }
}

impl<'a> Drop for CursorObject<'a> {
    fn drop(&mut self) {
        unsafe {ffi::sp_destroy(self.obj);}
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe {ffi::sp_destroy(self.obj);}
    }
}

impl Cursor {
    pub fn next<'a>(&'a self) -> Option<CursorObject<'a>> {
        let res = unsafe { ffi::sp_get(self.obj, ptr::null_mut()) };
        if res.is_null() {
            None
        }
        else {
            Some(CursorObject{obj: res, cur: &self})
        }
    }
}

impl Db {
    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        unsafe {
            //ffi::sp_set_kv(self.db, key.as_ptr(), key.len() as i32, value.as_ptr(), value.len() as i32);
            let obj = ffi::sp_object(self.db);
            assert!(!obj.is_null());
            ffi::setkey(obj, key);
            ffi::setvalue(obj, value);
            ffi::sp_set(self.db, obj);
            ffi::sp_destroy(obj);
        }
    }

    pub fn iter_all(&mut self) -> Cursor {
        unsafe {
            let obj = ffi::sp_object(self.db);
            assert!(!obj.is_null());
            ffi::setstring(obj, "order\0".as_bytes(), ">=\0".as_bytes(), 0);
            let cursor = ffi::sp_cursor(self.db, obj);
            assert!(!cursor.is_null());
            Cursor {obj: cursor}
        }
    }

    pub fn get<'a>(&self, key: &'a [u8]) -> Option<Object<'a>> {
        assert!(key.len() > 0);
        unsafe {
            let pattern = ffi::sp_object(self.db);
            if pattern.is_null() {
                // XXX: Return Err
                return None;
            }
            ffi::setkey(pattern, key);
            let res = ffi::sp_get(self.db, pattern);
            if res.is_null() {
                ffi::sp_destroy(pattern);
                return None;
            }
            return Some(Object{obj: res, key: key});
        }
    }

    // XXX: Cannot distinguish between key not found and no value.
    pub fn get_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.get(key) {
            None => None,
            Some(mut valobj) => {
                match valobj.get_value() {
                    Some(val) => Some(val.to_vec()),
                    None => None
                }
            }
        }
    }
}
