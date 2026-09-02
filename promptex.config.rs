#[path = "prompts/example.rs"]
mod example;

use std::path::Path;

use promptex_rs::config::{claude, ConfigUnit};

pub fn units(_project_root: &Path) -> Vec<ConfigUnit> {
    // 兩個平台目標：claude 是內建適配，ex_minimal_adapter_rs 是本專案自帶的
    // 第三方適配，同一份源碼因此投影出兩套平台原生產物。plugin 在求值後的改寫遍
    // 追加內容，兩個平台的產物都看得到它的痕跡。
    vec![ConfigUnit::new(
        "./prompts",
        vec![claude(), ex_minimal_adapter_rs::target(None)],
    )
    .name("ex-minimal-project-rs")
    .out_dir(".")
    .plugins(vec![ex_minimal_plugin_rs::create_plugin(None)])
    .evaluate(example::declare)]
}
