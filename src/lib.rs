pub mod cmd;
pub mod config;
pub mod renderer;
pub mod utils;

use std::{
    env,
    io::{self, Stdout},
    panic,
};

use color_eyre::{
    config::HookBuilder,
    eyre::{self, Result},
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use fluent_templates::static_loader;
use once_cell::sync::Lazy;
use ratatui::{backend::CrosstermBackend, Terminal};
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
        // Windows terminal does not seem to support isolating marks
        // See https://github.com/XAMPPRocky/fluent-templates/issues/21
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

pub static LANG_ID: Lazy<LanguageIdentifier> = Lazy::new(|| {
    let mut locale = sys_locale::get_locale().unwrap_or_else(|| {
        eprintln!("Failed to get active locale for the system, use `en-US`");
        String::from("en-US")
    });

    if locale == "zh-CN" {
        locale = "zh-Hans".to_string();
    } else if locale == "zh-HK" || locale == "zh-TW" {
        locale = "zh-Hant".to_string();
    } else if locale == "C" {
        locale = "en-US".to_string();
    }

    match locale.parse::<LanguageIdentifier>() {
        Ok(lang_id) => lang_id,
        Err(error) => {
            eprintln!("Failed to parse LanguageIdentifier: {error}, use `en-US`");
            "en-US".parse::<LanguageIdentifier>().unwrap()
        }
    }
});

pub fn init_error_hooks(restore: bool) -> Result<()> {
    if env::var("RUST_SPANTRACE").is_err() {
        env::set_var("RUST_SPANTRACE", "0");
    }

    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if restore {
            let _ = restore_terminal();
        }
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |e| {
        if restore {
            let _ = restore_terminal();
        }
        eyre_hook(e)
    }))?;

    Ok(())
}

pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

pub(crate) fn init_terminal() -> Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal::enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

pub(crate) fn restore_terminal() -> Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
