use crate::qrp::QrpFormat;
use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::primitives::ByteStream;
use chrono::Utc;
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug, Clone)]
pub struct AwsConf {
    #[clap(long, env)]
    pub qrp_bucket_name: String,
    #[clap(long, env, default_value = "false")]
    pub s3_dry_run: bool,
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
        data: &[u8],
        vat_number: &String,
        user: &String,
        format: &QrpFormat,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        let date_time = now.format("%d_%m_%Y_%H:%M:%S");
        let lower_case_format = format.as_str();
        let file = format!("qrp/{vat_number}/{date_time}_{user}.{lower_case_format}");

        if self.aws_conf.s3_dry_run {
            info!("Dry run: not uploading to S3");
            return Ok(());
        }

        Ok(self
            .client
            .put_object()
            .bucket(&self.aws_conf.qrp_bucket_name)
            .key(&file)
            .set_body(Option::from(ByteStream::from(data.to_owned())))
            .send()
            .await?)
        .map(|_res| {
            info!("Uploaded to S3: {}", file);
        })
    }
}
