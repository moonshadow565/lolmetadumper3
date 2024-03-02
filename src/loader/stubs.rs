extern crate libc;
use libc::{off_t, size_t, RTLD_NOW};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{self, NonNull};

lazy_static! {
    static ref DLLS: Box<[usize]> = {
        [
            "libc++.so.1\0",
            "libc++abi.so.1\0",
            "libc++.so\0",
            "libc++abi.so\0",
        ]
        .iter()
        .map(|s| {
            let result = unsafe { libc::dlopen(s.as_ptr() as _, RTLD_NOW) as usize };
            result
        })
        .filter(|x| *x != 0)
        .collect()
    };
}

#[repr(C)]
struct RuneEntry {
    min: u32,
    max: u32,
    map: u32,
    types: Option<NonNull<*mut u32>>,
}

#[repr(C)]
struct RuneRange {
    nranges: c_int,
    ranges: Option<NonNull<*mut RuneEntry>>,
}

#[repr(C)]
struct RuneCharClass {
    name: [u8; 14],
    mask: u32,
}

#[repr(C)]
struct RuneLocale {
    magic: [u8; 8],
    encoding: [u8; 32],
    sgetrune: Option<unsafe extern "C" fn(*const c_char, usize, *const *const c_char) -> u32>,
    sputrune: Option<unsafe extern "C" fn(u32, *mut c_char, usize, *const *mut c_char) -> c_int>,
    invalid_rune: u32,
    runetype: [u32; 256],
    maplower: [u32; 256],
    mapupper: [u32; 256],
    runetype_ext: RuneRange,
    maplower_ext: RuneRange,
    mapupper_ext: RuneRange,
    variable: Option<NonNull<*mut c_void>>,
    variable_len: c_int,
    ncharclasses: c_int,
    charclasses: Option<NonNull<*mut RuneCharClass>>,
}

static mut STUB_DEFAULT_RUNE_LOCALE: RuneLocale = RuneLocale {
    magic: *b"RuneMagA",
    encoding: *b"NONE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    sgetrune: None,
    sputrune: None,
    invalid_rune: 0x0FFFD,
    runetype: [
        0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x24200, 0x4200, 0x4200,
        0x4200, 0x4200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200,
        0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x200, 0x64000, 0x42800, 0x42800, 0x42800,
        0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800,
        0x42800, 0x42800, 0x50C00, 0x50C01, 0x50C02, 0x50C03, 0x50C04, 0x50C05, 0x50C06, 0x50C07,
        0x50C08, 0x50C09, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800, 0x5890A,
        0x5890B, 0x5890C, 0x5890D, 0x5890E, 0x5890F, 0x48900, 0x48900, 0x48900, 0x48900, 0x48900,
        0x48900, 0x48900, 0x48900, 0x48900, 0x48900, 0x48900, 0x48900, 0x48900, 0x48900, 0x48900,
        0x48900, 0x48900, 0x48900, 0x48900, 0x48900, 0x42800, 0x42800, 0x42800, 0x42800, 0x42800,
        0x42800, 0x5190A, 0x5190B, 0x5190C, 0x5190D, 0x5190E, 0x5190F, 0x41900, 0x41900, 0x41900,
        0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900,
        0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x41900, 0x42800, 0x42800, 0x42800,
        0x42800, 0x200, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    maplower: [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13,
        0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22,
        0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31,
        0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40,
        0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
        0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x5B, 0x5C, 0x5D, 0x5E,
        0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D,
        0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C,
        0x7D, 0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B,
        0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A,
        0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0x0A0, 0x0A1, 0x0A2, 0x0A3, 0x0A4, 0x0A5, 0x0A6, 0x0A7,
        0x0A8, 0x0A9, 0x0AA, 0x0AB, 0x0AC, 0x0AD, 0x0AE, 0x0AF, 0x0B0, 0x0B1, 0x0B2, 0x0B3, 0x0B4,
        0x0B5, 0x0B6, 0x0B7, 0x0B8, 0x0B9, 0x0BA, 0x0BB, 0x0BC, 0x0BD, 0x0BE, 0x0BF, 0x0C0, 0x0C1,
        0x0C2, 0x0C3, 0x0C4, 0x0C5, 0x0C6, 0x0C7, 0x0C8, 0x0C9, 0x0CA, 0x0CB, 0x0CC, 0x0CD, 0x0CE,
        0x0CF, 0x0D0, 0x0D1, 0x0D2, 0x0D3, 0x0D4, 0x0D5, 0x0D6, 0x0D7, 0x0D8, 0x0D9, 0x0DA, 0x0DB,
        0x0DC, 0x0DD, 0x0DE, 0x0DF, 0x0E0, 0x0E1, 0x0E2, 0x0E3, 0x0E4, 0x0E5, 0x0E6, 0x0E7, 0x0E8,
        0x0E9, 0x0EA, 0x0EB, 0x0EC, 0x0ED, 0x0EE, 0x0EF, 0x0F0, 0x0F1, 0x0F2, 0x0F3, 0x0F4, 0x0F5,
        0x0F6, 0x0F7, 0x0F8, 0x0F9, 0x0FA, 0x0FB, 0x0FC, 0x0FD, 0x0FE, 0x0FF,
    ],
    mapupper: [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13,
        0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22,
        0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31,
        0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40,
        0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
        0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E,
        0x5F, 0x60, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D,
        0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x7B, 0x7C,
        0x7D, 0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B,
        0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A,
        0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0x0A0, 0x0A1, 0x0A2, 0x0A3, 0x0A4, 0x0A5, 0x0A6, 0x0A7,
        0x0A8, 0x0A9, 0x0AA, 0x0AB, 0x0AC, 0x0AD, 0x0AE, 0x0AF, 0x0B0, 0x0B1, 0x0B2, 0x0B3, 0x0B4,
        0x0B5, 0x0B6, 0x0B7, 0x0B8, 0x0B9, 0x0BA, 0x0BB, 0x0BC, 0x0BD, 0x0BE, 0x0BF, 0x0C0, 0x0C1,
        0x0C2, 0x0C3, 0x0C4, 0x0C5, 0x0C6, 0x0C7, 0x0C8, 0x0C9, 0x0CA, 0x0CB, 0x0CC, 0x0CD, 0x0CE,
        0x0CF, 0x0D0, 0x0D1, 0x0D2, 0x0D3, 0x0D4, 0x0D5, 0x0D6, 0x0D7, 0x0D8, 0x0D9, 0x0DA, 0x0DB,
        0x0DC, 0x0DD, 0x0DE, 0x0DF, 0x0E0, 0x0E1, 0x0E2, 0x0E3, 0x0E4, 0x0E5, 0x0E6, 0x0E7, 0x0E8,
        0x0E9, 0x0EA, 0x0EB, 0x0EC, 0x0ED, 0x0EE, 0x0EF, 0x0F0, 0x0F1, 0x0F2, 0x0F3, 0x0F4, 0x0F5,
        0x0F6, 0x0F7, 0x0F8, 0x0F9, 0x0FA, 0x0FB, 0x0FC, 0x0FD, 0x0FE, 0x0FF,
    ],
    runetype_ext: RuneRange {
        nranges: 0,
        ranges: None,
    },
    maplower_ext: RuneRange {
        nranges: 0,
        ranges: None,
    },
    mapupper_ext: RuneRange {
        nranges: 0,
        ranges: None,
    },
    variable: None,
    variable_len: 0,
    ncharclasses: 0,
    charclasses: None,
};

static mut STUB_STACK_CHK_GUARD: [i64; 8] = [0; 8];

static mut MACH_TASK_SELF: *const c_void = ptr::null();

unsafe extern "C" fn stub_atexit(_lpfunc: Option<unsafe extern "C" fn()>) -> c_int {
    // NOTE: do nothing, don't care about exits
    0
}

unsafe extern "C" fn stub_cxa_atexit(
    _lpfunc: Option<unsafe extern "C" fn(*mut c_void)>,
    _obj: *mut c_void,
    _lpdso_handle: *mut c_void,
) -> c_int {
    // NOTE: do nothing, don't care about exits
    0
}

unsafe extern "C" fn stub_bzero(dst: *mut c_void, size: size_t) {
    libc::memset(dst, 0, size);
}

unsafe extern "C" fn stub_mmap(
    addr: *mut c_void,
    len: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    let flags = (flags & 0x1f) | (flags & 0x1000 != 0).then(|| 0x20).unwrap_or(0);
    libc::mmap(addr, len, prot, flags, fd, offset)
}

unsafe extern "C" fn stub_dlsym(_handle: *mut c_void, symbol: *const c_char) -> *const c_void {
    if symbol == ptr::null() {
        return ptr::null();
    }
    resolve_impl(CStr::from_ptr(symbol))
}

unsafe extern "C" fn stub_dladdr(_c: *mut c_void, _info: *mut c_void) -> c_int {
    // sentry/crashpad bs, allways fail, dont care
    0
}

unsafe extern "C" fn stub_tolower(c: c_int) -> c_int {
    libc::tolower(c)
}

unsafe extern "C" fn stub_dlclose(_handle: *mut c_void) -> c_int {
    0 // Always success
}

unsafe extern "C" fn stub_dlerror() -> *const c_char {
    b"Not implemented!\0".as_ptr() as *const c_char
}

unsafe extern "C" fn stub_dlopen(path: *const c_char, _mode: c_int) -> *mut c_void {
    if path.is_null() {
        return ptr::null_mut();
    }
    match std::ffi::CStr::from_ptr(path).to_str() {
        Ok(s) if s.contains("libc") => "libc\0".as_ptr() as *mut c_void,
        _ => ptr::null_mut(),
    }
}

unsafe extern "C" fn stub_pthread_ignore() -> c_int {
    // NOTE: no threading support
    0
}

unsafe extern "C" fn stub_task_info(
    _task: *const c_void,
    _flavor: u32,
    _info_out: *mut i32,
    _info_out_cnt: *mut u32,
) -> u32 {
    // TODO:
    return 0;
}

unsafe fn resolve_impl(name: &CStr) -> *const c_void {
    match name.to_bytes() {
        b"_DefaultRuneLocale" => &STUB_DEFAULT_RUNE_LOCALE as *const _ as *const c_void,
        b"__stack_chk_guard" => &STUB_STACK_CHK_GUARD as *const _ as *const c_void,
        b"mach_task_self_" => &MACH_TASK_SELF as *const _ as *const c_void,
        b"__stderrp" => libc::dlsym(ptr::null_mut(), b"stderr\0".as_ptr() as *const _),
        b"__stdinp" => libc::dlsym(ptr::null_mut(), b"stdin\0".as_ptr() as *const _),
        b"__stdoutp" => libc::dlsym(ptr::null_mut(), b"stdout\0".as_ptr() as *const _),
        b"atexit" => stub_atexit as *const () as *const c_void,
        b"__cxa_atexit" => stub_cxa_atexit as *const () as *const c_void,
        b"__tolower" => stub_tolower as *const () as *const c_void,
        b"__bzero" => stub_bzero as *const () as *const c_void,
        b"mmap" => stub_mmap as *const () as *const c_void,
        b"dladdr" => stub_dladdr as *const () as *const c_void,
        b"dlclose" => stub_dlclose as *const () as *const c_void,
        b"dlerror" => stub_dlerror as *const () as *const c_void,
        b"dlopen" => stub_dlopen as *const () as *const c_void,
        b"dlsym" => stub_dlsym as *const () as *const c_void,
        b"pthread_mutex_destroy" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutex_init" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutex_lock" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutex_trylock" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutex_unlock" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutexattr_destroy" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutexattr_init" => stub_pthread_ignore as *const () as *const c_void,
        b"pthread_mutexattr_settype" => stub_pthread_ignore as *const () as *const c_void,
        b"task_info" => stub_task_info as *const () as *const c_void,
        _ => {
            // These would need more complex to handle properly, don't really care about them for just init.
            // Just allways resolve them to null, that way diagnosing the crash gets much easier.
            const BLOCKLIST: &[&str] = &[
                // block pthread
                "pthread_",
                // block macos apis
                "mach_",
                "dispatch_",
                // block dyld stubs
                "tlv_bootstrap",
                "dyld_",
                // block signals
                "sigaction",
                "signal",
                // block fork
                "fork",
                "posix_spawn",
                // block filesystem
                "opendir",
                "readdir",
                "closedir",
                "fstat",
                "lstat",
                "stat",
            ];
            for blocked in BLOCKLIST {
                if name.to_bytes().starts_with(blocked.as_bytes()) {
                    return ptr::null();
                }
            }
            for &dll in DLLS.iter() {
                let result = libc::dlsym(dll as _, name.as_ptr());
                if result != ptr::null_mut() {
                    return result as _;
                }
            }
            libc::dlsym(ptr::null_mut(), name.as_ptr())
        }
    }
}

pub fn resolve(name: &str) -> usize {
    lazy_static::initialize(&DLLS);

    let name = if name.starts_with("_") {
        &name[1..]
    } else {
        name
    };
    let cname = CString::new(name).unwrap();
    let result = unsafe { resolve_impl(&cname) as _ };
    result
}
