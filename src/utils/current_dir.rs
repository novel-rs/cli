use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};

pub struct CurrentDir {
    backup_path: PathBuf,
}

impl CurrentDir {
    pub fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        ensure!(
            path.is_dir(),
            "The directory does not exist and cannot be set as the current working directory: `{}`",
            path.display()
        );

        let backup_path = env::current_dir()?;
        env::set_current_dir(path)?;

        Ok(Self { backup_path })
    }

    pub fn restore(self) -> Result<()> {
        env::set_current_dir(self.backup_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    #[test]
    fn current_dir() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let backup_dir = env::current_dir()?;

        let current_dir = CurrentDir::new(temp_dir.path())?;
        assert_ne!(env::current_dir()?, backup_dir);

        current_dir.restore()?;
        assert_eq!(env::current_dir()?, backup_dir);

        Ok(())
    }
}
