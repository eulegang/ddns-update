use eyre::ContextCompat;
use reqwest::blocking::get;
use std::net::IpAddr;
use structopt::StructOpt;

mod digital_ocean;

pub trait RecordUpdater {
    fn update(&self, ip: IpAddr, domain: &Domain<'_>) -> eyre::Result<()>;
}

#[derive(StructOpt)]
struct Cli {
    url: String,

    #[structopt(short, long, env = "TOKEN")]
    token: String,

    #[structopt(short, long)]
    domain: String,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::from_args();

    let domain = Domain::new(&cli.domain).wrap_err("cannot work with tld")?;

    let content = get(&cli.url)?.error_for_status()?.text()?;

    let addr: IpAddr = content.parse()?;

    let updater = digital_ocean::DigitalOceanRecordUpdater::new(&cli.token)?;
    updater.update(addr, &domain)?;

    println!("setting {} to {}", cli.domain, addr);

    Ok(())
}

#[derive(Debug)]
pub struct Domain<'a> {
    pub name: &'a str,
    pub root: &'a str,
    pub full: &'a str,
}

impl Domain<'_> {
    fn new(repr: &str) -> Option<Domain> {
        let pos = repr.find('.')?;

        Some(Domain {
            name: &repr[..pos],
            root: &repr[pos + 1..],
            full: repr,
        })
    }
}
