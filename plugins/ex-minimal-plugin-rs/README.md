# ex-minimal-plugin-rs

[ex-minimal-project-rs](../../) 的 plugin 擴展，不是獨立套件。由專案以路徑依賴掛上，隨專案自己的 promptex 執行檔一起編譯：Rust 的求值在編譯期綁定，擴展因此是同一次編譯的一部分，不是執行期載入的模組。

plugin 掛在求值管線的三個生命週期上，順序固定：

1. `prepare`——唯一 async 的階段，跑在收集遍之前。真實 plugin 在這裡請求外部資源，或以 `ctx.cacheDir` 快取、`ctx.writeLock` 記錄鎖定，再經閉包把資料交給後續階段。
2. `transform`——改寫遍。改寫既有節點一律經 ctx 操作函式（`appendContent`／`patchConfig`），型別安全、變更可追蹤、衝突可偵測；也可在此以 `define*` 新增節點。
3. `validate`——解析遍，核心檢查在先。對註冊表做結構驗證並回傳診斷，空清單即通過。

plugin 產出的是節點而非檔案：要落地的內容以 `define*` 新增資源節點，由核心落地為產物。

## 在本專案的接法

`promptex.config.rs` 以 `.plugins(vec![ex_minimal_plugin_rs::create_plugin(None)])` 掛上。跑 `cargo run -- install .` 後，兩個平台的 `example-rule` 產物末尾都會多出本 plugin 在 `transform` 追加的那一行——那是它有生效的可見證據。

## 參數宣告

參數宣告（標準 JSON Schema）住crate 根的 `promptex.config.schema.json`，由 `src/lib.rs` 自己讀進來，隨中介表示交給讀取端；`promptex config declare ex-minimal-plugin-rs` 讀的是同一份檔案。
