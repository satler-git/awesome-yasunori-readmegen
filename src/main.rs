use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use serde::Deserialize;
use std::fs;
use chrono::{Datelike, NaiveDate};

const TABLE_HEADER: &str = r#"
| date           | senpan            | place                  | title                                                        |
|----------------|-------------------|------------------------|--------------------------------------------------------------|
"#;

const MARKDOWN_HEADER: &str = r#"# Awesome yasunori

A curated list of awesome yasunori, the post about
[yasunori0418](https://github.com/yasunori0418). Inspired by
[mattn/awesome-sonomasakada](https://github.com/mattn/awesome-sonomasakada).

> [!CAUTION]
> It's a story YOU([takeokunn](https://github.com/takeokunn)) started by use
> ME([yasunori0418](https://github.com/yasunori0418))!!
>
> お前([takeokunn](https://github.com/takeokunn))が俺([yasunori0418](https://github.com/yasunori0418))で始めた物語だろ！！
>
> by [yasunori0418(原義)](https://github.com/yasunori0418)

## Contributing

Please take a quick gander at the
[contribution guidelines](https://github.com/takeokunn/awesome-yasunori/blob/master/CONTRIBUTING.md)
first. Thanks to all
[contributors](https://github.com/takeokunn/awesome-yasunori/graphs/contributors);
you rock!

## Indexes

"#;

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct YasunoriEntry {
    title: String,
    date: NaiveDate,
    content: String,
    meta: String,
    at: String,
    senpan: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
struct YasunoriEntryRaw {
    title: String,
    date: NaiveDate,
    content: String,
    meta: Option<String>,
    at: String,
    senpan: String,
}
#[derive(Deserialize, Debug, Eq, PartialEq)]
struct ConfigRaw {
    yasunori: Vec<YasunoriEntryRaw>,
}
#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Config {
    yasunori: Vec<YasunoriEntry>,
}

fn serialize_naive_date(date: &NaiveDate) -> String {
    format!("{date} {}", date.weekday())
}

fn entry_from_toml(toml_str: String) -> Result<Config> {
    let raw: ConfigRaw = toml::from_str(&toml_str).context("Unable to parse the toml")?;
    Ok(Config {
        yasunori: raw.yasunori.iter().map(|yi| {
        let yi = yi.clone();
        YasunoriEntry {
                title: yi.title,
                date: yi.date,
                content: yi.content,
                meta: yi.meta.unwrap_or_default(),
                at: yi.at,
                senpan: yi.senpan,
        }}).collect()
    })
}

fn make_table(cfg: &Config) -> String {
    let mut table_ctx = String::new(); // TODO: chronoをもとにソートを追加するならここ
    for yi in &cfg.yasunori {
        let column = format!(
            "| {} | {} | {} | {} |\n",
            serialize_naive_date(&yi.date), yi.senpan, yi.at, yi.title
        );
        table_ctx = format!("{table_ctx}{column}");
    }
    format!("{TABLE_HEADER}{table_ctx}")
}

fn make_markdown_contents(cfg: &Config) -> String {
    (&cfg.yasunori)
        .iter()
        .map(make_markdown_content)
        .collect::<Vec<String>>()
        .join("\n")
}

fn make_markdown_content(entry: &YasunoriEntry) -> String {
    // TODO: 改行コードの扱い
    format!(
        "
### {} ({})

{} by {}

```markdown
{}```

{}",
        entry.title, serialize_naive_date(&entry.date), entry.at, entry.senpan, entry.content, entry.meta
    )
}

fn make_content_all(cfg: &Config) -> String {
    format!(
        "{MARKDOWN_HEADER}{}
## Contents

{}
",
        make_table(&cfg),
        make_markdown_contents(&cfg)
    )
}

/// Simple readme generatar from toml
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the toml file
    #[arg(value_name = "FILEPATH", index = 1)]
    path: Utf8PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let toml = fs::read_to_string(args.path).context("File does not exit")?;
    let cfg = entry_from_toml(toml)?;
    println!("{}", make_content_all(&cfg));
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{make_markdown_content, make_table, serialize_naive_date, Config, YasunoriEntry};

    use super::entry_from_toml;
    use anyhow::Result;
    use chrono::NaiveDate;
    #[test]
    fn test_entry_from_toml() -> Result<()> {
        assert_eq!(
            entry_from_toml(
                r#"
[[yasunori]]
title = "Hello"
date = "2024-09-30"
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
                    date: NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(), // 月曜日
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
                    date: NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
                    at: "vim-jp".into(),
                    senpan: "None".into(),
                    content: String::new(),
                    meta: String::new()
                }]
            }),
            r#"
| date           | senpan            | place                  | title                                                        |
|----------------|-------------------|------------------------|--------------------------------------------------------------|
| 2024-09-30 Mon | None | vim-jp | Hello |
"#
        );
        Ok(())
    }

    #[test]
    fn test_make_markdown_content() -> Result<()> {
        assert_eq!(
            make_markdown_content(&YasunoriEntry {
                title: "brain-yasu**ri".into(),
                date: NaiveDate::from_ymd_opt(2024, 9, 29).unwrap(),
                content: "content\n".into(),
                meta: "memo\n".into(),
                at: "vim-jp #times-yasunori".into(),
                senpan: "takeokunn".into()
            }),
            r#"
### brain-yasu**ri (2024-09-29 Sun)

vim-jp #times-yasunori by takeokunn

```markdown
content
```

memo
"#
            .to_string()
        );
        Ok(())
    }
    #[test]
    fn test_serialize_naive_date() -> Result<()> {
        assert_eq!("2024-09-30 Mon", serialize_naive_date(&NaiveDate::from_ymd_opt(2024, 9, 30).unwrap()));
        Ok(())
    }
}
