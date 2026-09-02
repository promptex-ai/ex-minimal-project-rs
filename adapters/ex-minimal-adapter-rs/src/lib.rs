//! 第三方 adapter：以 SDK 層實作，只用 AdapterContext 的公開介面。emit 的
//! 責任鏈固定四步：① 算落點表 → ② 回報不支援的 kind → ③ 渲染並產出節點
//! （平台缺某機制時記 degrade 降級）→ ④ 產出被引用的資源。發布前把落點
//! 規則與能力集合換成你的平台實況。
//!
//! adapter 綁語言：本 crate 服務 Rust 專案；TypeScript／Python 專案要用同一
//! 個平台時，各自以該語言的 SDK 實作一份。

use std::collections::BTreeMap;

use promptex_rs::config::Target;
use promptex_rs::refs::kind_name;
use promptex_rs::{define_target, AdapterContext, DegradeItem, EmitFiles, Kind, PluginEntry};

/// 平台能力：本範例只有提示詞單檔與參考資料，無代理、無事件機制。
const SUPPORTED: [Kind; 3] = [Kind::Skill, Kind::Rule, Kind::Instruction];
const UNSUPPORTED: [Kind; 4] = [Kind::Agent, Kind::Hook, Kind::Mcp, Kind::Permission];

pub fn target(root: Option<String>) -> Target {
    let root = root.unwrap_or_else(|| ".ex-minimal-adapter-rs".to_string());

    // 落點規則：提示詞一律單檔平鋪，資源集中於 refs/。
    let node_path = {
        let root = root.clone();
        move |entry: &PluginEntry| format!("{root}/prompts/{}.md", entry.id)
    };
    let resource_path = move |entry: &PluginEntry| format!("{root}/refs/{}.md", entry.id);

    let emit = move |ctx: &AdapterContext| -> EmitFiles {
        let mut files = EmitFiles::new();

        // ① 先算落點表：渲染需要它解析引用，故必須在渲染之前完成。
        let mut layout: BTreeMap<String, String> = BTreeMap::new();
        for kind in SUPPORTED {
            for e in ctx.entries(Some(kind)) {
                layout.insert(format!("{}:{}", kind_name(kind), e.id), node_path(&e));
            }
        }
        for kind in [Kind::Resource, Kind::Asset, Kind::Dir] {
            for e in ctx.entries(Some(kind)) {
                layout.insert(format!("{}:{}", kind_name(kind), e.id), resource_path(&e));
            }
        }

        // ② 不支援的 kind 逐一列報告，不靜默丟棄。
        for kind in UNSUPPORTED {
            for e in ctx.entries(Some(kind)) {
                let name = kind_name(kind);
                ctx.unsupported(DegradeItem::new(kind, &e.id, name, format!("ex-minimal-adapter-rs 無 {name} 對應機制，該節點未產出")));
            }
        }

        // ③ 產出提示詞單檔；rule 的載入範圍在本平台無對應機制，降級為內文標註。
        for kind in SUPPORTED {
            for e in ctx.entries(Some(kind)) {
                let path = node_path(&e);
                let applies_to: Vec<String> = e
                    .config
                    .as_ref()
                    .and_then(|c| c.get("appliesTo"))
                    .and_then(|v| v.as_array())
                    .map(|items| items.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                let note = if kind == Kind::Rule && !applies_to.is_empty() {
                    ctx.degrade(DegradeItem::new(
                        kind,
                        &e.id,
                        "scopedLoading",
                        "ex-minimal-adapter-rs 無範圍載入機制，改為常駐並於內文標註適用範圍",
                    ));
                    format!("適用範圍：{}\n\n", applies_to.join("、"))
                } else {
                    String::new()
                };
                let body =
                    format!("{}# {}\n\n{note}{}\n", frontmatter(ctx, &e), e.name, ctx.render(&e, &path, &layout));
                files.insert(path, body.into_bytes());
            }
        }

        // ④ 被引用的資源。
        for e in ctx.entries(Some(Kind::Resource)) {
            let path = resource_path(&e);
            let body = format!("# {}\n\n{}\n", e.name, ctx.render(&e, &path, &layout));
            files.insert(path, body.into_bytes());
        }

        files
    };

    // 參數宣告（標準 JSON Schema）：include_str! 讀 crate 根的同一份檔案、隨
    // 中介表示交給讀取端，`promptex config declare` 也讀它。
    define_target("ex-minimal-adapter-rs", emit).config_schema(include_str!("../promptex.config.schema.json"))
}

/// 平台原生鍵的覆寫：一律經 `ctx.overrides` 讀取，不自己走
/// `entry.config["platforms"]`：「空物件視同未宣告」的判定與框架保留鍵
/// （`promptex:` 前綴）的過濾都由核心同一份實作承載，自己讀會各自重造而漂移。
fn frontmatter(ctx: &AdapterContext, entry: &PluginEntry) -> String {
    let Some(overrides) = ctx.overrides(entry) else { return String::new() };
    let body: Vec<String> = overrides
        .iter()
        .map(|(k, v)| format!("{k}: {}", v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())))
        .collect();
    format!("---\n{}\n---\n\n", body.join("\n"))
}
