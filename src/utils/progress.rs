use std::fmt::Write;

use indicatif::{ProgressState, ProgressStyle};

#[must_use]
pub(crate) struct ProgressBar {
    pb: indicatif::ProgressBar,
}

impl ProgressBar {
    pub(crate) fn new(total_size: usize) -> Self {
        let pb = indicatif::ProgressBar::new(total_size as u64);

        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {msg:40} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"));

        Self { pb }
    }

    pub(crate) fn inc<T>(&mut self, msg: T)
    where
        T: AsRef<str>,
    {
        self.pb
            .set_message(console::truncate_str(msg.as_ref(), 40, "...").to_string());
        self.pb.inc(1);
    }

    pub(crate) fn finish(&mut self) {
        self.pb.finish_and_clear();
    }
}
