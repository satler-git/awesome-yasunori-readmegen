use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use std::fs;

const TABLE_HEADER: &str = r#"
| id | date           | senpan            | place                  | title                                                        |
|----|----------------|-------------------|------------------------|--------------------------------------------------------------|
"#;

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct YasunoriEntry {
    id: u32,
    title: String,
    date: NaiveDate,
    content: String,
    meta: String,
    at: String,
    senpan: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
struct YasunoriEntryRaw {
    id: u32,
    title: String,
    date: NaiveDate,
    content: String,
    meta: Option<String>,
    at: String,
    senpan: String,
}
#[derive(Deserialize, Debug, Eq, PartialEq)]
struct ConfigRaw {
    markdown_header: String,
    yasunori: Vec<YasunoriEntryRaw>,
}
#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Config {
    markdown_header: String,
    yasunori: Vec<YasunoriEntry>,
}

fn serialize_naive_date(date: &NaiveDate) -> String {
    format!("{date}")
}

fn entry_from_toml(toml_str: String) -> Result<Config> {
    let raw: ConfigRaw = toml::from_str(&toml_str).context("Unable to parse the toml")?;
    Ok(Config {
        markdown_header: raw.markdown_header,
        yasunori: raw
            .yasunori
            .iter()
            .map(|yi| {
                let yi = yi.clone();
                YasunoriEntry {
                    id: yi.id,
                    title: yi.title,
                    date: yi.date,
                    content: yi.content,
                    meta: yi.meta.unwrap_or_default(),
                    at: yi.at,
                    senpan: yi.senpan,
                }
            })
            .collect(),
    })
}

fn make_table(cfg: &Config) -> String {
    let mut table_ctx = String::new(); // TODO: chronoをもとにソートを追加するならここ
    for yi in &cfg.yasunori {
        let column = format!(
            "| [{}]({}) | {} | {} | {} | {} |\n",
            yi.id,
            make_anchor_link(&yi.title, &yi.date),
            serialize_naive_date(&yi.date),
            yi.senpan,
            yi.at,
            yi.title
        );
        table_ctx = format!("{table_ctx}{column}");
    }
    format!("{TABLE_HEADER}{table_ctx}")
}

fn make_anchor_link(title: &str, date: &NaiveDate) -> String {
    let re = Regex::new(r#"[!@#$%^&*()+|~=`\[\]{};':",.<>?]|[！”＃＄％＆’（）＊＋，－．／：；＜＝＞？＠［＼］＾＿｀｛｜｝～]"#).unwrap(); // 全角記号は変になるかも
    let lower_spaceless_title = title.replace("　", "-").replace(" ", "-").to_lowercase();
    format!(
        "#{}-{}",
        re.replace_all(&lower_spaceless_title, ""),
        date.to_string().to_lowercase()
    ) // HACK:
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
        entry.title,
        serialize_naive_date(&entry.date),
        entry.at,
        entry.senpan,
        entry.content,
        entry.meta
    )
}

fn make_content_all(cfg: &Config) -> String {
    format!(
        "{}{}
## Contents

{}
",
        &cfg.markdown_header,
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
    use crate::{
        make_anchor_link, make_markdown_content, make_table, serialize_naive_date, Config,
        YasunoriEntry,
    };

    use super::entry_from_toml;
    use anyhow::Result;
    use chrono::NaiveDate;
    #[test]
    fn test_entry_from_toml() -> Result<()> {
        assert_eq!(
            entry_from_toml(
                r#"
markdown_header = ""
[[yasunori]]
id = 1
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
                markdown_header: String::new(),
                yasunori: vec![YasunoriEntry {
                    id: 1,
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
                markdown_header: String::new(),
                yasunori: vec![YasunoriEntry {
                    id: 1,
                    title: "Hello World!".into(),
                    date: NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
                    at: "vim-jp".into(),
                    senpan: "None".into(),
                    content: String::new(),
                    meta: String::new()
                }]
            }),
            r#"
| id | date           | senpan            | place                  | title                                                        |
|----|----------------|-------------------|------------------------|--------------------------------------------------------------|
| [1](#hello-world-2024-09-30) | 2024-09-30 | None | vim-jp | Hello World! |
"#
        );
        Ok(())
    }

    #[test]
    fn test_make_markdown_content() -> Result<()> {
        assert_eq!(
            make_markdown_content(&YasunoriEntry {
                id: 1,
                title: "brain-yasu**ri".into(),
                date: NaiveDate::from_ymd_opt(2024, 9, 29).unwrap(),
                content: "content\n".into(),
                meta: "memo\n".into(),
                at: "vim-jp #times-yasunori".into(),
                senpan: "takeokunn".into()
            }),
            r#"
### brain-yasu**ri (2024-09-29)

vim-jp #times-yasunori by takeokunn

```markdown
content
```

memo
"#
        );
        Ok(())
    }
    #[test]
    fn test_serialize_naive_date() -> Result<()> {
        assert_eq!(
            "2024-09-30",
            serialize_naive_date(&NaiveDate::from_ymd_opt(2024, 9, 30).unwrap())
        );
        Ok(())
    }
    #[test]
    fn test_make_anchor_link() -> Result<()> {
        assert_eq!(
            make_anchor_link(
                "サンプルセクション",
                &NaiveDate::from_ymd_opt(2024, 9, 30).unwrap()
            ),
            "#サンプルセクション-2024-09-30",
        );
        assert_eq!(
            make_anchor_link(
                "テスト! セクション",
                &NaiveDate::from_ymd_opt(2024, 9, 30).unwrap()
            ),
            "#テスト-セクション-2024-09-30"
        );
        assert_eq!(
            make_anchor_link(
                "HELLO WORLD",
                &NaiveDate::from_ymd_opt(2024, 9, 30).unwrap()
            ),
            "#hello-world-2024-09-30"
        );
        assert_eq!(
            make_anchor_link("-_!!", &NaiveDate::from_ymd_opt(2024, 9, 30).unwrap()),
            "#-_-2024-09-30"
        );
        Ok(())
    }
}
