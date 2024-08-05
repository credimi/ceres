use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub mod cerved_qrp;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QrpFormat {
    Pdf,
    Xml,
}

impl Display for QrpFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QrpProduct {
    Qrp,
}

impl Serialize for QrpProduct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.value().as_str())
    }
}

impl QrpProduct {
    fn value(&self) -> String {
        match &self {
            QrpProduct::Qrp => "62001".to_owned(),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubjectType {
    Company,
    CompanyAndNorea,
    Person,
}

#[derive(Serialize, Debug, TypedBuilder)]
pub struct QrpRequest {
    format: QrpFormat,
    product_id: QrpProduct,
    reference: Uuid,
    subject_type: SubjectType,
    vat_number: Option<String>,
    tax_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DeliveryStatus {
    Ok,
    Deferred,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QrpResponse {
    pub content: Option<String>,
    delivery_status: DeliveryStatus,
    format: QrpFormat,
    pub request_id: u32,
}
