use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::Local;
use std::process::Command;
use sysinfo::System;
use uuid::Uuid;
use tauri::AppHandle;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")] // camelCase for JS
pub struct AccountInfo {
    pub id: String,
    pub folder_name: String,
    pub display_name: String,
    pub finger_print: String,
    pub last_backup_time: String,
}

fn get_game_data_path() -> Result<PathBuf, String> {
    let appdata = std::env::var("USERPROFILE").map_err(|_| "Failed to get USERPROFILE".to_string())?;
    let local = PathBuf::from(&appdata).join("AppData").join("Local").join("Hypergryph").join("Endfield");
    let locallow = PathBuf::from(&appdata).join("AppData").join("LocalLow").join("Hypergryph").join("Endfield");

    let target_dirs = vec![local.clone(), locallow.clone()];
    
    for base in target_dirs {
        if base.exists() {
            if let Ok(entries) = fs::read_dir(&base) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() && entry.file_name().to_string_lossy().starts_with("sdk") {
                            let login_cache = entry.path().join("login_cache");
                            if login_cache.exists() {
                                return Ok(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err("未找到登录数据目录，请确保游戏已安装并至少运行过一次。".to_string())
}

fn get_app_dir(_app_handle: &AppHandle) -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent().ok_or("Failed to get executable directory")?;
    Ok(exe_dir.to_path_buf())
}

fn get_backup_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = get_app_dir(app_handle)?;
    let backup_dir = app_dir.join("Backups");
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    }
    Ok(backup_dir)
}

fn get_index_file_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = get_app_dir(app_handle)?;
    Ok(app_dir.join("accounts.json"))
}

fn compute_file_hash(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

#[tauri::command]
fn load_accounts(app_handle: tauri::AppHandle) -> Result<Vec<AccountInfo>, String> {
    let index_file = get_index_file_path(&app_handle)?;
    if !index_file.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(index_file).map_err(|e| e.to_string())?;
    let accounts: Vec<AccountInfo> = serde_json::from_str(&content).unwrap_or_else(|_| vec![]);
    Ok(accounts)
}

fn save_accounts(app_handle: &AppHandle, accounts: &[AccountInfo]) -> Result<(), String> {
    let index_file = get_index_file_path(app_handle)?;
    if let Some(parent) = index_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let json = serde_json::to_string_pretty(accounts).map_err(|e| e.to_string())?;
    fs::write(index_file, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn backup_account(app_handle: tauri::AppHandle, note: String) -> Result<AccountInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all();
    for (_pid, process) in sys.processes() {
        if process.name().to_string_lossy().to_lowercase().contains("endfield.exe") {
            return Err("游戏正在运行中，为防止数据损坏，请先退出游戏后再备份！".to_string());
        }
    }

    let game_data_path = get_game_data_path()?;
    let backup_path = get_backup_path(&app_handle)?;
    
    let folder_name = Uuid::new_v4().simple().to_string();
    let target_dir = backup_path.join(&folder_name);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let files = ["login_cache", "login_cache.crc"];
    let mut hash = String::new();

    for file in files {
        let src = game_data_path.join(file);
        let dest = target_dir.join(file);
        if src.exists() {
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            if file == "login_cache" {
                hash = compute_file_hash(&src)?;
            }
        }
    }

    let mut accounts = load_accounts(app_handle.clone())?;

    // Check for duplicates
    if accounts.iter().any(|a| a.finger_print == hash) {
        // Cleanup the folder we just created
        let _ = fs::remove_dir_all(&target_dir);
        return Err("当前账号已被保存，请勿重复备份。".to_string());
    }

    let act = AccountInfo {
        id: Uuid::new_v4().to_string(),
        folder_name,
        display_name: note,
        finger_print: hash,
        last_backup_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    accounts.insert(0, act.clone());
    save_accounts(&app_handle, &accounts)?;

    Ok(act)
}

#[tauri::command]
fn switch_account(app_handle: tauri::AppHandle, folder_name: String) -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_all();
    for (_pid, process) in sys.processes() {
        if process.name().to_string_lossy().to_lowercase().contains("endfield.exe") {
            return Err("游戏正在运行中，为防止数据损坏，请先关闭游戏后再切换账号！".to_string());
        }
    }

    let game_data_path = get_game_data_path()?;
    let backup_path = get_backup_path(&app_handle)?;
    let source_dir = backup_path.join(&folder_name);
    
    if !source_dir.exists() {
        return Err("备份文件夹找不到了！请尝试删除此记录。".to_string());
    }

    let files = ["login_cache", "login_cache.crc"];
    for file in files {
        let game_file = game_data_path.join(file);
        if game_file.exists() {
            fs::remove_file(&game_file).map_err(|e| e.to_string())?;
        }
    }

    for file in files {
        let src = source_dir.join(file);
        let dest = game_data_path.join(file);
        if src.exists() {
            fs::copy(src, dest).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
fn delete_account(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut accounts = load_accounts(app_handle.clone())?;
    
    if let Some(pos) = accounts.iter().position(|a| a.id == id) {
        let act = &accounts[pos];
        let backup_path = get_backup_path(&app_handle)?;
        let dir = backup_path.join(&act.folder_name);
        if dir.exists() {
            fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
        }
        accounts.remove(pos);
        save_accounts(&app_handle, &accounts)?;
    }
    
    Ok(())
}

#[tauri::command]
fn rename_account(app_handle: tauri::AppHandle, id: String, new_name: String) -> Result<(), String> {
    let mut accounts = load_accounts(app_handle.clone())?;
    if let Some(act) = accounts.iter_mut().find(|a| a.id == id) {
        act.display_name = new_name;
        save_accounts(&app_handle, &accounts)?;
    }
    Ok(())
}

#[tauri::command]
fn launch_game(exe_path: String) -> Result<(), String> {
    if !Path::new(&exe_path).exists() {
        return Err("找不到游戏程序！请检查路径是否正确。".to_string());
    }
    
    // 使用 cmd start 触发 UAC 提权以解决 (os error 740) 报错
    Command::new("cmd")
        .args(["/C", "start", "", &exe_path])
        .spawn()
        .map_err(|e| format!("无法启动游戏: {}", e))?;
        
    Ok(())
}

#[tauri::command]
fn get_current_fingerprint() -> Result<String, String> {
    let game_data_path = get_game_data_path()?;
    compute_file_hash(&game_data_path.join("login_cache"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_accounts,
            backup_account,
            switch_account,
            delete_account,
            rename_account,
            launch_game,
            get_current_fingerprint
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
