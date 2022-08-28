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
        #[clap(short = 'k', long = "key", env, hide_env_values = true, value_hint = clap::ValueHint::Other,)]
        key: String,

        /// Changed URLs for search engines to crawl
        #[clap(
            value_name = "URL",
            required = true,
            use_value_delimiter = false,
            max_values = 10_000,
            value_hint = clap::ValueHint::Url,
        )]
        urls: Vec<String>,

        /// URL of the key file
        #[clap(short = 'l', long, env, value_hint = clap::ValueHint::Url,)]
        key_location: Option<String>,

        /// Endpoint of the `IndexNow.org` search engine API
        #[clap(short = 'e', long, env, value_hint = clap::ValueHint::Url,)]
        endpoint: Option<String>,
    },
}

fn main() {
    let args = argfile::expand_args(argfile::parse_fromfile, argfile::PREFIX).unwrap();
    let cli = CliArguments::parse_from(args);
    println!("{:?}", cli);
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CliArguments::command().debug_assert()
}
