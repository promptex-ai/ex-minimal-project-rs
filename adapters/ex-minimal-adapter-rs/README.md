# ex-minimal-adapter-rs

[ex-minimal-project-rs](../../) 的 adapter 擴展。由專案以路徑依賴掛上，隨專案自己的 promptex 執行檔一起編譯：Rust 的求值在編譯期綁定，擴展因此是同一次編譯的一部分，不是執行期載入的模組；同時保持可發布形態——中繼欄位、參數宣告與授權都隨套件出貨，版號比照 promptex 的 alpha 原型套件。

適配以 SDK 層實作，只用 `AdapterContext` 的公開介面。`emit` 的責任鏈固定四步：

1. 算落點表——渲染要靠它解析引用，因此必須在渲染之前完成。
2. 回報不支援的 kind——逐一列進報告，不靜默丟棄。
3. 渲染並產出節點——平台缺某機制時記 degrade 降級。
4. 產出被引用的資源。

本適配示範的平台只有提示詞單檔與參考資料：支援 skill、rule、instruction，不支援 agent、hook、mcp、permission。適配綁語言——同一個平台要服務另兩個生態的專案時，各自以該語言的 SDK 實作一份。

## 在本專案的接法

`promptex.config.rs` 的目標清單寫成 `vec![claude(), ex_minimal_adapter_rs::target(None)]`，claude 是內建適配、本擴展是第三方適配，同一份源碼因此投影出兩套平台原生產物。跑 `cargo run -- install .` 後產物落在 `.ex-minimal-adapter-rs/prompts/`，安裝報告會列出一項降級：範例規則宣告了適用範圍，而本平台無範圍載入機制，改為常駐並在內文標註適用範圍。

## 參數宣告

參數宣告（標準 JSON Schema）住crate 根的 `promptex.config.schema.json`，由 `src/lib.rs` 以 `include_str!` 內嵌，隨中介表示交給讀取端；`promptex config declare ex-minimal-adapter-rs` 讀的是同一份檔案。

## 發布

```bash
cargo package --list
cargo publish --dry-run
cargo publish
```

`cargo package`／`cargo publish` 刻意不吃外層工作區的 `[patch.crates-io]`，乾跑用的是 registry 上真實存在的 SDK 版本——這正是它能提前抓出版本問題的原因。

發布前先把 SDK 依賴換掉：`[dependencies]` 的 `promptex-rs = "0.0.0"` 指的是開發中的本地 SDK，registry 上目前只有 alpha 原型（只有 `defineSkill` 與 `build` 那一條最窄路徑），不含本擴展用到的 plugin 與適配介面。照現況發布出去，安裝端裝得下來卻編不動。

名稱刻意不帶 `promptex-adapter-` 前綴——那是給要被消費端搜尋到的套件用的；本擴展的定位是示範，改以 keywords 的 `promptex-adapter` 承載可搜尋性。
