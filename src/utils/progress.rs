use std::fmt::Write;

use indicatif::{ProgressState, ProgressStyle};

#[must_use]
pub struct ProgressBar {
    pb: indicatif::ProgressBar,
}

impl ProgressBar {
    pub fn new(total_size: usize) -> Self {
        let pb = indicatif::ProgressBar::new(total_size as u64);

        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {msg:40} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"));

        Self { pb }
    }

    pub fn inc<T>(&mut self, msg: T)
    where
        T: AsRef<str>,
    {
        self.pb.set_message(
            console::truncate_str(msg.as_ref(), (viuer::terminal_size().0 / 3) as usize, "...")
                .to_string(),
        );
        self.pb.inc(1);
    }

    pub fn finish(&mut self) {
        self.pb.finish_and_clear();
    }
}
