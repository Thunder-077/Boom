use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn get_pdf_editor_font() -> Result<Vec<u8>, String> {
    let candidates = [
        r"C:\Windows\Fonts\Deng.ttf",
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
    ];
    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return fs::read(&path).map_err(|e| format!("读取 PDF 中文字体失败: {e}"));
        }
    }
    Err("未找到可用于 PDF 导出的中文字体".to_string())
}

#[tauri::command]
pub fn save_pdf_file(path: String, bytes: Vec<u8>) -> Result<(), String> {
    let target = PathBuf::from(path.trim());
    if target.as_os_str().is_empty() {
        return Err("保存路径不能为空".to_string());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建保存目录失败: {e}"))?;
    }
    fs::write(&target, bytes).map_err(|e| format!("写入 PDF 文件失败: {e}"))?;
    Ok(())
}
