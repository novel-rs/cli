use std::{path::PathBuf, sync::Arc, time::Duration};

use clap::Args;
use color_eyre::eyre::{bail, Result};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use fluent_templates::Loader;
use novel_api::{
    ChapterInfo, CiweimaoClient, CiyuanjiClient, Client, ContentInfo, NovelInfo, SfacgClient,
    VolumeInfos,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect, Size},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{
        Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget, Wrap,
    },
    Frame,
};
use tokio::{runtime::Handle, task};
use tui_popup::Popup;
use tui_scrollview::{ScrollView, ScrollViewState};
use tui_tree_widget::{Tree, TreeItem, TreeState};
use url::Url;

use crate::{
    cmd::{Convert, Source},
    utils, Tui, LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "read_command"))]
pub struct Read {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id"))]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source"))]
    pub source: Source,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts"))]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring"))]
    pub ignore_keyring: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::DEFAULT_PROXY,
        help = LOCALES.lookup(&LANG_ID, "proxy"))]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy"))]
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
            utils::log_in(&client, &config.source, config.ignore_keyring).await?;
            do_execute(client, config).await?;
        }
        Source::Ciweimao => {
            let mut client = CiweimaoClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            utils::log_in(&client, &config.source, config.ignore_keyring).await?;
            do_execute(client, config).await?;
        }
        Source::Ciyuanji => {
            let mut client = CiyuanjiClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            utils::log_in_without_password(&client).await?;
            do_execute(client, config).await?;
        }
    }

    Ok(())
}

async fn do_execute<T>(client: T, config: Read) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    let mut terminal = crate::init_terminal()?;
    App::new(client, config).await?.run(&mut terminal)?;
    crate::restore_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}

struct App<T> {
    exit: bool,
    percentage: u16,

    chapter_list: ChapterList,
    content_state: ScrollViewState,
    show_subscription: bool,

    chapter_list_area: Rect,
    content_area: Rect,

    config: Read,
    client: Arc<T>,

    money: u32,
    novel_info: NovelInfo,
    volume_infos: VolumeInfos,
}

impl<T> App<T>
where
    T: Client + Send + Sync + 'static,
{
    pub async fn new(client: Arc<T>, config: Read) -> Result<Self> {
        let money = client.money().await?;
        let novel_info = utils::novel_info(&client, config.novel_id).await?;

        let Some(volume_infos) = client.volume_infos(config.novel_id).await? else {
            bail!("Unable to get chapter information");
        };

        let chapter_list = ChapterList::new(&volume_infos, &config.converts)?;

        Ok(App {
            exit: false,
            percentage: 30,
            chapter_list,
            content_state: ScrollViewState::default(),
            chapter_list_area: Rect::default(),
            show_subscription: false,
            content_area: Rect::default(),
            config,
            client,
            money,
            novel_info,
            volume_infos,
        })
    }

    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        terminal.draw(|frame| self.render_frame(frame))?;

        while !self.exit {
            if self.handle_events()? {
                terminal.draw(|frame| self.render_frame(frame))?;
            }
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(16))? {
            return match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                Event::Mouse(mouse_event) => Ok(self.handle_mouse_event(mouse_event)),
                _ => Ok(false),
            };
        }
        Ok(false)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        let result = match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Down => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.content_state.scroll_down();
                    true
                } else {
                    self.content_state.scroll_to_top();
                    self.chapter_list.state.key_down()
                }
            }
            KeyCode::Up => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.content_state.scroll_up();
                    true
                } else {
                    self.content_state.scroll_to_top();
                    self.chapter_list.state.key_up()
                }
            }
            KeyCode::Right => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.increase()
                } else {
                    self.chapter_list.state.key_right()
                }
            }
            KeyCode::Left => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.reduce()
                } else {
                    self.chapter_list.state.key_left()
                }
            }
            KeyCode::Char('y') if self.show_subscription => {
                self.buy_chapter()?;
                self.show_subscription = false;
                true
            }
            _ => false,
        };

        Ok(result)
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        let pos = Position::new(mouse_event.column, mouse_event.row);

        match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                if self.chapter_list_area.contains(pos) {
                    self.chapter_list.state.scroll_down(1)
                } else if self.content_area.contains(pos) {
                    self.content_state.scroll_down();
                    true
                } else {
                    false
                }
            }
            MouseEventKind::ScrollUp => {
                if self.chapter_list_area.contains(pos) {
                    self.chapter_list.state.scroll_up(1)
                } else if self.content_area.contains(pos) {
                    self.content_state.scroll_up();
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self.chapter_list.state.click_at(pos) {
                    self.content_state.scroll_to_top();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn exit(&mut self) -> bool {
        self.exit = true;
        true
    }

    fn increase(&mut self) -> bool {
        if self.percentage <= 45 {
            self.percentage += 5;
            return true;
        }
        false
    }

    fn reduce(&mut self) -> bool {
        if self.percentage >= 25 {
            self.percentage -= 5;
            return true;
        }
        false
    }

    fn render_chapterlist(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let widget = Tree::new(&self.chapter_list.items)
            .unwrap()
            .block(Block::bordered().title(utils::convert_str(
                &self.novel_info.name,
                &self.config.converts,
                false,
            )?))
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );

        StatefulWidget::render(widget, area, buf, &mut self.chapter_list.state);

        Ok(())
    }

    fn render_content(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        if self.chapter_list.state.selected().len() == 2 {
            let chapter_id = self.chapter_list.state.selected()[1];
            let chapter_info = self.find_chapter_info(chapter_id).unwrap();

            if chapter_info.payment_required() {
                let block = Block::bordered().title(utils::convert_str(
                    &chapter_info.title,
                    &self.config.converts,
                    false,
                )?);
                Widget::render(block, area, buf);

                self.show_subscription = true;
            } else {
                let (content, title) = self.content(chapter_id)?;

                let block = Block::bordered().title(title);
                let paragraph = Paragraph::new(content).wrap(Wrap { trim: false });

                let mut scroll_view = ScrollView::new(Size::new(area.width - 1, 100));
                let mut block_area = block.inner(scroll_view.buf().area);

                scroll_view = ScrollView::new(Size::new(
                    area.width - 1,
                    paragraph.line_count(block_area.width) as u16 + 1,
                ));

                let scroll_view_buf = scroll_view.buf_mut();
                block_area = block.inner(scroll_view_buf.area);

                Widget::render(block, scroll_view_buf.area, scroll_view_buf);
                Widget::render(paragraph, block_area, scroll_view_buf);
                StatefulWidget::render(scroll_view, area, buf, &mut self.content_state);

                self.show_subscription = false;
            }
        } else {
            Widget::render(Clear, area, buf);
            self.show_subscription = false;
        }

        Ok(())
    }

    fn render_popup(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        if self.chapter_list.state.selected().len() == 2 {
            let chapter_id = self.chapter_list.state.selected()[1];
            let chapter_info = self.find_chapter_info(chapter_id).unwrap();

            let text = format!(
                "订阅本章：{}，账户余额：{}\n输入 y 订阅",
                chapter_info.price.unwrap(),
                self.money
            );
            let text = Text::styled(
                utils::convert_str(text, &self.config.converts, false)?,
                Style::default().fg(Color::Yellow),
            );
            let popup = Popup::new(
                utils::convert_str("订阅章节", &self.config.converts, false)?,
                text,
            );
            Widget::render(&popup, area, buf);
        }

        Ok(())
    }

    fn content(&mut self, chapter_id: u32) -> Result<(String, String)> {
        let mut result = String::with_capacity(8192);
        let chapter_info = self.find_chapter_info(chapter_id).unwrap();

        let client = Arc::clone(&self.client);
        let content_info = task::block_in_place(move || {
            Handle::current().block_on(async move { client.content_infos(chapter_info).await })
        })?;

        for info in content_info {
            if let ContentInfo::Text(text) = info {
                result.push_str(&utils::convert_str(&text, &self.config.converts, false)?);
                result.push_str("\n\n");
            } else if let ContentInfo::Image(url) = info {
                result.push_str(url.to_string().as_str());
                result.push_str("\n\n");
            } else {
                unreachable!("ContentInfo can only be Text or Image");
            }
        }

        Ok((
            result,
            utils::convert_str(&chapter_info.title, &self.config.converts, false)?,
        ))
    }

    fn buy_chapter(&mut self) -> Result<()> {
        if self.chapter_list.state.selected().len() == 2 {
            let chapter_id = self.chapter_list.state.selected()[1];
            let chapter_info = self.find_chapter_info(chapter_id).unwrap();

            let client = Arc::clone(&self.client);
            task::block_in_place(move || {
                Handle::current().block_on(async move { client.buy_chapter(chapter_info).await })
            })?;

            let chapter_info = self.find_chapter_info_mut(chapter_id).unwrap();
            chapter_info.payment_required = Some(false);

            self.money -= chapter_info.price.unwrap() as u32;

            self.chapter_list.items =
                ChapterList::new(&self.volume_infos, &self.config.converts)?.items;
        }

        Ok(())
    }

    fn find_chapter_info(&self, chapter_id: u32) -> Option<&ChapterInfo> {
        for volume in &self.volume_infos {
            for chapter in &volume.chapter_infos {
                if chapter.id == chapter_id {
                    return Some(chapter);
                }
            }
        }
        None
    }

    fn find_chapter_info_mut(&mut self, chapter_id: u32) -> Option<&mut ChapterInfo> {
        for volume in &mut self.volume_infos {
            for chapter in &mut volume.chapter_infos {
                if chapter.id == chapter_id {
                    return Some(chapter);
                }
            }
        }
        None
    }
}

impl<T> Widget for &mut App<T>
where
    T: Client + Send + Sync + 'static,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(self.percentage),
                Constraint::Percentage(100 - self.percentage),
            ])
            .split(area);

        self.chapter_list_area = layout[0];
        self.content_area = layout[1];

        self.render_chapterlist(layout[0], buf).unwrap();
        self.render_content(layout[1], buf).unwrap();

        if self.show_subscription {
            self.render_popup(area, buf).unwrap();
        }
    }
}

struct ChapterList {
    state: TreeState<u32>,
    items: Vec<TreeItem<'static, u32>>,
}

impl ChapterList {
    fn new(volume_infos: &VolumeInfos, converts: &[Convert]) -> Result<Self> {
        let mut result = Self {
            state: TreeState::default(),
            items: Vec::with_capacity(4),
        };

        for (index, volume_info) in volume_infos.iter().enumerate() {
            let index = index as u32;

            let mut chapters = Vec::with_capacity(32);
            for chapter in &volume_info.chapter_infos {
                if chapter.is_valid() {
                    let mut title_prefix = "";
                    if chapter.payment_required() {
                        title_prefix = "【未订阅】";
                    }

                    chapters.push(TreeItem::new_leaf(
                        chapter.id,
                        utils::convert_str(
                            format!("{title_prefix}{}", chapter.title),
                            converts,
                            true,
                        )?,
                    ));
                }
            }

            if !chapters.is_empty() {
                result.items.push(
                    TreeItem::new(
                        index,
                        utils::convert_str(&volume_info.title, converts, true)?,
                        chapters,
                    )
                    .unwrap(),
                );
            }
        }

        Ok(result)
    }
}
