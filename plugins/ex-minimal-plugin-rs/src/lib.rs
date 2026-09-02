//! 最小可發布 plugin：三個生命週期各示範一件事，發布前把內容換成你的邏輯。
//!
//! 生命週期固定順序：prepare（收集遍之前；Rust 側為同步，理由見
//! `promptex_rs::plugin` 模組文件）→ 源碼求值 → transform（改寫遍）→
//! validate（解析遍，核心檢查在先）。plugin 產出的是「節點」而非檔案：要
//! 落地的內容以 `define_*` 新增資源節點，由核心落地為產物。

use std::cell::RefCell;
use std::rc::Rc;

use promptex_rs::{Diagnostic, Kind, Plugin, Severity};

const DEFAULT_BANNER: &str = "（本節點由 ex-minimal-plugin-rs plugin 追加此行）";

pub fn create_plugin(banner: Option<String>) -> Plugin {
    // prepare 取得的資料存進共享狀態，transform 只讀它、不知道資料從何而來
    // （另兩個生態用語言原生閉包，Rust 以 Rc<RefCell> 表達同一件事）。
    let resolved: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let for_prepare = Rc::clone(&resolved);
    let for_transform = Rc::clone(&resolved);

    Plugin::new("ex-minimal-plugin-rs")
        // 宣告本擴充作用在哪幾種節點類型（供文件與讀取端），不隱含過濾。
        .kinds(vec![Kind::Skill, Kind::Rule])
        // 相容的框架版本範圍：安裝的 promptex-rs 落在範圍外時於編譯開始前報錯。
        .version("^0.0.0")
        // 參數宣告（標準 JSON Schema）：include_str! 讀 crate 根的同一份檔案，
        // `promptex config declare` 也讀它。
        .config_schema(include_str!("../promptex.config.schema.json"))
        // 準備：於收集遍之前執行。真實 plugin 在這裡取外部資源，或以
        // ctx.cache_dir 快取、ctx.write_lock 記錄鎖定；本範例只把參數收斂成
        // transform 要用的值。失敗回 Err(訊息)。
        .prepare(move |_ctx| {
            *for_prepare.borrow_mut() = banner.clone().unwrap_or_else(|| DEFAULT_BANNER.to_string());
            Ok(())
        })
        // 改寫：改寫既有節點一律經 ctx 操作函式（append_content／patch_config），
        // 變更可追蹤、衝突可偵測；也可在此以 define_* 新增節點。
        .transform(move |ctx| {
            let banner = for_transform.borrow().clone();
            for kind in [Kind::Skill, Kind::Rule] {
                for entry in ctx.entries_of(kind) {
                    ctx.append_content(&entry, banner.clone().into());
                }
            }
        })
        // 驗證：對註冊表做結構驗證，回傳診斷（空清單即通過）。
        .validate(|ctx| {
            ctx.entries()
                .iter()
                .filter(|e| e.id.starts_with("promptex-"))
                .map(|e| Diagnostic {
                    code: "ex-minimal-plugin-rs-reserved-id".into(),
                    message: format!("節點 {} 以保留前綴 promptex- 命名，請改用其他 id", e.id),
                    at: vec![e.at.clone()],
                    hint: None,
                    severity: Severity::default(),
                })
                .collect()
        })
}
