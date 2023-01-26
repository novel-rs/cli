use std::io::{self, Write};

use anyhow::Result;
use novel_api::{Client, Keyring};
use tracing::info;

use crate::cmd::Source;

pub(crate) async fn login<T>(client: &T, source: Source, ignore_keyring: bool) -> Result<()>
where
    T: Client,
{
    let user_info = client.user_info().await?;

    if user_info.is_none() {
        let username = get_username()?;

        if ignore_keyring {
            let password = get_password()?;
            client.login(username, password).await?;
        } else {
            let keyring = Keyring::new(source, &username);
            let password = keyring.get_password();

            if password.is_ok() {
                info!("Successfully obtained password from Keyring");
                client.login(username, password.unwrap()).await?;
            } else {
                info!("Unable to get password from keyring");

                let password = get_password()?;
                client.login(username, &password).await?;

                keyring.set_password(password)?;
            }
        }
    }

    Ok(())
}

fn get_username() -> Result<String> {
    print!("Username: ");
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    Ok(buffer.trim().to_string())
}

fn get_password() -> Result<String> {
    let password = rpassword::prompt_password("Password: ")?;
    Ok(password)
}
