mod barcode;
mod eps;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct BarcodeRequest {
    isbn: String,
    addon: String,
    bar_height_mm: f64,
    dpi: u32,
    addon_offset_mm: f64,
}

#[derive(Serialize)]
pub struct BarcodeResult {
    success: bool,
    message: String,
    eps_content: Option<String>,
    file_path: Option<String>,
}

#[tauri::command]
fn generate_barcode(request: BarcodeRequest) -> BarcodeResult {
    // Validate ISBN
    if request.isbn.len() != 13 || !request.isbn.chars().all(|c| c.is_ascii_digit()) {
        return BarcodeResult {
            success: false,
            message: "ISBN은 13자리 숫자여야 합니다.".to_string(),
            eps_content: None,
            file_path: None,
        };
    }

    if !barcode::validate_isbn13(&request.isbn) {
        return BarcodeResult {
            success: false,
            message: "ISBN 체크디짓이 올바르지 않습니다.".to_string(),
            eps_content: None,
            file_path: None,
        };
    }

    // Validate add-on
    if !request.addon.is_empty()
        && (request.addon.len() != 5 || !request.addon.chars().all(|c| c.is_ascii_digit()))
    {
        return BarcodeResult {
            success: false,
            message: "분류번호는 5자리 숫자여야 합니다.".to_string(),
            eps_content: None,
            file_path: None,
        };
    }

    match eps::generate_eps(&request.isbn, &request.addon, request.bar_height_mm, request.dpi, request.addon_offset_mm) {
        Some(content) => BarcodeResult {
            success: true,
            message: "바코드가 생성되었습니다.".to_string(),
            eps_content: Some(content),
            file_path: None,
        },
        None => BarcodeResult {
            success: false,
            message: "바코드 생성에 실패했습니다.".to_string(),
            eps_content: None,
            file_path: None,
        },
    }
}

#[tauri::command]
fn save_eps(content: String, file_path: String) -> BarcodeResult {
    let path = PathBuf::from(&file_path);
    match fs::write(&path, &content) {
        Ok(_) => BarcodeResult {
            success: true,
            message: format!("저장 완료: {}", file_path),
            eps_content: None,
            file_path: Some(file_path),
        },
        Err(e) => BarcodeResult {
            success: false,
            message: format!("저장 실패: {}", e),
            eps_content: None,
            file_path: None,
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![generate_barcode, save_eps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
