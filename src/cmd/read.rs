use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use crossterm::terminal;
use cursive::event::Key;
use cursive::theme::{BorderStyle, Color::TerminalDefault, Palette, PaletteColor::*, Theme};
use cursive::view::Nameable;
use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ScrollView, SelectView, TextView,
};
use cursive::{Cursive, CursiveRunnable, With};
use fluent_templates::Loader;
use futures::executor;
use novel_api::{ChapterInfo, CiweimaoClient, Client, ContentInfo, SfacgClient};
use url::Url;

use crate::cmd::{Convert, Source};
use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "read_command").unwrap())]
pub struct Read {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id").unwrap())]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").unwrap())]
    pub source: Source,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").unwrap())]
    pub ignore_keyring: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::DEFAULT_PROXY,
        help = LOCALES.lookup(&LANG_ID, "proxy").unwrap())]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy").unwrap())]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = super::cert_help_msg())]
    pub cert: Option<PathBuf>,
}

pub async fn execute(config: Read) -> Result<()> {
    match config.source {
        Source::Sfacg => {
            let mut client = SfacgClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?
        }
        Source::Ciweimao => {
            let mut client = CiweimaoClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?
        }
    }

    Ok(())
}

type ScrollableTextView = ScrollView<NamedView<TextView>>;
type ScrollableSelectView = ScrollView<NamedView<SelectView<Option<ChapterInfo>>>>;
type HideableSelectView = HideableView<NamedView<ScrollableSelectView>>;

async fn do_execute<T>(client: T, config: Read) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    utils::login(&client, &config.source, config.ignore_keyring).await?;

    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    let mut siv = cursive::default();
    set_theme(&mut siv);
    set_shortcut_keys(&mut siv);

    let mut select = SelectView::new();
    let select_width = (terminal::size()?.0 / 3) as usize;

    for volume in client.volume_infos(config.novel_id).await? {
        let volume_title = utils::convert_str(volume.title, &config.converts)?;
        select.add_item(
            console::truncate_str(&volume_title, select_width, "..."),
            None,
        );

        for chapter in volume.chapter_infos {
            let chapter_title = format!(
                "  {}",
                utils::convert_str(&chapter.title, &config.converts)?
            );

            select.add_item(
                console::truncate_str(&chapter_title, select_width, "..."),
                Some(chapter),
            );
        }
    }

    let client_copy = Arc::clone(&client);
    let convert_copy = config.converts.clone();
    select.set_on_select(move |s, info| {
        if info.is_some() {
            let info = info.as_ref().unwrap();

            // TODO Distinguish between chapters not purchased and inaccessible
            if info.can_download() {
                if let Ok(content) = download(&client_copy, info, &convert_copy) {
                    s.call_on_name("scrollable_text", |view: &mut ScrollableTextView| {
                        view.scroll_to_top();
                    });

                    s.call_on_name("text", |view: &mut TextView| {
                        view.set_content(content);
                    });
                } else {
                    s.add_layer(create_dialog(
                        LOCALES.lookup(&LANG_ID, "download_failed_msg").unwrap(),
                    ));
                }
            } else {
                s.add_layer(create_dialog(
                    LOCALES.lookup(&LANG_ID, "inaccessible_msg").unwrap(),
                ));
            };
        }
    });

    siv.add_fullscreen_layer(
        LinearLayout::horizontal()
            .child(
                HideableView::new(
                    ScrollView::new(select.with_name("select")).with_name("scrollable_select"),
                )
                .with_name("hideable_select"),
            )
            .child(DummyView)
            .child(
                ScrollView::new(
                    TextView::new(introduction(&client, config.novel_id, &config.converts).await?)
                        .with_name("text"),
                )
                .with_name("scrollable_text"),
            ),
    );

    siv.run();

    Ok(())
}

fn set_theme(siv: &mut CursiveRunnable) {
    siv.set_theme(Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::retro().with(|palette| {
            palette[Background] = TerminalDefault;
            palette[Shadow] = TerminalDefault;
            palette[View] = TerminalDefault;
            palette[Primary] = TerminalDefault;
            palette[Secondary] = TerminalDefault;
            palette[Tertiary] = TerminalDefault;
            palette[TitlePrimary] = TerminalDefault;
            palette[TitleSecondary] = TerminalDefault;
            palette[Highlight] = TerminalDefault;
            palette[HighlightInactive] = TerminalDefault;
            palette[HighlightText] = TerminalDefault;
        }),
    });
}

fn set_shortcut_keys(siv: &mut CursiveRunnable) {
    siv.add_global_callback('q', Cursive::quit);
    siv.add_global_callback('h', |s| {
        s.call_on_name("hideable_select", |view: &mut HideableSelectView| {
            if view.is_visible() {
                view.hide();
            } else {
                view.unhide();
            }
        });
    });

    // TODO Handle the situation where a non-downloadable chapter is encountered
    siv.add_global_callback(Key::Left, |s| {
        let callback = s
            .call_on_name("select", |view: &mut SelectView<Option<ChapterInfo>>| {
                let mut callback = view.select_up(1);

                loop {
                    let id = view.selected_id().unwrap();
                    if id == 0 {
                        break;
                    }

                    let (_, info) = view.get_item(id).unwrap();
                    if info.is_none() {
                        callback = view.select_up(1);
                    } else {
                        break;
                    }
                }

                callback
            })
            .unwrap();
        callback(s);

        s.call_on_name("scrollable_select", |view: &mut ScrollableSelectView| {
            view.scroll_to_important_area();
        });
    });

    siv.add_global_callback(Key::Right, |s| {
        let callback = s
            .call_on_name("select", |view: &mut SelectView<Option<ChapterInfo>>| {
                let mut callback = view.select_down(1);

                loop {
                    let id = view.selected_id().unwrap();
                    if id == view.len() - 1 {
                        break;
                    }

                    let (_, info) = view.get_item(id).unwrap();
                    if info.is_none() {
                        callback = view.select_down(1);
                    } else {
                        break;
                    }
                }

                callback
            })
            .unwrap();
        callback(s);

        s.call_on_name("scrollable_select", |view: &mut ScrollableSelectView| {
            view.scroll_to_important_area();
        });
    });
}

fn download<T, E>(client: &Arc<T>, chapter_info: &ChapterInfo, converts: E) -> Result<String>
where
    T: Client + Send + Sync + 'static,
    E: AsRef<[Convert]>,
{
    let mut result = String::with_capacity(8192);
    result.push_str(&utils::convert_str(&chapter_info.title, &converts)?);
    result.push_str("\n\n");

    for info in executor::block_on(client.content_infos(chapter_info))? {
        if let ContentInfo::Text(text) = info {
            result.push_str(&utils::convert_str(&text, &converts)?);
            result.push_str("\n\n");
        } else if let ContentInfo::Image(url) = info {
            result.push_str(url.to_string().as_str());
            result.push_str("\n\n");
        } else {
            unreachable!("ContentInfo can only be Text or Image");
        }
    }

    Ok(result)
}

async fn introduction<T, E>(client: &Arc<T>, novel_id: u32, converts: E) -> Result<String>
where
    T: Client + Send + Sync + 'static,
    E: AsRef<[Convert]>,
{
    let novel_info = utils::novel_info(client, novel_id).await?;

    let mut introduction = String::with_capacity(4096);
    if let Some(lines) = novel_info.introduction {
        for line in lines {
            introduction.push_str(&utils::convert_str(&line, &converts)?);
            introduction.push_str("\n\n");
        }
    }

    Ok(introduction)
}

fn create_dialog<T>(msg: T) -> Dialog
where
    T: AsRef<str>,
{
    Dialog::text(msg.as_ref()).button("Ok", |s| {
        s.pop_layer();
    })
}
