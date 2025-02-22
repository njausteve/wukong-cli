use crate::{
    auth::login,
    config::{AuthConfig, Config, CONFIG_FILE},
    error::CliError,
    graphql::QueryClientBuilder,
    GlobalContext,
};
use dialoguer::{theme::ColorfulTheme, Select};
use owo_colors::OwoColorize;

pub async fn handle_init(
    context: GlobalContext,
    existing_config: Option<Config>,
) -> Result<bool, CliError> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let mut login_selections = vec!["Log in with a new account"];
    if let Some(ref account) = context.account {
        login_selections.splice(..0, vec![account.as_str()]);
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose the account you would like to use to perform operations for this configuration:")
                .default(0)
                .items(&login_selections[..])
                .interact()
                .unwrap();

    // "Log in with a new account" is selected
    let mut config = if selection == login_selections.len() - 1 {
        let auth_info = login().await?;

        // we don't really care about the exisiting config (id_token, refresh_token, account)
        // if the user choose to log in with a new account
        let config = Config {
            auth: Some(AuthConfig {
                account: auth_info.account.clone(),
                id_token: auth_info.id_token,
                access_token: auth_info.access_token,
                expiry_time: auth_info.expiry_time,
                refresh_token: auth_info.refresh_token,
            }),
            ..Default::default()
        };

        println!("You are logged in as: [{}].\n", auth_info.account);
        config
    } else {
        println!("You are logged in as: [{}].\n", login_selections[selection]);

        let mut new_config = Config::default();

        if let Some(ref config) = existing_config {
            new_config.auth = config.auth.clone();
        }

        new_config
    };

    // SAFETY: The auth must not be None here
    let auth_config = config.auth.as_ref().unwrap();

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(auth_config.id_token.clone())
        .build()?;

    let applications_data: Vec<String> = client
        .fetch_application_list()
        .await?
        .data
        .unwrap()
        .applications
        .iter()
        .map(|application| application.name.clone())
        .collect();

    let application_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select the application")
        .default(0)
        .items(&applications_data[..])
        .interact()
        .unwrap();

    println!(
        "Your current application has been set to: [{}].",
        &applications_data[application_selection]
    );

    config.core.application = applications_data[application_selection].clone();

    println!(
        r#"
Your Wukong CLI is configured and ready to use!

* Commands that require authentication will use `{}` by default
* Commands will reference application `{}` by default
Run `wukong config help` to learn how to change individual settings

Some things to try next:

* Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                     "#,
        auth_config.account.green(),
        config.core.application.green()
    );

    if let Some(ref config_file) = *CONFIG_FILE {
        config.save(config_file).expect("Config file save failed");
    }

    Ok(true)
}
