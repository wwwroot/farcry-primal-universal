use core::ffi::c_void;

type FARPROC = usize;
type HMODULE = *mut c_void;

#[link(name = "kernel32")]
extern "system" {
    fn GetSystemDirectoryA(lpBuffer: *mut u8, uSize: u32) -> u32;
    fn LoadLibraryA(lpLibFileName: *const u8) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: *const u8) -> FARPROC;
}

static mut REAL_DLL: HMODULE = core::ptr::null_mut();

static mut FN_GETFILEVERSIONINFOA: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOBYHANDLE: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOEXA: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOEXW: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOSIZEA: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOSIZEEXA: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOSIZEEXW: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOSIZEW: FARPROC = 0;
static mut FN_GETFILEVERSIONINFOW: FARPROC = 0;
static mut FN_VERFINDFILEA: FARPROC = 0;
static mut FN_VERFINDFILEW: FARPROC = 0;
static mut FN_VERINSTALLFILEA: FARPROC = 0;
static mut FN_VERINSTALLFILEW: FARPROC = 0;
static mut FN_VERLANGUAGENAMEA: FARPROC = 0;
static mut FN_VERLANGUAGENAMEW: FARPROC = 0;
static mut FN_VERQUERYVALUEA: FARPROC = 0;
static mut FN_VERQUERYVALUEW: FARPROC = 0;

pub unsafe fn init_proxy() {
    let mut sys_dir = [0u8; 260];
    let len = GetSystemDirectoryA(sys_dir.as_mut_ptr(), sys_dir.len() as u32) as usize;
    if len == 0 || len + 13 >= sys_dir.len() {
        return;
    }

    let suffix = b"\\version.dll\0";
    core::ptr::copy_nonoverlapping(suffix.as_ptr(), sys_dir.as_mut_ptr().add(len), suffix.len());

    REAL_DLL = LoadLibraryA(sys_dir.as_ptr());
    if REAL_DLL.is_null() {
        return;
    }

    FN_GETFILEVERSIONINFOA = GetProcAddress(REAL_DLL, b"GetFileVersionInfoA\0".as_ptr());
    FN_GETFILEVERSIONINFOBYHANDLE = GetProcAddress(REAL_DLL, b"GetFileVersionInfoByHandle\0".as_ptr());
    FN_GETFILEVERSIONINFOEXA = GetProcAddress(REAL_DLL, b"GetFileVersionInfoExA\0".as_ptr());
    FN_GETFILEVERSIONINFOEXW = GetProcAddress(REAL_DLL, b"GetFileVersionInfoExW\0".as_ptr());
    FN_GETFILEVERSIONINFOSIZEA = GetProcAddress(REAL_DLL, b"GetFileVersionInfoSizeA\0".as_ptr());
    FN_GETFILEVERSIONINFOSIZEEXA = GetProcAddress(REAL_DLL, b"GetFileVersionInfoSizeExA\0".as_ptr());
    FN_GETFILEVERSIONINFOSIZEEXW = GetProcAddress(REAL_DLL, b"GetFileVersionInfoSizeExW\0".as_ptr());
    FN_GETFILEVERSIONINFOSIZEW = GetProcAddress(REAL_DLL, b"GetFileVersionInfoSizeW\0".as_ptr());
    FN_GETFILEVERSIONINFOW = GetProcAddress(REAL_DLL, b"GetFileVersionInfoW\0".as_ptr());
    FN_VERFINDFILEA = GetProcAddress(REAL_DLL, b"VerFindFileA\0".as_ptr());
    FN_VERFINDFILEW = GetProcAddress(REAL_DLL, b"VerFindFileW\0".as_ptr());
    FN_VERINSTALLFILEA = GetProcAddress(REAL_DLL, b"VerInstallFileA\0".as_ptr());
    FN_VERINSTALLFILEW = GetProcAddress(REAL_DLL, b"VerInstallFileW\0".as_ptr());
    FN_VERLANGUAGENAMEA = GetProcAddress(REAL_DLL, b"VerLanguageNameA\0".as_ptr());
    FN_VERLANGUAGENAMEW = GetProcAddress(REAL_DLL, b"VerLanguageNameW\0".as_ptr());
    FN_VERQUERYVALUEA = GetProcAddress(REAL_DLL, b"VerQueryValueA\0".as_ptr());
    FN_VERQUERYVALUEW = GetProcAddress(REAL_DLL, b"VerQueryValueW\0".as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoA(a: *const u8, b: u32, c: u32, d: *mut c_void) -> i32 {
    if FN_GETFILEVERSIONINFOA != 0 {
        let f: extern "system" fn(*const u8, u32, u32, *mut c_void) -> i32 = core::mem::transmute(FN_GETFILEVERSIONINFOA);
        f(a, b, c, d)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoByHandle(a: *mut c_void, b: *mut c_void, c: *mut c_void) -> i32 {
    if FN_GETFILEVERSIONINFOBYHANDLE != 0 {
        let f: extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32 = core::mem::transmute(FN_GETFILEVERSIONINFOBYHANDLE);
        f(a, b, c)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoExA(a: u32, b: *const u8, c: u32, d: u32, e: *mut c_void) -> i32 {
    if FN_GETFILEVERSIONINFOEXA != 0 {
        let f: extern "system" fn(u32, *const u8, u32, u32, *mut c_void) -> i32 = core::mem::transmute(FN_GETFILEVERSIONINFOEXA);
        f(a, b, c, d, e)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoExW(a: u32, b: *const u16, c: u32, d: u32, e: *mut c_void) -> i32 {
    if FN_GETFILEVERSIONINFOEXW != 0 {
        let f: extern "system" fn(u32, *const u16, u32, u32, *mut c_void) -> i32 = core::mem::transmute(FN_GETFILEVERSIONINFOEXW);
        f(a, b, c, d, e)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeA(a: *const u8, b: *mut u32) -> u32 {
    if FN_GETFILEVERSIONINFOSIZEA != 0 {
        let f: extern "system" fn(*const u8, *mut u32) -> u32 = core::mem::transmute(FN_GETFILEVERSIONINFOSIZEA);
        f(a, b)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeExA(a: u32, b: *const u8, c: *mut u32) -> u32 {
    if FN_GETFILEVERSIONINFOSIZEEXA != 0 {
        let f: extern "system" fn(u32, *const u8, *mut u32) -> u32 = core::mem::transmute(FN_GETFILEVERSIONINFOSIZEEXA);
        f(a, b, c)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeExW(a: u32, b: *const u16, c: *mut u32) -> u32 {
    if FN_GETFILEVERSIONINFOSIZEEXW != 0 {
        let f: extern "system" fn(u32, *const u16, *mut u32) -> u32 = core::mem::transmute(FN_GETFILEVERSIONINFOSIZEEXW);
        f(a, b, c)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeW(a: *const u16, b: *mut u32) -> u32 {
    if FN_GETFILEVERSIONINFOSIZEW != 0 {
        let f: extern "system" fn(*const u16, *mut u32) -> u32 = core::mem::transmute(FN_GETFILEVERSIONINFOSIZEW);
        f(a, b)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoW(a: *const u16, b: u32, c: u32, d: *mut c_void) -> i32 {
    if FN_GETFILEVERSIONINFOW != 0 {
        let f: extern "system" fn(*const u16, u32, u32, *mut c_void) -> i32 = core::mem::transmute(FN_GETFILEVERSIONINFOW);
        f(a, b, c, d)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerFindFileA(a: u32, b: *const u8, c: *const u8, d: *const u8, e: *mut u8, f: *mut u32, g: *mut u8, h: *mut u32) -> i32 {
    if FN_VERFINDFILEA != 0 {
        let func: extern "system" fn(u32, *const u8, *const u8, *const u8, *mut u8, *mut u32, *mut u8, *mut u32) -> i32 = core::mem::transmute(FN_VERFINDFILEA);
        func(a, b, c, d, e, f, g, h)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerFindFileW(a: u32, b: *const u16, c: *const u16, d: *const u16, e: *mut u16, f: *mut u32, g: *mut u16, h: *mut u32) -> i32 {
    if FN_VERFINDFILEW != 0 {
        let func: extern "system" fn(u32, *const u16, *const u16, *const u16, *mut u16, *mut u32, *mut u16, *mut u32) -> i32 = core::mem::transmute(FN_VERFINDFILEW);
        func(a, b, c, d, e, f, g, h)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerInstallFileA(a: u32, b: *const u8, c: *const u8, d: *const u8, e: *const u8, f: *const u8, g: *mut u8, h: *mut u32) -> u32 {
    if FN_VERINSTALLFILEA != 0 {
        let func: extern "system" fn(u32, *const u8, *const u8, *const u8, *const u8, *const u8, *mut u8, *mut u32) -> u32 = core::mem::transmute(FN_VERINSTALLFILEA);
        func(a, b, c, d, e, f, g, h)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerInstallFileW(a: u32, b: *const u16, c: *const u16, d: *const u16, e: *const u16, f: *const u16, g: *mut u16, h: *mut u32) -> u32 {
    if FN_VERINSTALLFILEW != 0 {
        let func: extern "system" fn(u32, *const u16, *const u16, *const u16, *const u16, *const u16, *mut u16, *mut u32) -> u32 = core::mem::transmute(FN_VERINSTALLFILEW);
        func(a, b, c, d, e, f, g, h)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerLanguageNameA(a: u32, b: *mut u8, c: u32) -> u32 {
    if FN_VERLANGUAGENAMEA != 0 {
        let f: extern "system" fn(u32, *mut u8, c: u32) -> u32 = core::mem::transmute(FN_VERLANGUAGENAMEA);
        f(a, b, c)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerLanguageNameW(a: u32, b: *mut u16, c: u32) -> u32 {
    if FN_VERLANGUAGENAMEW != 0 {
        let f: extern "system" fn(u32, *mut u16, c: u32) -> u32 = core::mem::transmute(FN_VERLANGUAGENAMEW);
        f(a, b, c)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerQueryValueA(a: *const c_void, b: *const u8, c: *mut *mut c_void, d: *mut u32) -> i32 {
    if FN_VERQUERYVALUEA != 0 {
        let f: extern "system" fn(*const c_void, *const u8, *mut *mut c_void, *mut u32) -> i32 = core::mem::transmute(FN_VERQUERYVALUEA);
        f(a, b, c, d)
    } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn VerQueryValueW(a: *const c_void, b: *const u16, c: *mut *mut c_void, d: *mut u32) -> i32 {
    if FN_VERQUERYVALUEW != 0 {
        let f: extern "system" fn(*const c_void, *const u16, *mut *mut c_void, *mut u32) -> i32 = core::mem::transmute(FN_VERQUERYVALUEW);
        f(a, b, c, d)
    } else { 0 }
}
