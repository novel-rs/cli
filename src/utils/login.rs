use color_eyre::eyre::Result;
use dialoguer::{theme::ColorfulTheme, Input, Password};
use fluent_templates::Loader;
use novel_api::{Client, Keyring};
use tracing::info;

use crate::{cmd::Source, LANG_ID, LOCALES};

pub async fn log_in<T>(client: &T, source: &Source, ignore_keyring: bool) -> Result<()>
where
    T: Client,
{
    if !client.logged_in().await? {
        let user_name = get_user_name()?;

        if ignore_keyring {
            let password = get_password()?;
            client.log_in(user_name, Some(password)).await?;
        } else {
            let keyring = Keyring::new(source, &user_name)?;
            let password = keyring.get_password();

            if password.is_ok() {
                info!("Successfully obtained password from Keyring");

                client.log_in(user_name, Some(password.unwrap())).await?;
            } else {
                info!("Unable to get password from Keyring");

                let password = get_password()?;
                client.log_in(user_name, Some(password.clone())).await?;

                keyring.set_password(password)?;
            }
        }
    }

    Ok(())
}

pub async fn log_in_without_password<T>(client: &T) -> Result<()>
where
    T: Client,
{
    if !client.logged_in().await? {
        let user_name = get_user_name()?;
        client.log_in(user_name, None).await?;
    }

    Ok(())
}

fn get_user_name() -> Result<String> {
    Ok(Input::with_theme(&ColorfulTheme::default())
        .with_prompt(LOCALES.lookup(&LANG_ID, "enter_user_name"))
        .interact_text()?)
}

fn get_password() -> Result<String> {
    Ok(Password::with_theme(&ColorfulTheme::default())
        .with_prompt(LOCALES.lookup(&LANG_ID, "enter_password"))
        .interact()?)
}
