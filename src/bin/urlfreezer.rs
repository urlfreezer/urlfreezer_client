use csv::{Reader, Writer};
use std::io::{stdin, stdout};
use std::path::PathBuf;
use structopt::StructOpt;
use urlfreezer_client::blocking::Client;

#[derive(StructOpt)]
#[structopt(
    name = "urlfreezer",
    author = "URL Freezer",
    about = "client cli for interact with urlfreezer service"
)]
struct Cli {
    #[structopt(long, short)]
    user_id: String,

    #[structopt(parse(from_os_str), long, short)]
    input_file: Option<PathBuf>,

    #[structopt(parse(from_os_str), long, short)]
    output_file: Option<PathBuf>,

    #[structopt(long, short)]
    host: Option<String>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::from_args();

    let client = if let Some(host) = cli.host {
        Client::connect_host(&host, &cli.user_id)?
    } else {
        Client::connect(&cli.user_id)?
    };
    if let Some(f) = cli.input_file {
        if let Some(out) = cli.output_file {
            client.fetch_with_csv(Reader::from_path(f)?, Writer::from_path(out)?)?;
        } else {
            client.fetch_with_csv(Reader::from_path(f)?, Writer::from_writer(stdout()))?;
        }
    } else {
        if let Some(out) = cli.output_file {
            client.fetch_with_csv(Reader::from_reader(stdin()), Writer::from_path(out)?)?;
        } else {
            client.fetch_with_csv(Reader::from_reader(stdin()), Writer::from_writer(stdout()))?;
        }
    };

    Ok(())
}
