use std::io;

use anyhow::{ensure, Result};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement,
    Table,
};
use image::DynamicImage;
use is_terminal::IsTerminal;
use novel_api::{Client, NovelInfo};
use viuer::{Config, KittySupport};

use crate::{cmd::Convert, utils};

pub(crate) async fn novel_info<T>(client: &T, novel_id: u32) -> Result<NovelInfo>
where
    T: Client,
{
    let novel_info = client.novel_info(novel_id).await?;
    ensure!(novel_info.is_some(), "The novel does not exist");

    Ok(novel_info.unwrap())
}

pub(crate) fn print_novel_info(
    image: Option<DynamicImage>,
    novel_info: NovelInfo,
    converts: &Vec<Convert>,
) -> Result<()> {
    if io::stdout().is_terminal()
        && (viuer::is_iterm_supported() || viuer::get_kitty_support() != KittySupport::None)
    {
        if let Some(image) = image {
            let (width, height) = viuer::terminal_size();

            let config = Config {
                absolute_offset: false,
                width: Some(width as u32 / 4),
                height: Some(height as u32 / 4),
                ..Default::default()
            };

            viuer::print(&image, &config)?;
        }
    }

    println!(
        "{}：{}",
        utils::convert_str("名字", converts)?,
        utils::convert_str(&novel_info.name, converts)?
    );

    println!(
        "{}：{}",
        utils::convert_str("作者", converts)?,
        utils::convert_str(&novel_info.author_name, converts)?
    );

    println!(
        "{}：{}",
        utils::convert_str("类型", converts)?,
        utils::convert_str(try_get_genre(&novel_info), converts)?
    );

    println!(
        "{}：{}",
        utils::convert_str("标签", converts)?,
        utils::convert_str(try_get_tags(&novel_info), converts)?
    );

    println!(
        "{}：{}",
        utils::convert_str("字数", converts)?,
        utils::convert_str(try_get_word_count(&novel_info), converts)?
    );

    println!(
        "{}：{}",
        utils::convert_str("状态", converts)?,
        utils::convert_str(try_get_finished(&novel_info), converts)?
    );

    println!(
        "{}：{}",
        utils::convert_str("创建时间", converts)?,
        try_get_create_time(&novel_info)
    );

    println!(
        "{}：{}",
        utils::convert_str("更新时间", converts)?,
        try_get_update_time(&novel_info)
    );

    println!(
        "{}：{}",
        utils::convert_str("简介", converts)?,
        utils::convert_str(try_get_introduction(&novel_info), converts)?
    );

    Ok(())
}

pub(crate) fn print_novel_infos(
    novel_infos: Vec<NovelInfo>,
    converts: &Vec<Convert>,
) -> Result<()> {
    let mut row = vec![
        utils::convert_str("序号", converts)?,
        utils::convert_str("编号", converts)?,
        utils::convert_str("名字", converts)?,
        utils::convert_str("作者", converts)?,
    ];

    let mut genre = false;
    let mut tags = false;
    let mut word_count = false;
    let mut finished = false;
    let mut create_time = false;
    let mut update_time = false;

    for item in &novel_infos {
        if item.genre.is_some() {
            genre = true;
            row.push(utils::convert_str("类型", converts)?);
            break;
        }
    }

    for item in &novel_infos {
        if item.tags.is_some() {
            tags = true;
            row.push(utils::convert_str("标签", converts)?);
            break;
        }
    }

    for item in &novel_infos {
        if item.word_count.is_some() {
            word_count = true;
            row.push(utils::convert_str("字数", converts)?);
            break;
        }
    }

    for item in &novel_infos {
        if item.finished.is_some() {
            finished = true;
            row.push(utils::convert_str("状态", converts)?);
            break;
        }
    }

    for item in &novel_infos {
        if item.create_time.is_some() {
            create_time = true;
            row.push(utils::convert_str("创建时间", converts)?);
            break;
        }
    }

    for item in &novel_infos {
        if item.update_time.is_some() {
            update_time = true;
            row.push(utils::convert_str("更新时间", converts)?);
            break;
        }
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(row);

    let mut index = 1;
    for novel_info in novel_infos {
        let mut row = vec![
            Cell::new(index.to_string()),
            Cell::new(novel_info.id.to_string()),
            Cell::new(utils::convert_str(&novel_info.name, converts)?),
            Cell::new(utils::convert_str(&novel_info.author_name, converts)?),
        ];

        if genre {
            row.push(Cell::new(utils::convert_str(
                try_get_genre(&novel_info),
                converts,
            )?));
        }

        if tags {
            row.push(Cell::new(utils::convert_str(
                try_get_tags(&novel_info),
                converts,
            )?));
        }

        if word_count {
            row.push(
                Cell::new(utils::convert_str(
                    try_get_word_count(&novel_info),
                    converts,
                )?)
                .set_alignment(CellAlignment::Right),
            );
        }

        if finished {
            row.push(Cell::new(utils::convert_str(
                try_get_finished(&novel_info),
                converts,
            )?));
        }

        if create_time {
            row.push(Cell::new(try_get_create_time(&novel_info)));
        }

        if update_time {
            row.push(Cell::new(try_get_update_time(&novel_info)));
        }

        table.add_row(row);

        index += 1;
    }

    println!("{table}");

    Ok(())
}

#[must_use]
fn try_get_introduction(novel_info: &NovelInfo) -> String {
    if novel_info.introduction.is_some() {
        novel_info.introduction.as_ref().unwrap().join("\n")
    } else {
        String::default()
    }
}

#[must_use]
fn try_get_word_count(novel_info: &NovelInfo) -> String {
    if novel_info.word_count.is_some() {
        let word_count = novel_info.word_count.as_ref().unwrap();

        if *word_count >= 10000 {
            format!("{}万", word_count / 10000)
        } else {
            word_count.to_string()
        }
    } else {
        String::default()
    }
}

#[must_use]
fn try_get_finished(novel_info: &NovelInfo) -> String {
    if novel_info.finished.is_some() {
        if novel_info.finished.unwrap() {
            String::from("已完结")
        } else {
            String::from("未完结")
        }
    } else {
        String::default()
    }
}

#[must_use]
fn try_get_create_time(novel_info: &NovelInfo) -> String {
    if novel_info.create_time.is_some() {
        novel_info.create_time.as_ref().unwrap().to_string()
    } else {
        String::default()
    }
}

#[must_use]
fn try_get_update_time(novel_info: &NovelInfo) -> String {
    if novel_info.update_time.is_some() {
        novel_info.update_time.as_ref().unwrap().to_string()
    } else {
        String::default()
    }
}

#[must_use]
fn try_get_genre(novel_info: &NovelInfo) -> String {
    if novel_info.genre.is_some() {
        novel_info.genre.as_ref().unwrap().to_string()
    } else {
        String::default()
    }
}

#[must_use]
fn try_get_tags(novel_info: &NovelInfo) -> String {
    if novel_info.tags.is_some() {
        novel_info
            .tags
            .as_ref()
            .unwrap()
            .iter()
            .map(|tag| tag.name.to_string())
            .collect::<Vec<String>>()
            .join("、")
    } else {
        String::default()
    }
}
