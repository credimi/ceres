use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::operation::put_object::PutObjectOutput;
use aws_sdk_s3::primitives::ByteStream;
use chrono::Utc;
use clap::Parser;
use std::io::Bytes;

use crate::qrp::QrpFormat;

#[derive(Parser, Debug, Clone)]
pub struct AwsConf {
    #[clap(long, env)]
    pub aws_endpoint: Option<String>,
    #[clap(long, env)]
    pub qrp_bucket_name: String,
}

#[derive(Clone)]
pub struct S3Client {
    aws_conf: AwsConf,
    client: aws_sdk_s3::Client,
}

impl S3Client {
    pub async fn from_env(aws_conf: AwsConf) -> anyhow::Result<Self> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        Ok(S3Client {
            aws_conf,
            client: aws_sdk_s3::Client::new(&config),
        })
    }

    pub async fn upload(
        &self,
        data: &Vec<u8>,
        vat_number: &String,
        user: &String,
        format: QrpFormat,
    ) -> anyhow::Result<PutObjectOutput> {
        let now = Utc::now();
        let date_time = now.format("%d_%m_%Y_%T");
        let lower_case_format = format.to_string().to_ascii_lowercase();
        let file = format!("qrp/{vat_number}/{date_time}_{user}.{lower_case_format}");

        Ok(self
            .client
            .put_object()
            .bucket(&self.aws_conf.qrp_bucket_name)
            .key(file)
            .set_body(Option::from(ByteStream::from(data.clone())))
            .send()
            .await?)
    }
}
