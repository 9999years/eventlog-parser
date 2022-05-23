use std::path::PathBuf;
use structopt::StructOpt;

mod parser;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Input eventlog.
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();
    let input = std::fs::read(&opt.input)?;
    let (_rest, (types, events)) = parser::parse_eventlog(&input)?;

    println!("Types: {types:?}");
    println!("Events: {events:?}");

    Ok(())
}
