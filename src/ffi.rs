use libc::{c_void, c_int, c_char};

pub type Voidptr = *mut c_void;

#[link(name = "sophia", kind="static")]
extern {
    pub fn sp_env() -> Voidptr;
    pub fn sp_object(a: Voidptr) -> Voidptr;
    pub fn sp_open(a: Voidptr) -> c_int;
    pub fn sp_drop(a: Voidptr) -> c_int;
    pub fn sp_destroy(a: Voidptr) -> c_int;
    pub fn sp_error(a: Voidptr) -> c_int;
    pub fn sp_asynchronous(a: Voidptr) -> Voidptr;
    pub fn sp_poll(a: Voidptr) -> Voidptr;
    pub fn sp_setobject(a: Voidptr, b: *const c_char, c: Voidptr) -> c_int;
    pub fn sp_setstring(a: Voidptr, b: *const c_char, c: *const c_void, d: c_int) -> c_int;
    pub fn sp_setint(a: Voidptr, b: *const c_char, c: i64) -> c_int;

    pub fn sp_getobject(a: Voidptr, b: *const u8) -> Voidptr;
    pub fn sp_getstring(a: Voidptr, b: *const u8, c: *mut c_int) -> Voidptr;
    pub fn sp_getint(a: Voidptr, b: *mut u8) -> i64;
    pub fn sp_set(a: Voidptr, b: Voidptr) -> c_int;
    pub fn sp_update(a: Voidptr, b: Voidptr) -> c_int;
    pub fn sp_delete(a: Voidptr, b: Voidptr) -> c_int;
    pub fn sp_get(a: Voidptr, b: Voidptr) -> Voidptr;
    pub fn sp_cursor(a: Voidptr, b: Voidptr) -> Voidptr;
    pub fn sp_begin(a: Voidptr) -> Voidptr;
    pub fn sp_prepare(a: Voidptr) -> c_int;
    pub fn sp_commit(a: Voidptr) -> c_int;

    //pub fn sp_set_kv(a: Voidptr, key: *const u8, key_sz: c_int, val: *const u8, val_sz: c_int) -> c_int; 
}

// NOTE: `b' must be NUL terminated string.
#[inline(always)]
pub fn setstring(a: Voidptr, b: &[u8], c: &[u8]) -> c_int {
    assert!(c.len() > 0);
    unsafe { sp_setstring(a, b.as_ptr() as *const c_char, c.as_ptr() as *const c_void, c.len() as i32) }
}

#[inline(always)]
pub fn setkey(o: Voidptr, key: &[u8]) -> c_int {
    setstring(o, "key\0".as_bytes(), key)
}

#[inline(always)]
pub fn setvalue(o: Voidptr, value: &[u8]) -> c_int {
    setstring(o, "value\0".as_bytes(), value)
}
