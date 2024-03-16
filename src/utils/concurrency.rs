use std::{num::NonZeroUsize, thread};

pub fn maximum_concurrency() -> usize {
    thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(4).unwrap())
        .into()
}
