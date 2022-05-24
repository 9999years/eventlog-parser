#![deny(elided_lifetimes_in_paths)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

mod parser;
use parser::Event;
use parser::EventType;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Input eventlog.
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();
    let input = std::fs::read(&opt.input)?;
    let (types, events) = parser::parse_eventlog(&input)
        .map_err(|err| eyre::eyre!(format!("{} at {:?}", err.code.description(), err.input)))?;

    println!("Types:");
    for (
        i,
        EventType {
            id,
            size,
            description,
            extra_info,
        },
    ) in types.iter().enumerate()
    {
        println!("  {i:>4}: id:    {id}");
        println!("        size:  {size:?}");
        println!("        desc:  {}", String::from_utf8_lossy(description));
        if !extra_info.is_empty() {
            println!("        extra: {}", String::from_utf8_lossy(extra_info));
        }
    }

    let types_map: HashMap<_, _> = types
        .iter()
        .map(|ty| (ty.id, String::from_utf8_lossy(&ty.description).clone()))
        .collect();

    println!("Events:");
    for (i, Event { ty, time, data }) in events.iter().enumerate() {
        let ty_desc = types_map
            .get(ty)
            .map(|s| s.to_owned().into_owned())
            .unwrap_or_default();

        println!("  {i:>4}: id:    {ty} ({ty_desc})");
        println!("        time:  {:?}", Duration::from_nanos(*time));
        if !data.is_empty() {
            println!("        data:  {} bytes", data.len());
            match String::from_utf8(data.clone()) {
                Ok(data) if !data.trim().is_empty() => println!("        data:  {data}"),
                _ => println!("        data:  {data:?}"),
            }
        }
    }

    Ok(())
}
