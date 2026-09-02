// 本專案自己的 promptex 執行檔：Rust 的求值在編譯期綁定，因此每個專案編譯
// 成自己的執行檔，而非由通用工具讀取源碼求值。
//
// 執行安裝動詞：cargo run -- install .
// （cargo run -- build . 是編譯動詞，只刷新框架中繼、不落平台產物）
//
// 檔案放在 src/bin/ 底下，bin 名因此是 promptex（Cargo 的自動發現：
// src/bin/<名字>.rs 即 bin <名字>，不必在 Cargo.toml 寫 [[bin]]）。這個名字是
// 查詢進入點慣例的另一半：別的專案要問本專案的結構時，跑的是
// cargo run --quiet --bin promptex -- manifest <本專案根> --platform <平台>。
// 改名或搬到 src/main.rs 之後本專案照樣建得動，但別人問不到它。

// 路徑相對於本檔所在目錄（src/bin/），因此要退兩層才回到專案根。
#[path = "../../promptex.config.rs"]
mod config;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(promptex_rs::cli::run(&argv, config::units));
}
