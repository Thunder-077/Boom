use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mupdf::{Document, TextPageFlags};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfNativeTextFragment {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_size: f64,
    pub editability: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfNativeTextLine {
    pub text: String,
    pub fragments: Vec<PdfNativeTextFragment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfNativeTextPage {
    pub page_number: usize,
    pub width: f64,
    pub height: f64,
    pub lines: Vec<PdfNativeTextLine>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfNativeEditAnalysis {
    pub engine: String,
    pub pages: Vec<PdfNativeTextPage>,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn analyze_pdf_native_editability(bytes: Vec<u8>) -> Result<PdfNativeEditAnalysis, String> {
    let path = write_temp_pdf(&bytes)?;
    let result = analyze_pdf_path(&path);
    let _ = fs::remove_file(&path);
    result
}

fn write_temp_pdf(bytes: &[u8]) -> Result<PathBuf, String> {
    let mut path = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("生成临时文件名失败: {e}"))?
        .as_millis();
    path.push(format!("boom-mupdf-analysis-{stamp}.pdf"));
    fs::write(&path, bytes).map_err(|e| format!("写入 MuPDF 临时文件失败: {e}"))?;
    Ok(path)
}

fn analyze_pdf_path(path: &PathBuf) -> Result<PdfNativeEditAnalysis, String> {
    let path_text = path
        .to_str()
        .ok_or_else(|| "MuPDF 只能打开 UTF-8 路径的临时文件。".to_string())?;
    let document = Document::open(path_text).map_err(|e| format!("MuPDF 打开 PDF 失败: {e}"))?;
    let mut pages = Vec::new();
    let mut warnings = vec![
        "当前阶段仅做 MuPDF 文本结构分析；真正内容流改写将在此基础上分流实现。".to_string(),
        "复杂字体子集、竖排文字、扫描件和旋转文本会降级到覆盖式替换。".to_string(),
    ];

    for (page_index, page_result) in document
        .pages()
        .map_err(|e| format!("MuPDF 读取页面失败: {e}"))?
        .enumerate()
    {
        let page = page_result.map_err(|e| format!("MuPDF 载入第 {} 页失败: {e}", page_index + 1))?;
        let bounds = page
            .bounds()
            .map_err(|e| format!("MuPDF 读取第 {} 页尺寸失败: {e}", page_index + 1))?;
        let page_height = f64::from(bounds.height());
        let page_width = f64::from(bounds.width());
        let text_page = page
            .to_text_page(TextPageFlags::empty())
            .map_err(|e| format!("MuPDF 提取第 {} 页文本失败: {e}", page_index + 1))?;

        let mut lines = Vec::new();
        for block in text_page.blocks() {
            for line in block.lines() {
                let mut fragments = Vec::new();
                let mut line_text = String::new();
                let mut current = NativeFragmentBuilder::default();
                for character in line.chars() {
                    let Some(ch) = character.char() else {
                        continue;
                    };
                    if ch.is_whitespace() {
                        if let Some(fragment) = current.finish(page_height) {
                            if !line_text.is_empty() {
                                line_text.push(' ');
                            }
                            line_text.push_str(&fragment.text);
                            fragments.push(fragment);
                        }
                        continue;
                    }
                    current.push(ch, &character, page_height);
                }
                if let Some(fragment) = current.finish(page_height) {
                    if !line_text.is_empty() {
                        line_text.push(' ');
                    }
                    line_text.push_str(&fragment.text);
                    fragments.push(fragment);
                }
                if !fragments.is_empty() {
                    lines.push(PdfNativeTextLine {
                        text: line_text,
                        fragments,
                    });
                }
            }
        }
        if lines.is_empty() {
            warnings.push(format!("第 {} 页未提取到文本，可能是扫描件或图片页。", page_index + 1));
        }
        pages.push(PdfNativeTextPage {
            page_number: page_index + 1,
            width: page_width,
            height: page_height,
            lines,
        });
    }

    Ok(PdfNativeEditAnalysis {
        engine: "MuPDF".to_string(),
        pages,
        warnings,
    })
}

#[derive(Default)]
struct NativeFragmentBuilder {
    text: String,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    font_size: f64,
    initialized: bool,
}

impl NativeFragmentBuilder {
    fn push(&mut self, ch: char, character: &mupdf::text_page::TextChar, page_height: f64) {
        let quad = character.quad();
        let x0 = f64::from(quad.ll.x.min(quad.ul.x));
        let x1 = f64::from(quad.lr.x.max(quad.ur.x));
        let bottom = f64::from(quad.ll.y.min(quad.lr.y));
        let top = f64::from(quad.ul.y.max(quad.ur.y));
        let y0 = page_height - top;
        let y1 = page_height - bottom;
        if !self.initialized {
            self.x0 = x0;
            self.y0 = y0;
            self.x1 = x1;
            self.y1 = y1;
            self.font_size = f64::from(character.size());
            self.initialized = true;
        } else {
            self.x0 = self.x0.min(x0);
            self.y0 = self.y0.min(y0);
            self.x1 = self.x1.max(x1);
            self.y1 = self.y1.max(y1);
            self.font_size = self.font_size.max(f64::from(character.size()));
        }
        self.text.push(ch);
    }

    fn finish(&mut self, _page_height: f64) -> Option<PdfNativeTextFragment> {
        if !self.initialized || self.text.trim().is_empty() {
            *self = Self::default();
            return None;
        }
        let text = self.text.trim().to_string();
        let width = (self.x1 - self.x0).max(1.0);
        let height = (self.y1 - self.y0).max(1.0);
        let editability = classify_editability(&text, width, self.font_size);
        let reason = match editability.as_str() {
            "native-candidate" => "单行文本片段，长度和字符形态适合尝试内容流替换。",
            "overlay-recommended" => "文本较长或包含复杂字符，建议继续使用覆盖式替换。",
            _ => "无法判断。",
        }
        .to_string();
        let fragment = PdfNativeTextFragment {
            text,
            x: self.x0,
            y: self.y0,
            width,
            height,
            font_size: self.font_size,
            editability,
            reason,
        };
        *self = Self::default();
        Some(fragment)
    }
}

fn classify_editability(text: &str, width: f64, font_size: f64) -> String {
    let char_count = text.chars().count();
    let has_control = text.chars().any(char::is_control);
    let average_width = if char_count == 0 {
        0.0
    } else {
        width / char_count as f64
    };
    if !has_control && char_count <= 24 && average_width > font_size * 0.15 {
        "native-candidate".to_string()
    } else {
        "overlay-recommended".to_string()
    }
}
