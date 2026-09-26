#![windows_subsystem = "windows"]

use core::ffi::c_void;

mod config;
mod proxy;

type BOOL = i32;
type DWORD = u32;
type HINSTANCE = *mut c_void;
type LPVOID = *mut c_void;
type HANDLE = *mut c_void;
type SizeT = usize;

const DLL_PROCESS_ATTACH: DWORD = 1;
const PAGE_EXECUTE_READWRITE: DWORD = 0x40;
const MEM_COMMIT: DWORD = 0x1000;

#[repr(C)]
struct MEMORY_BASIC_INFORMATION {
    base_address: *mut c_void,
    allocation_base: *mut c_void,
    allocation_protect: DWORD,
    partition_id: u16,
    region_size: SizeT,
    state: DWORD,
    protect: DWORD,
    type_: DWORD,
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateThread(
        lpThreadAttributes: *mut c_void,
        dwStackSize: SizeT,
        lpStartAddress: unsafe extern "system" fn(LPVOID) -> DWORD,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    fn DisableThreadLibraryCalls(hLibModule: HINSTANCE) -> BOOL;
    fn Sleep(dwMilliseconds: DWORD);
    fn GetCurrentProcess() -> HANDLE;
    fn VirtualProtect(
        lpAddress: LPVOID,
        dwSize: SizeT,
        flNewProtect: DWORD,
        lpflOldProtect: *mut DWORD,
    ) -> BOOL;
    fn VirtualQuery(
        lpAddress: *const c_void,
        lpBuffer: *mut MEMORY_BASIC_INFORMATION,
        dwLength: SizeT,
    ) -> SizeT;
    fn FlushInstructionCache(hProcess: HANDLE, lpBaseAddress: *const c_void, dwSize: SizeT) -> BOOL;
    fn CreateFileA(
        lpFileName: *const u8,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const u8,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: *mut c_void,
    ) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
}

struct Logger {
    handle: HANDLE,
}

impl Logger {
    fn open() -> Self {
        const GENERIC_WRITE: DWORD = 0x40000000;
        const CREATE_ALWAYS: DWORD = 2;
        const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;
        let bin_dir = config::get_bin_directory();
        let log_path = bin_dir.join("farcryp_universal.log");
        let path_str = log_path.to_str().unwrap_or("farcryp_universal.log");
        let mut path_bytes = path_str.as_bytes().to_vec();
        path_bytes.push(0);

        let handle = unsafe {
            CreateFileA(
                path_bytes.as_ptr(),
                GENERIC_WRITE,
                1, // FILE_SHARE_READ
                core::ptr::null_mut(),
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                core::ptr::null_mut(),
            )
        };
        Logger { handle }
    }

    fn write(&self, msg: &str) {
        if self.handle as isize == -1 || self.handle.is_null() {
            return;
        }
        let mut written: DWORD = 0;
        unsafe {
            WriteFile(
                self.handle,
                msg.as_ptr(),
                msg.len() as DWORD,
                &mut written,
                core::ptr::null_mut(),
            );
        }
    }

    fn writeln(&self, msg: &str) {
        self.write(msg);
        self.write("\r\n");
    }

    fn close(self) {
        if self.handle as isize != -1 && !self.handle.is_null() {
            unsafe { CloseHandle(self.handle); }
        }
    }
}

unsafe fn write_memory(target: *mut u8, patch: &[u8]) -> bool {
    let mut old_protect: DWORD = 0;
    if VirtualProtect(
        target as LPVOID,
        patch.len(),
        PAGE_EXECUTE_READWRITE,
        &mut old_protect,
    ) == 0 {
        return false;
    }
    core::ptr::copy_nonoverlapping(patch.as_ptr(), target, patch.len());
    let mut temp: DWORD = 0;
    VirtualProtect(
        target as LPVOID,
        patch.len(),
        old_protect,
        &mut temp,
    );
    true
}

unsafe fn scan_pattern(base: *const u8, size: usize, pattern: &[u8]) -> Option<*mut u8> {
    if size < pattern.len() {
        return None;
    }
    let max = size - pattern.len();
    for i in 0..max {
        let p = base.add(i);
        let mut match_found = true;
        for j in 0..pattern.len() {
            if *p.add(j) != pattern[j] {
                match_found = false;
                break;
            }
        }
        if match_found {
            return Some(p as *mut u8);
        }
    }
    None
}

unsafe fn scan_process_memory(pattern: &[u8]) -> Option<*mut u8> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleA(lpModuleName: *const u8) -> *mut c_void;
    }

    let base = GetModuleHandleA(core::ptr::null()) as usize;
    if base == 0 {
        return None;
    }

    let e_lfanew = *((base + 0x3C) as *const i32) as usize;
    let size_of_image = *((base + e_lfanew + 0x18 + 0x38) as *const u32) as usize;

    let mut addr = base;
    let end_addr = base + size_of_image;
    let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

    while addr < end_addr {
        let ret = VirtualQuery(
            addr as *const c_void,
            &mut mbi,
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        if ret == 0 {
            addr += 0x1000;
            continue;
        }

        let readable = (mbi.protect & 0x02 != 0) // PAGE_READONLY
            || (mbi.protect & 0x04 != 0) // PAGE_READWRITE
            || (mbi.protect & 0x20 != 0) // PAGE_EXECUTE_READ
            || (mbi.protect & 0x40 != 0); // PAGE_EXECUTE_READWRITE

        if mbi.state == MEM_COMMIT && readable && (mbi.protect & 0x01 == 0) {
            if let Some(p) = scan_pattern(mbi.base_address as *const u8, mbi.region_size, pattern) {
                return Some(p);
            }
        }

        addr = (mbi.base_address as usize) + mbi.region_size;
    }
    None
}

unsafe extern "system" fn worker_thread(_param: LPVOID) -> DWORD {
    let logger = Logger::open();
    logger.writeln("==================================================");
    logger.writeln("Far Cry Primal Universal Mod - v1.0.0");
    logger.writeln("Crash-Proof Survivor Minimap Restoration Engine");
    logger.writeln("==================================================");

    let cfg = config::load_or_create_config();
    logger.writeln("[*] Loaded Configuration (farcryp_universal.ini):");
    logger.writeln(if cfg.enable_survivor_minimap { "    EnableSurvivorMinimap   = true" } else { "    EnableSurvivorMinimap   = false" });
    logger.writeln(if cfg.enable_minimap_menu_toggle { "    EnableMinimapMenuToggle = true" } else { "    EnableMinimapMenuToggle = false" });
    logger.writeln(if cfg.unlock_survivor_skills { "    UnlockSurvivorSkills    = true" } else { "    UnlockSurvivorSkills    = false" });
    logger.writeln(if cfg.skip_intro_screens { "    SkipIntroScreens        = true" } else { "    SkipIntroScreens        = false" });
    logger.writeln(if cfg.skip_primal_logo { "    SkipPrimalLogo          = true" } else { "    SkipPrimalLogo          = false" });
    logger.writeln("==================================================");

    logger.writeln("[*] Waiting for Denuvo code decryption...");

    // Signatures
    let sig_enabled = [0x53, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x89, 0xCB, 0x84, 0xD2, 0x75, 0x34];
    let sig_opacity = [0x53, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x89, 0xCB, 0x84, 0xD2, 0x75, 0x31];
    let sig_hostile = [0x53, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x89, 0xCB, 0x84, 0xD2, 0x75, 0x32];
    let sig_trail = [0x48, 0x83, 0xEC, 0x28, 0x84, 0xD2, 0x74, 0x07, 0x30, 0xC0, 0x48, 0x83, 0xC4, 0x28, 0xC3];
    let sig_caller = [0x0F, 0xB6, 0x88, 0x9B, 0x01, 0x00, 0x00, 0x88, 0x8B, 0x88, 0x13, 0x00, 0x00];
    let sig_fcc_logo = [0x57, 0x48, 0x83, 0xEC, 0x50, 0x80, 0x79, 0x11, 0x00, 0x48, 0x89, 0xCF, 0x75, 0x7B];
    let sig_initial_screens = [0x0F, 0xB6, 0x81, 0xA1, 0x00, 0x00, 0x00, 0xC3];

    let mut attempts = 0;
    let max_attempts = 120; // 60 seconds

    loop {
        attempts += 1;
        if attempts > max_attempts {
            logger.writeln("[-] Timed out waiting for decrypted game code.");
            break;
        }

        // Check if the caller signature is ready
        if let Some(caller_ptr) = scan_process_memory(&sig_caller) {
            logger.writeln("[+] Decrypted game code detected!");

            // 1. Minimap Menu Toggle: IsMinimapEnabled evaluated in OPTIONS -> INTERFACE
            if cfg.enable_minimap_menu_toggle {
                if let Some(p) = scan_process_memory(&sig_enabled) {
                    write_memory(p.add(0x33), &[0xEB, 0x0B]);
                    logger.writeln("[+] Patched: IsMinimapEnabled -> Normal Mode Evaluation");
                }
            }

            // 2. Survivor Minimap: Opacity & Hostile Icons
            if cfg.enable_survivor_minimap {
                if let Some(p) = scan_process_memory(&sig_opacity) {
                    write_memory(p.add(0x33), &[0xEB, 0x08]);
                    logger.writeln("[+] Patched: MinimapOpacity -> Normal Mode Evaluation");
                }
                if let Some(p) = scan_process_memory(&sig_hostile) {
                    write_memory(p.add(0x33), &[0xEB, 0x09]);
                    logger.writeln("[+] Patched: HostileIcon -> Normal Mode Evaluation");
                }
            }

            // 3. Survivor Skills: PlayerTrail & Show Plants skill unlock
            if cfg.unlock_survivor_skills {
                if let Some(p) = scan_process_memory(&sig_trail) {
                    write_memory(p.add(0x36), &[0x31, 0xD2, 0x90]);
                    logger.writeln("[+] Patched: PlayerTrail & Survivor Skills Unlocked");
                }
            }

            // 4. HUD Caller: force ecx=1 and unconditional jump to minimap updater
            // (Preserves 74 78 exit check so game never crashes on death/main menu)
            if cfg.enable_survivor_minimap {
                write_memory(caller_ptr, &[0xB9, 0x01, 0x00, 0x00, 0x00, 0x90, 0x90]);
                write_memory(caller_ptr.add(0x1E), &[0xEB, 0x09]);
                logger.writeln("[+] Patched: HUD Caller Pipeline -> Unlocked (Crash-Proof)");
            }

            // 5. Fast Startup: Native Initial Screens Skip (0x144BC6F60)
            // Forces IsShowInitialScreens() to return 0 (false) -> takes native skip branch at 0x144DED31A
            // Bypasses Ubisoft Logo, SelectLanguage, AutoSave Warning, and Epilepsy Warning cleanly!
            if cfg.skip_intro_screens {
                if let Some(p) = scan_process_memory(&sig_initial_screens) {
                    write_memory(p, &[0x31, 0xC0, 0xC3]);
                    logger.writeln("[+] Patched: Fast Startup -> Initial Screens (Logos & Warnings) Skipped");
                }
            }

            // 6. Fast Startup: Skip Far Cry Primal Short Logo video (0x1452CB8B0)
            if cfg.skip_primal_logo {
                if let Some(p) = scan_process_memory(&sig_fcc_logo) {
                    write_memory(p, &[0xC6, 0x41, 0x11, 0x01, 0xC3]);
                    logger.writeln("[+] Patched: Fast Startup -> FCC Logo Skipped");
                }
            }

            FlushInstructionCache(GetCurrentProcess(), core::ptr::null(), 0);
            logger.writeln("==================================================");
            logger.writeln("[SUCCESS] Far Cry Primal Universal is fully active!");
            logger.writeln("All configured modules have been applied cleanly.");
            logger.writeln("==================================================");
            break;
        }

        Sleep(500);
    }

    logger.close();
    0
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: DWORD,
    _reserved: LPVOID,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        DisableThreadLibraryCalls(hinst);
        proxy::init_proxy();
        CreateThread(
            core::ptr::null_mut(),
            0,
            worker_thread,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
    }
    1
}
