use lang_core::Language;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{Attribute, Fields, Item, ItemEnum, Type, parse_file};

// Parse target: Extract Commands and its sub-enum structures from lib.rs, dynamically generate UI
const LIB_RS_SRC: &str = include_str!("../../cli/src/lib.rs");

#[derive(Debug, Clone)]
pub enum UiFieldType {
    PathBuf,
    String,
    Usize,
    F64,
    Bool,
    Enum(Vec<String>), // Enum variant list
    Other,
}

#[derive(Debug, Clone)]
pub struct UiFieldSpec {
    pub name: String,
    pub ty: UiFieldType,
    pub has_long: bool,
    pub long_name: Option<String>,
    pub default: Option<String>,
    pub value_name: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiVariantSpec {
    pub name: String,
    pub fields: Vec<UiFieldSpec>,
    pub doc: Option<String>,
    pub zh_name: Option<String>,
    pub zh_desc: Option<String>,
    pub en_name: Option<String>,
    pub en_desc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiSubEnumSpec {
    pub variants: Vec<UiVariantSpec>,
}

#[derive(Debug, Clone)]
pub struct UiTopSpec {
    pub variant_ident: String,
    pub sub_enum_ident: String,
    pub doc: Option<String>,
    pub zh_name: Option<String>,
    pub zh_desc: Option<String>,
    pub en_name: Option<String>,
    pub en_desc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiTree {
    pub top_commands: Vec<UiTopSpec>,
    pub sub_enums: HashMap<String, UiSubEnumSpec>,
}

fn ident_to_string_path(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => format!("{}", ty.to_token_stream()),
    }
}

fn type_to_ui_type(ty: &Type) -> UiFieldType {
    let ts = ident_to_string_path(ty);
    match ts.as_str() {
        "PathBuf" | "std::path::PathBuf" => UiFieldType::PathBuf,
        "String" | "std::string::String" => UiFieldType::String,
        "usize" => UiFieldType::Usize,
        "f64" => UiFieldType::F64,
        "bool" => UiFieldType::Bool,
        "BmsFolderSetNameType" => UiFieldType::Enum(vec![
            "replace_title_artist".to_string(),
            "append_title_artist".to_string(),
            "append_artist".to_string(),
        ]),
        "RemoveMediaPreset" => UiFieldType::Enum(vec![
            "oraja".to_string(),
            "wav_fill_flac".to_string(),
            "mpg_fill_wmv".to_string(),
        ]),
        "ReplacePreset" => {
            UiFieldType::Enum(vec!["default".to_string(), "update_pack".to_string()])
        }
        _ => UiFieldType::Other,
    }
}

fn attr_tokens_contains_long(attr: &Attribute) -> bool {
    // Try not to hardcode parsing, loosely judge "long" pattern
    let s = attr.to_token_stream().to_string();
    s.contains(" long") || s.contains("long ") || s.contains("long,") || s.contains("long=")
}

fn extract_arg_value_name(attr: &Attribute) -> Option<String> {
    // Find value_name = "..."
    let s = attr.to_token_stream().to_string();
    let key = "value_name = \"";
    if let Some(pos) = s.find(key) {
        let rest = &s[pos + key.len()..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn extract_default_value(attr: &Attribute) -> Option<String> {
    // Find default_value = "..."
    let s = attr.to_token_stream().to_string();
    let key = "default_value = \"";
    if let Some(pos) = s.find(key) {
        let rest = &s[pos + key.len()..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

// Extract accumulated doc comment strings from attributes (converted from /// to #[doc = "..."])
fn extract_doc_string(attrs: &[Attribute]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for a in attrs {
        if a.path().is_ident("doc") {
            let s = a.to_token_stream().to_string();
            if let Some(pos) = s.find('"') {
                let rest = &s[pos + 1..];
                if let Some(end) = rest.rfind('"') {
                    let mut line = rest[..end].to_string();
                    // Remove one space from the beginning of the line (common leading space in rustdoc)
                    if line.starts_with(' ') {
                        line.remove(0);
                    }
                    lines.push(line);
                }
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn extract_lang(
    attrs: &[Attribute],
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut zh_name = None;
    let mut zh_desc = None;
    let mut en_name = None;
    let mut en_desc = None;
    for a in attrs {
        if a.path().is_ident("lang_chinese") || a.path().is_ident("lang_english") {
            let s = a.to_token_stream().to_string();
            let name_key = "name = \"";
            let desc_key = "desc = \"";
            let nm = s.find(name_key).and_then(|p| {
                let rest = &s[p + name_key.len()..];
                rest.find('"').map(|e| rest[..e].to_string())
            });
            let ds = s.find(desc_key).and_then(|p| {
                let rest = &s[p + desc_key.len()..];
                rest.find('"').map(|e| rest[..e].to_string())
            });
            if a.path().is_ident("lang_chinese") {
                zh_name = nm;
                zh_desc = ds;
            } else {
                en_name = nm;
                en_desc = ds;
            }
        }
    }
    (zh_name, zh_desc, en_name, en_desc)
}

pub fn to_kebab_case(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for ch in name.chars() {
        if ch == '_' {
            out.push('-');
            prev_lower = false;
            continue;
        }
        if ch.is_ascii_uppercase() {
            if prev_lower {
                out.push('-');
            }
            for c in ch.to_lowercase() {
                out.push(c);
            }
            prev_lower = false;
        } else {
            out.push(ch);
            prev_lower = ch.is_ascii_lowercase();
        }
    }
    out
}

pub fn to_long_flag(name: &str) -> String {
    name.replace('_', "-")
}

pub fn build_ui_tree() -> UiTree {
    let file = parse_file(LIB_RS_SRC).expect("parse lib.rs failed");
    let mut enums: HashMap<String, ItemEnum> = HashMap::new();
    for item in file.items {
        if let Item::Enum(e) = item {
            enums.insert(e.ident.to_string(), e);
        }
    }

    let commands = enums
        .get("Commands")
        .expect("Commands enum not found in lib.rs");

    let mut top_commands: Vec<UiTopSpec> = Vec::new();
    for var in &commands.variants {
        // Variant like: Work { command: WorkCommands }
        let mut sub_enum_ident = None;
        if let Fields::Named(named) = &var.fields {
            for f in &named.named {
                if let Some(ident) = &f.ident
                    && ident == "command"
                {
                    sub_enum_ident = Some(ident_to_string_path(&f.ty));
                    break;
                }
            }
        }
        if let Some(sub) = sub_enum_ident {
            let (zh_name, zh_desc, en_name, en_desc) = extract_lang(&var.attrs);
            top_commands.push(UiTopSpec {
                variant_ident: var.ident.to_string(),
                sub_enum_ident: sub,
                doc: extract_doc_string(&var.attrs),
                zh_name,
                zh_desc,
                en_name,
                en_desc,
            });
        }
    }

    let mut sub_enums: HashMap<String, UiSubEnumSpec> = HashMap::new();
    for top in &top_commands {
        if let Some(sub_enum) = enums.get(&top.sub_enum_ident) {
            let mut variants: Vec<UiVariantSpec> = Vec::new();
            for v in &sub_enum.variants {
                // Skip CLI-only interactive commands
                if v.ident == "SetFileNum" {
                    continue;
                }

                let mut fields_spec: Vec<UiFieldSpec> = Vec::new();
                match &v.fields {
                    Fields::Named(named) => {
                        for f in &named.named {
                            let name = f
                                .ident
                                .as_ref()
                                .map(|i| i.to_string())
                                .unwrap_or_else(|| "arg".to_string());

                            let ty = type_to_ui_type(&f.ty);
                            let field_doc = extract_doc_string(&f.attrs);
                            let mut has_long = false;
                            let mut value_name = None;
                            let mut default = None;
                            for attr in &f.attrs {
                                if attr.path().is_ident("arg") {
                                    if attr_tokens_contains_long(attr) {
                                        has_long = true;
                                    }
                                    if value_name.is_none() {
                                        value_name = extract_arg_value_name(attr);
                                    }
                                    if default.is_none() {
                                        default = extract_default_value(attr);
                                    }
                                }
                            }
                            let long_name = if has_long {
                                Some(to_long_flag(&name))
                            } else {
                                None
                            };
                            fields_spec.push(UiFieldSpec {
                                name,
                                ty,
                                has_long,
                                long_name,
                                default,
                                value_name,
                                doc: field_doc,
                            });
                        }
                    }
                    Fields::Unnamed(_) | Fields::Unit => {}
                }
                let (zh_name, zh_desc, en_name, en_desc) = extract_lang(&v.attrs);
                variants.push(UiVariantSpec {
                    name: v.ident.to_string(),
                    fields: fields_spec,
                    doc: extract_doc_string(&v.attrs),
                    zh_name,
                    zh_desc,
                    en_name,
                    en_desc,
                });
            }
            sub_enums.insert(top.sub_enum_ident.clone(), UiSubEnumSpec { variants });
        }
    }

    UiTree {
        top_commands,
        sub_enums,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ui_tree() {
        let tree = build_ui_tree();

        // 测试基本结构
        assert!(
            !tree.top_commands.is_empty(),
            "top_commands should not be empty"
        );
        assert!(!tree.sub_enums.is_empty(), "sub_enums should not be empty");

        // 测试Root命令存在
        let root_cmd = tree.top_commands.iter().find(|t| t.variant_ident == "Root");
        assert!(root_cmd.is_some(), "Root command should exist");

        if let Some(root_cmd) = root_cmd {
            // 测试Root命令有文档注释
            assert!(
                root_cmd.doc.is_some(),
                "Root command should have documentation"
            );

            // 测试Root命令的子枚举存在
            let sub_enum = tree.sub_enums.get(&root_cmd.sub_enum_ident);
            assert!(sub_enum.is_some(), "Root sub enum should exist");

            if let Some(sub_enum) = sub_enum {
                // 测试子枚举有变体
                assert!(
                    !sub_enum.variants.is_empty(),
                    "Root sub enum should have variants"
                );

                // 测试SetName变体存在且有文档注释
                let set_name_variant = sub_enum.variants.iter().find(|v| v.name == "SetName");
                assert!(set_name_variant.is_some(), "SetName variant should exist");

                if let Some(set_name_variant) = set_name_variant {
                    assert!(
                        set_name_variant.doc.is_some(),
                        "SetName variant should have documentation"
                    );

                    // 测试set_type字段存在且为枚举类型
                    let set_type_field = set_name_variant
                        .fields
                        .iter()
                        .find(|f| f.name == "set_type");
                    assert!(set_type_field.is_some(), "set_type field should exist");

                    if let Some(set_type_field) = set_type_field {
                        match &set_type_field.ty {
                            UiFieldType::Enum(variants) => {
                                assert_eq!(
                                    variants.len(),
                                    3,
                                    "BmsFolderSetNameType should have 3 variants"
                                );
                                assert!(variants.contains(&"replace_title_artist".to_string()));
                                assert!(variants.contains(&"append_title_artist".to_string()));
                                assert!(variants.contains(&"append_artist".to_string()));
                            }
                            other => panic!("set_type should be enum type, got: {:?}", other),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_work_commands() {
        let tree = build_ui_tree();

        // 测试Work命令存在
        let work_cmd = tree.top_commands.iter().find(|t| t.variant_ident == "Work");
        assert!(work_cmd.is_some(), "Work command should exist");

        if let Some(work_cmd) = work_cmd {
            // 测试Work命令有文档注释
            assert!(
                work_cmd.doc.is_some(),
                "Work command should have documentation"
            );

            // 测试Work命令的子枚举存在
            let sub_enum = tree.sub_enums.get(&work_cmd.sub_enum_ident);
            assert!(sub_enum.is_some(), "Work sub enum should exist");

            if let Some(sub_enum) = sub_enum {
                // 测试SetName变体存在且有文档注释
                let set_name_variant = sub_enum.variants.iter().find(|v| v.name == "SetName");
                assert!(set_name_variant.is_some(), "SetName variant should exist");

                if let Some(set_name_variant) = set_name_variant {
                    assert!(
                        set_name_variant.doc.is_some(),
                        "SetName variant should have documentation"
                    );

                    // 测试dir字段存在且为PathBuf类型
                    let dir_field = set_name_variant.fields.iter().find(|f| f.name == "dir");
                    assert!(dir_field.is_some(), "dir field should exist");

                    if let Some(dir_field) = dir_field {
                        match &dir_field.ty {
                            UiFieldType::PathBuf => {
                                // 正确
                            }
                            other => panic!("dir should be PathBuf type, got: {:?}", other),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_enum_documentation_extraction() {
        let tree = build_ui_tree();

        // 测试所有顶级命令都有文档注释
        for cmd in &tree.top_commands {
            assert!(
                cmd.doc.is_some(),
                "Command {} should have documentation",
                cmd.variant_ident
            );
        }

        // 测试所有子枚举变体都有文档注释
        for (enum_name, sub_enum) in &tree.sub_enums {
            for variant in &sub_enum.variants {
                assert!(
                    variant.doc.is_some(),
                    "Variant {} in {} should have documentation",
                    variant.name,
                    enum_name
                );
            }
        }
    }

    #[test]
    fn test_field_documentation_extraction() {
        let tree = build_ui_tree();

        // 测试SetName命令的字段有文档注释
        if let Some(root_cmd) = tree.top_commands.iter().find(|t| t.variant_ident == "Root")
            && let Some(sub_enum) = tree.sub_enums.get(&root_cmd.sub_enum_ident)
            && let Some(set_name_variant) = sub_enum.variants.iter().find(|v| v.name == "SetName")
        {
            for field in &set_name_variant.fields {
                assert!(
                    field.doc.is_some(),
                    "Field {} should have documentation",
                    field.name
                );
            }
        }
    }

    #[test]
    fn test_utility_functions() {
        // 测试to_kebab_case函数
        assert_eq!(to_kebab_case("ReplaceTitleArtist"), "replace-title-artist");
        assert_eq!(to_kebab_case("set_name_by_bms"), "set-name-by-bms");

        // 测试to_long_flag函数
        assert_eq!(to_long_flag("set_name"), "set-name");
        assert_eq!(to_long_flag("dry_run"), "dry-run");
    }
}
