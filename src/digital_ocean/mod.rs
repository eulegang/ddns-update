use super::{Domain, RecordUpdater};
use eyre::WrapErr;
use reqwest::blocking::Client;
use reqwest::header;
use std::net::IpAddr;

use payload::*;

mod payload;

pub struct DigitalOceanRecordUpdater {
    client: Client,
}

impl DigitalOceanRecordUpdater {
    pub fn new(token: &str) -> eyre::Result<DigitalOceanRecordUpdater> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let mut header = header::HeaderValue::from_str(&format!("Bearer {}", token))?;
        header.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, header);

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .wrap_err("failed to build http client")?;

        Ok(DigitalOceanRecordUpdater { client })
    }

    fn insert(&self, ip: IpAddr, domain: &Domain) -> eyre::Result<()> {
        let ty = dns_type(&ip).to_string();
        let data = ip.to_string();
        let name = domain.name.to_string();

        self.client
            .post(format!(
                "https://api.digitalocean.com/v2/domains/{}/records",
                domain.root,
            ))
            .json(&UploadRecord { ty, name, data })
            .send()?
            .error_for_status()?;

        Ok(())
    }

    fn update(&self, ip: IpAddr, domain: &Domain, record: &DomainRecord) -> eyre::Result<()> {
        if ip.to_string() == record.data {
            return Ok(());
        }

        let data = ip.to_string();
        let name = domain.name.to_string();
        let ty = dns_type(&ip).to_string();

        self.client
            .put(format!(
                "https://api.digitalocean.com/v2/domains/{}/records/{}",
                domain.root, record.id
            ))
            .json(&UploadRecord { ty, name, data })
            .send()?
            .error_for_status()?;

        Ok(())
    }

    fn purge(&self, domain: &Domain, records: &[DomainRecord]) -> eyre::Result<()> {
        for record in records {
            self.client
                .delete(format!(
                    "https://api.digitalocean.com/v2/domains/{}/records/{}",
                    domain.root, record.id
                ))
                .send()?
                .error_for_status()?;
        }

        Ok(())
    }
}

impl RecordUpdater for DigitalOceanRecordUpdater {
    fn update(&self, ip: IpAddr, domain: &Domain) -> eyre::Result<()> {
        let list: DomainList = self
            .client
            .get(format!(
                "https://api.digitalocean.com/v2/domains/{}/records?type={}&name={}",
                domain.root,
                dns_type(&ip),
                domain.full,
            ))
            .send()?
            .error_for_status()?
            .json()?;

        match &list.domain_records.as_slice() {
            [] => self.insert(ip, domain)?,
            [single] => self.update(ip, domain, single)?,
            [first, ..] => {
                self.update(ip, domain, first)?;
                self.purge(domain, &list.domain_records[1..])?;
            }
        }

        Ok(())
    }
}

fn dns_type(ip: &IpAddr) -> &'static str {
    match ip {
        IpAddr::V4(_) => "A",
        IpAddr::V6(_) => "AAAA",
    }
}
