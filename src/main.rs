use clap::Parser as _;

/// Submit changed URLs for search engines to crawl using `IndexNow.org` API
#[derive(Debug, clap::Parser)]
struct CliArguments {
    #[clap(subcommand)]
    command: CliCommands,
}

#[derive(Debug, clap::Subcommand)]
enum CliCommands {
    /// Submit URLs for search engines to crawl
    Submit {
        /// Key to verify ownership of submitted URLs
        #[clap(
            short = 'k',
            long = "key",
            env = "INDEXNOW_KEY",
            hide_env_values = true,
            value_hint = clap::ValueHint::Other,
        )]
        key: indexnow::Key,

        /// Changed URLs for search engines to crawl
        #[clap(
            value_name = "URL",
            required = true,
            use_value_delimiter = false,
            num_args = 1..=10_000,
            value_hint = clap::ValueHint::Url,
        )]
        urls: Vec<indexnow::ContentUrl>,

        /// URL of the key file
        #[clap(
            short = 'l',
            long,
            env = "INDEXNOW_KEY_LOCATION",
            value_hint = clap::ValueHint::Url,
        )]
        key_location: Option<indexnow::KeyfileUrl>,

        /// Endpoint of the `IndexNow.org` search engine API
        #[clap(
            short = 'e',
            long,
            env = "INDEXNOW_ENDPOINT",
            value_hint = clap::ValueHint::Url,
            default_value_t,
        )]
        endpoint: indexnow::EndpointUrl,
    },
}

#[derive(Debug, thiserror::Error)]
enum IndexnowCliError {
    #[error("client")]
    Client(#[source] indexnow::client::ClientError),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<std::process::ExitCode, crate::IndexnowCliError> {
    let args = argfile::expand_args(argfile::parse_fromfile, argfile::PREFIX).unwrap();
    let cli = CliArguments::parse_from(args);
    println!("{:?}", cli);

    match cli.command {
        CliCommands::Submit {
            endpoint,
            key,
            key_location,
            urls,
        } => {
            let key_location = match key_location {
                None => indexnow::KeyfileLocation::RootDirectory,
                Some(url) => indexnow::KeyfileLocation::Url(url),
            };
            let keyfile_config = indexnow::KeyfileConfig::new(key, key_location);

            let client =
                indexnow::Client::from_reqwest(endpoint, keyfile_config, reqwest::Client::new());

            if urls.len() == 1 {
                let result = client
                    .submit_one(&urls[0])
                    .await
                    .map_err(IndexnowCliError::Client)?;
                dbg!(&result);
            } else {
                let result = client
                    .submit_set(&urls)
                    .await
                    .map_err(IndexnowCliError::Client)?;
                dbg!(&result);
            }
        }
    }

    Ok(std::process::ExitCode::SUCCESS)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CliArguments::command().debug_assert()
}
