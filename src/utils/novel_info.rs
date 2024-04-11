use std::sync::Arc;

use color_eyre::eyre::{ensure, Result};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement,
    Table,
};
use image::DynamicImage;
use novel_api::{Client, NovelInfo};
use viuer::{Config, KittySupport};

use crate::{cmd::Convert, utils};

pub async fn novel_info<T>(client: &Arc<T>, novel_id: u32) -> Result<NovelInfo>
where
    T: Client + Send + Sync + 'static,
{
    let novel_info = client.novel_info(novel_id).await?;
    ensure!(
        novel_info.is_some(),
        "The novel does not exist: `{}`",
        novel_id
    );

    Ok(novel_info.unwrap())
}

pub fn print_novel_info<T>(
    image: Option<DynamicImage>,
    novel_info: NovelInfo,
    converts: T,
) -> Result<()>
where
    T: AsRef<[Convert]>,
{
    if viuer::is_iterm_supported() || viuer::get_kitty_support() != KittySupport::None {
        if let Some(image) = image {
            let (width, height) = viuer::terminal_size();

            let config = Config {
                absolute_offset: false,
                width: Some(width as u32 / 2),
                height: Some(height as u32 / 2),
                ..Default::default()
            };

            viuer::print(&image, &config)?;
        }
    }

    let converts = converts.as_ref();

    println!(
        "{}：{}",
        utils::convert_str("名字", converts, false)?,
        utils::convert_str(&novel_info.name, converts, false)?
    );

    println!(
        "{}：{}",
        utils::convert_str("作者", converts, false)?,
        utils::convert_str(&novel_info.author_name, converts, false)?
    );

    if novel_info.category.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("类型", converts, false)?,
            utils::convert_str(try_get_category(&novel_info), converts, false)?
        );
    }

    if novel_info.tags.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("标签", converts, false)?,
            utils::convert_str(try_get_tags(&novel_info), converts, false)?
        );
    }

    if novel_info.word_count.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("字数", converts, false)?,
            utils::convert_str(try_get_word_count(&novel_info), converts, false)?
        );
    }

    if novel_info.is_finished.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("状态", converts, false)?,
            utils::convert_str(try_get_is_finished(&novel_info), converts, false)?
        );
    }

    if novel_info.create_time.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("创建时间", converts, false)?,
            try_get_create_time(&novel_info)
        );
    }

    if novel_info.update_time.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("更新时间", converts, false)?,
            try_get_update_time(&novel_info)
        );
    }

    if novel_info.introduction.is_some() {
        println!(
            "{}：{}",
            utils::convert_str("简介", converts, false)?,
            utils::convert_str(try_get_introduction(&novel_info), converts, false)?
        );
    }

    Ok(())
}

pub fn print_novel_infos<T>(novel_infos: Vec<NovelInfo>, converts: T) -> Result<()>
where
    T: AsRef<[Convert]>,
{
    let converts = converts.as_ref();

    let mut row = vec![
        utils::convert_str("序号", converts, false)?,
        utils::convert_str("编号", converts, false)?,
        utils::convert_str("名字", converts, false)?,
        utils::convert_str("作者", converts, false)?,
    ];

    let category = novel_infos
        .iter()
        .any(|novel_info| novel_info.category.is_some());
    if category {
        row.push(utils::convert_str("类型", converts, false)?);
    }

    let tags = novel_infos
        .iter()
        .any(|novel_info| novel_info.tags.is_some());
    if tags {
        row.push(utils::convert_str("标签", converts, false)?);
    }

    let word_count = novel_infos
        .iter()
        .any(|novel_info| novel_info.word_count.is_some());
    if word_count {
        row.push(utils::convert_str("字数", converts, false)?);
    }

    let is_finished = novel_infos
        .iter()
        .any(|novel_info| novel_info.is_finished.is_some());
    if is_finished {
        row.push(utils::convert_str("状态", converts, false)?);
    }

    let create_time = novel_infos
        .iter()
        .any(|novel_info| novel_info.create_time.is_some());
    if create_time {
        row.push(utils::convert_str("创建时间", converts, false)?);
    }

    let update_time = novel_infos
        .iter()
        .any(|novel_info| novel_info.update_time.is_some());
    if update_time {
        row.push(utils::convert_str("更新时间", converts, false)?);
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
            Cell::new(utils::convert_str(&novel_info.name, converts, false)?),
            Cell::new(utils::convert_str(
                &novel_info.author_name,
                converts,
                false,
            )?),
        ];

        if category {
            row.push(Cell::new(utils::convert_str(
                try_get_category(&novel_info),
                converts,
                false,
            )?));
        }

        if tags {
            row.push(Cell::new(utils::convert_str(
                try_get_tags(&novel_info),
                converts,
                false,
            )?));
        }

        if word_count {
            row.push(
                Cell::new(utils::convert_str(
                    try_get_word_count(&novel_info),
                    converts,
                    false,
                )?)
                .set_alignment(CellAlignment::Right),
            );
        }

        if is_finished {
            row.push(Cell::new(utils::convert_str(
                try_get_is_finished(&novel_info),
                converts,
                false,
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
fn try_get_is_finished(novel_info: &NovelInfo) -> String {
    if novel_info.is_finished.is_some() {
        if novel_info.is_finished.unwrap() {
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
fn try_get_category(novel_info: &NovelInfo) -> String {
    if novel_info.category.is_some() {
        novel_info.category.as_ref().unwrap().name.to_string()
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
