use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::{Client, Keyring, UserInfo};
use requestty::Question;
use tracing::info;

use crate::{cmd::Source, LANG_ID, LOCALES};

pub async fn login<T>(client: &T, source: &Source, ignore_keyring: bool) -> Result<Option<UserInfo>>
where
    T: Client,
{
    let user_info = client.user_info().await?;

    if user_info.is_none() {
        let user_name = get_user_name()?;

        if ignore_keyring {
            let password = get_password()?;
            client.login(user_name, password).await?;
        } else {
            let keyring = Keyring::new(source, &user_name)?;
            let password = keyring.get_password();

            if password.is_ok() {
                info!("Successfully obtained password from Keyring");

                client.login(user_name, password.unwrap()).await?;
            } else {
                info!("Unable to get password from Keyring");

                let password = get_password()?;
                client.login(user_name, password.clone()).await?;

                keyring.set_password(password)?;
            }
        }
    }

    Ok(user_info)
}

fn get_user_name() -> Result<String> {
    let question = Question::input("user_name")
        .message(LOCALES.lookup(&LANG_ID, "enter_user_name").unwrap())
        .build();

    Ok(requestty::prompt_one(question)?
        .as_string()
        .unwrap()
        .to_string())
}

fn get_password() -> Result<String> {
    let question = Question::password("password")
        .message(LOCALES.lookup(&LANG_ID, "enter_password").unwrap())
        .mask('*')
        .build();

    Ok(requestty::prompt_one(question)?
        .as_string()
        .unwrap()
        .to_string())
}
