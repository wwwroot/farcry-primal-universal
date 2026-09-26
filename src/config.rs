use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct ModConfig {
    pub enable_survivor_minimap: bool,
    pub enable_minimap_menu_toggle: bool,
    pub unlock_survivor_skills: bool,
    pub skip_intro_screens: bool,
    pub skip_primal_logo: bool,
}

impl Default for ModConfig {
    fn default() -> Self {
        Self {
            enable_survivor_minimap: true,
            enable_minimap_menu_toggle: true,
            unlock_survivor_skills: true,
            skip_intro_screens: true,
            skip_primal_logo: true,
        }
    }
}

pub fn get_bin_directory() -> PathBuf {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleFileNameA(
            hModule: *mut core::ffi::c_void,
            lpFilename: *mut u8,
            nSize: u32,
        ) -> u32;
    }

    let mut buf = [0u8; 1024];
    let len = unsafe { GetModuleFileNameA(core::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as u32) };
    if len > 0 {
        if let Ok(path_str) = core::str::from_utf8(&buf[..len as usize]) {
            let path = PathBuf::from(path_str);
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
    }
    PathBuf::from(".")
}

pub fn load_or_create_config() -> ModConfig {
    let mut config = ModConfig::default();
    let bin_dir = get_bin_directory();
    let ini_path = bin_dir.join("farcryp_universal.ini");

    if !ini_path.exists() {
        let default_ini_content = concat!(
            "[Features]\n",
            "; Restore Minimap HUD and compass navigation in Survivor Mode (true / false)\n",
            "EnableSurvivorMinimap = true\n\n",
            "; Restore Minimap toggle option (ON / OFF) under OPTIONS -> INTERFACE (true / false)\n",
            "EnableMinimapMenuToggle = true\n\n",
            "; Unlock restricted Survivor Mode skills (including 'Show Plants' on minimap) (true / false)\n",
            "UnlockSurvivorSkills = true\n\n",
            "; Skip unskippable Ubisoft Logo and startup warning screens (Epilepsy & AutoSave) (true / false)\n",
            "SkipIntroScreens = true\n\n",
            "; Skip Far Cry Primal short logo movie on game boot (true / false)\n",
            "SkipPrimalLogo = true\n"
        );
        let _ = std::fs::write(&ini_path, default_ini_content);
        return config;
    }

    if let Ok(content) = std::fs::read_to_string(&ini_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.starts_with('[') {
                continue;
            }

            if let Some((key, val_part)) = trimmed.split_once('=') {
                let key = key.trim();
                let val_clean = val_part.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
                let is_true = val_clean == "true" || val_clean == "1" || val_clean == "yes" || val_clean == "on";

                match key {
                    "EnableSurvivorMinimap" => config.enable_survivor_minimap = is_true,
                    "EnableMinimapMenuToggle" => config.enable_minimap_menu_toggle = is_true,
                    "UnlockSurvivorSkills" => config.unlock_survivor_skills = is_true,
                    "SkipIntroScreens" => config.skip_intro_screens = is_true,
                    "SkipPrimalLogo" => config.skip_primal_logo = is_true,
                    _ => {}
                }
            }
        }
    }

    config
}
