use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub mod cerved_qrp;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QrpFormat {
    PDF,
    XML,
}

impl Display for QrpFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum QrpProduct {
    QRP,
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
            QrpProduct::QRP => "62001".to_owned(),
        }
    }
}

#[derive(Serialize, Debug)]
pub enum SubjectType {
    COMPANY,
    COMPANY_AND_NOREA,
    PERSON,
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

#[derive(Serialize, Deserialize)]
enum DeliveryStatus {
    OK,
    DEFERRED,
}

#[derive(Serialize, Deserialize)]
pub struct QrpResponse {
    pub content: Option<String>,
    delivery_status: DeliveryStatus,
    format: QrpFormat,
    pub request_id: u32,
}
