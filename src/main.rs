use anyhow::{Context, Result};
use serde::Deserialize;

const TABLE_HEADER: &str = r#"
| date           | senpan            | place                  | title                                                        |
|----------------|-------------------|------------------------|--------------------------------------------------------------|
"#;

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct YasunoriEntry {
    title: String,
    date: String, // TODO: chrono
    content: String,
    meta: String,
    at: String,
    senpan: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Config {
    yasunori: Vec<YasunoriEntry>,
}

fn entry_from_toml(toml_str: String) -> Result<Config> {
    toml::from_str(&toml_str).context("Unable to parse the toml")
}

fn make_table(cfg: &Config) -> String {
    let mut table_ctx = String::new(); // TODO: chronoをもとにソートを追加するならここ
    for yi in &cfg.yasunori {
        let column = format!(
            "| {} | {} | {} | {} |\n",
            yi.date, yi.senpan, yi.at, yi.title
        );
        table_ctx = format!("{table_ctx}{column}");
    }
    format!("{TABLE_HEADER}{table_ctx}")
}

fn make_markdown_contents(cfg: &Config) -> String {
    todo!()
}

fn make_markdown_content(entry: &YasunoriEntry) -> String {
    // TODO: 改行コードの扱い
    format!(
        "
### {} ({})

{} by {}

{}
{}",
        entry.title, entry.date, entry.at, entry.senpan, entry.content, entry.meta
    )
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::{make_markdown_content, make_table, Config, YasunoriEntry};

    use super::entry_from_toml;
    use anyhow::Result;
    #[test]
    fn test_entry_from_toml() -> Result<()> {
        assert_eq!(
            entry_from_toml(
                r#"
[[yasunori]]
title = "Hello"
date = "Sooooo long ago"
at = "Earth"
senpan = ""
content = """
yasunori said,
Let there be light.
"""
meta = """
"""
        "#
                .into()
            )?,
            Config {
                yasunori: vec![YasunoriEntry {
                    title: "Hello".into(),
                    date: "Sooooo long ago".into(),
                    content: "yasunori said,
Let there be light.\n"
                        .into(),
                    at: "Earth".into(),
                    senpan: "".into(),
                    meta: "".into()
                }]
            }
        );
        Ok(())
    }

    #[test]
    fn test_make_table() -> Result<()> {
        assert_eq!(
            make_table(&Config {
                yasunori: vec![YasunoriEntry {
                    title: "Hello".into(),
                    date: "date".into(),
                    at: "vim-jp".into(),
                    senpan: "None".into(),
                    content: String::new(),
                    meta: String::new()
                }]
            }),
            r#"
| date           | senpan            | place                  | title                                                        |
|----------------|-------------------|------------------------|--------------------------------------------------------------|
| date | None | vim-jp | Hello |
"#
        );
        Ok(())
    }

    #[test]
    fn test_make_markdown_content() -> Result<()> {
        assert_eq!(
            make_markdown_content(&YasunoriEntry {
                title: "brain-yasu**ri".into(),
                date: "2024-09-29 Sun".into(),
                content: "content\n".into(),
                meta: "memo\n".into(),
                at: "vim-jp #times-yasunori".into(),
                senpan: "takeokunn".into()
            }),
            r#"
### brain-yasu**ri (2024-09-29 Sun)

vim-jp #times-yasunori by takeokunn

content

memo
"#
            .to_string()
        );
        Ok(())
    }
}
