use base64::Engine;
use serde::{Deserialize, Serialize};
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
    pub fn as_str(&self) -> &str {
        match &self {
            QrpFormat::Pdf => "pdf",
            QrpFormat::Xml => "xml",
        }
    }
}

impl Display for QrpFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QrpProduct {
    #[serde(rename = "62001")]
    Qrp,
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

impl QrpResponse {
    pub fn decode_content(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(
                self.content
                    .as_ref()
                    .unwrap_or_else(|| panic!("No content found in response: {:?}", self)),
            )
            .expect("Invalid base64")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    #[test]
    fn test_qrp_format() {
        let pdf = QrpFormat::Pdf;
        assert_eq!(pdf.as_str(), "pdf");

        let xml = QrpFormat::Xml;
        assert_eq!(xml.as_str(), "xml");
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
    
    #[test]
    fn test_decode_content() {
        let response = QrpResponse {
            content: Some("SGVsbG8gV29ybGQh".to_owned()),
            delivery_status: DeliveryStatus::Ok,
            format: QrpFormat::Pdf,
            request_id: 1,
        };
        let content = response.decode_content();
        assert_eq!(content, b"Hello World!".to_vec());
    }
}
