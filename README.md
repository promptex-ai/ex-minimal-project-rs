# ex-minimal-project-rs

promptex 的最小 Rust 消費端專案，同時是 [promptex-resources-rs](https://github.com/promptex-ai/promptex-resources-rs) 的 `example/` 成員（以 submodule 掛入）。骨架由 `promptex init` 產出，之上接了兩份擴展，讓「一份源碼投影到多個平台」與「擴展如何介入」都成為可讀的既成事實。

## 結構

| 內容 | 作用 |
| :--- | :--- |
| `Cargo.toml` | 套件宣告檔，宣告對 SDK 的依賴與兩份擴展的路徑來源 |
| `promptex.config.rs` | 配置檔，宣告單元名、源碼目錄、產物落點、目標平台與 plugin |
| `prompts/example.rs` | 提示詞源碼。一份最小的規則宣告，帶適用範圍 |
| `plugins/ex-minimal-plugin-rs/` | plugin 擴展，在改寫遍為每個 skill 與 rule 追加一行 |
| `adapters/ex-minimal-adapter-rs/` | 第三方平台適配擴展，把同一份源碼投影成另一套平台原生產物 |
| `src/bin/promptex.rs` | 本專案自己的 promptex 執行檔殼。Rust 的求值在編譯期綁定，沒有能動態載入任意路徑源碼的通用執行檔，每個專案因此編譯成自己的一份。bin 名恰好是 `promptex`，那是查詢進入點慣例的另一半：鄰居專案以 `cargo run --quiet --bin promptex` 問得到本專案的結構 |

兩份擴展是本專案的一部分而非獨立套件：它們不自成工作區、不帶各自的消費專案、名稱也不加 registry 搜尋用的 `promptex-plugin-`／`promptex-adapter-` 前綴。`Cargo.toml` 以路徑依賴掛上兩份擴展，它們隨本專案的 `promptex` 執行檔一起編譯：Rust 的求值在編譯期綁定，擴展是同一次編譯的一部分，不是執行期載入的模組。

## 建置與產物

```bash
cargo fetch
cargo run -- install .
```

產物落在專案根（配置單元的 `out_dir` 是 `.`），並隨源碼一起入版控——讀者不必先跑指令就看得到源碼與產物的對應：

- `.claude/rules/example-rule.md`——內建 claude 適配的產物，frontmatter 帶 `paths` 載入宣告
- `.ex-minimal-adapter-rs/prompts/example-rule.md`——第三方適配的產物
- `.promptex/ex-minimal-project-rs/claude.json` 與 `.promptex/ex-minimal-project-rs/ex-minimal-adapter-rs.json`——兩個平台各自的所有權登記，下一次安裝據它 prune

`cargo run -- build .` 只刷新框架中繼，不落平台產物。

## 讀產物時看什麼

- 兩份產物出自同一份源碼，差別全在適配：落點、檔案格式與載入宣告的表達方式由平台決定，源碼裡沒有任何平台相關程式碼
- 兩份產物的內文末尾都多出一行「本節點由 ex-minimal-plugin-rs plugin 追加此行」，那是 plugin 在改寫遍介入的可見證據
- 範例規則宣告了適用範圍，屬載入宣告三態中的範圍載入態：claude 產物把它表達成 frontmatter 的 `paths`；第三方適配無範圍載入機制，安裝報告因此記一項降級，改為常駐並在內文標註適用範圍
- 中繼與登記落在 `.promptex/ex-minimal-project-rs/` 而非推導出的 `unit-0`，因為配置宣告了單元名；未宣告時名字綁在陣列位置上，日後在前面插入第二個單元即等同把第一個單元改名

## 與 init 骨架的差異

骨架的產物原樣保留，只做以下調整：

| 項目 | 骨架 | 本專案 | 差異理由 |
| :--- | :--- | :--- | :--- |
| 套件名與配置單元名 | `promptex-prompts` | `ex-minimal-project-rs` | 本專案要當 promptex-resources-rs 的工作區成員，成員名必須唯一，且本倉庫的慣例是成員名等於目錄名 |
| 空的 `[workspace]` 表 | 有 | 移除 | 空表的作用是切斷外層工作區歸屬，讓目標目錄開在哪裡都編得動；本專案的處境相反，它要被外層倉庫的成員 glob 收進去。移除後獨立 clone（本檔自成工作區根）與掛為成員兩種形態都成立 |
| 目標平台與 plugin | 只有 claude | 加上第三方適配與 plugin | 骨架示範的是最小可建置形態；本專案要示範的是擴展怎麼介入，兩份擴展因此接進配置 |
