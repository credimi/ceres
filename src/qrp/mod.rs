use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub mod cerved_qrp;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QrpFormat {
    Pdf,
    Xml,
}

impl QrpFormat {
    pub fn value(&self) -> String {
        match &self {
            QrpFormat::Pdf => "pdf".to_owned(),
            QrpFormat::Xml => "xml".to_owned(),
        }
    }
}

impl Display for QrpFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QrpProduct {
    Qrp = 62001,
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

#[derive(Serialize, Debug)]
pub struct QrpRequest {
    pub(crate) format: QrpFormat,
    pub(crate) product_id: QrpProduct,
    pub(crate) reference: Uuid,
    pub(crate) subject_type: SubjectType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) vat_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tax_code: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    #[test]
    fn test_qrp_format() {
        let pdf = QrpFormat::Pdf;
        assert_eq!(pdf.value(), "pdf");

        let xml = QrpFormat::Xml;
        assert_eq!(xml.value(), "xml");
    }

    #[test]
    fn test_qrp_product() {
        let qrp = QrpProduct::Qrp;
        assert_eq!(qrp.value(), "62001");
    }

    #[test]
    fn test_qrp_request() {
        let request = QrpRequest {
            format: QrpFormat::Pdf,
            product_id: QrpProduct::Qrp,
            reference: uuid!("01912698-474d-7a13-b5b8-103bd86b7a44"),
            subject_type: SubjectType::CompanyAndNorea,
            vat_number: Some("12345678901".to_owned()),
            tax_code: None,
        };
        let expected_json = r#"{"format":"PDF","product_id":"62001","reference":"01912698-474d-7a13-b5b8-103bd86b7a44","subject_type":"COMPANY_AND_NOREA","vat_number":"12345678901"}"#;
        let actual_json = serde_json::to_string(&request).unwrap();

        assert_eq!(expected_json, actual_json);
    }
}
