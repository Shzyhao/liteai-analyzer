//! Windows 右键菜单集成：写入 HKCU\Software\Classes\*\shell\轻析（免管理员）。
//! Win10 经典菜单与 Win11"显示更多选项"均生效。

use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

const SHELL_KEY: &str = r"Software\Classes\*\shell\轻析";
const MENU_NAME: &str = "AI 分析";

/// 注册右键菜单。
pub fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (shell_key, _disp) = hkcu.create_subkey(SHELL_KEY).map_err(|e| e.to_string())?;
    shell_key.set_value("", &MENU_NAME).map_err(|e| e.to_string())?;
    // Icon 值格式为 "路径",索引 —— 路径含空格需加引号，索引不加引号
    shell_key.set_value("Icon", &format!("\"{}\",0", exe.display())).map_err(|e| e.to_string())?;
    let (cmd, _disp) = shell_key.create_subkey("command").map_err(|e| e.to_string())?;
    cmd.set_value("", &format!("\"{}\" \"%1\"", exe.display())).map_err(|e| e.to_string())?;
    Ok(())
}

/// 卸载右键菜单。
pub fn unregister() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.delete_subkey_all(r"Software\Classes\*\shell\轻析")
        .map_err(|e| e.to_string())
}

/// 当前是否已注册。
pub fn is_registered() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(SHELL_KEY)
        .is_ok()
}
