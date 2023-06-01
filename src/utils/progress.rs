use std::fmt::Write;

use color_eyre::eyre::Result;
use indicatif::{ProgressState, ProgressStyle};

#[must_use]
#[derive(Clone)]
pub struct ProgressBar {
    pb: indicatif::ProgressBar,
    message_width: usize,
}

impl ProgressBar {
    pub fn new(total_size: u64) -> Result<Self> {
        let message_width = (viuer::terminal_size().0 / 3) as usize;

        let pb = indicatif::ProgressBar::new(total_size);

        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {msg:40} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"));

        Ok(Self { pb, message_width })
    }

    pub fn inc<T>(&mut self, msg: T)
    where
        T: AsRef<str>,
    {
        self.pb.set_message(
            console::truncate_str(msg.as_ref(), self.message_width, "...").to_string(),
        );
        self.pb.inc(1);
    }

    pub fn finish(self) {
        self.pb.finish_and_clear();
    }
}
