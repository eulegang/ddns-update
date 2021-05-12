use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct DomainList {
    pub domain_records: Vec<DomainRecord>,
}

#[derive(Deserialize, Debug)]
pub struct DomainRecord {
    #[serde(rename = "type")]
    pub ty: String,

    pub id: u64,
    pub name: String,
    pub data: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadRecord {
    #[serde(rename = "type")]
    pub ty: String,

    pub name: String,
    pub data: String,
}
