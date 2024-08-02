//! Connection to a Kafka queue. Just pass your handler to `KafkaConfig::consume`

use std::future::Future;

use crate::utils::logging::get_root_logger;
use anyhow::{anyhow, Context};
use clap::Parser;
use futures_util::stream::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::producer::{BaseProducer, BaseRecord};
use rdkafka::{ClientConfig, Message};
use serde::{Deserialize, Serialize};
use slog::{debug, error, info, trace};

#[derive(Parser, Debug, Clone)]
pub struct KafkaConfig {
    #[arg(long, env)]
    pub topic: String,

    #[arg(long, env, default_value = "template")]
    pub group_id: String,

    /// Comma-separated list of Kafka brokers
    #[arg(long, env)]
    pub brokers: String,

    /// Comma-separated list of key=value pairs for additional Kafka options
    #[arg(long, env)]
    pub config: Option<String>,
}

impl KafkaConfig {
    fn config(&self) -> Vec<(String, String)> {
        self.config
            .as_ref()
            .map(|c| c.split(",").map(str::to_string).collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
            .map(|kv| kv.split("=").map(str::to_string).collect::<Vec<_>>().into_iter())
            .map(|mut kv| (kv.nth(0).unwrap(), kv.nth(0).unwrap()))
            .collect()
    }

    /// Loop over messages received from Kafka, deserialize their body into `P`, and
    /// pass it to the async function `consume_fn`.
    ///
    /// This assumes that the message body is valid Json.
    ///
    /// This method will return an error if:
    /// * it cannot connect to kafka or retrieve the message, or
    /// * it fails to deserialize the message into the payload type `P`, or
    /// * `consume_fn` returns an error.
    ///
    /// In all other circumstances it will run forever.
    pub async fn consume<P, F>(self, mut consume_fn: impl FnMut(P) -> F) -> anyhow::Result<()>
    where
        P: for<'a> Deserialize<'a> + 'static,
        F: Future<Output = anyhow::Result<()>>,
    {
        let log = get_root_logger();
        info!(log, "Starting up consumer");

        let consumer: StreamConsumer = {
            let mut client = ClientConfig::new();

            client.set("bootstrap.servers", &self.brokers);
            client.set("group.id", &self.group_id);
            client.set("enable.auto.commit", "false");
            for (k, v) in self.config() {
                client.set(k, v);
            }

            debug!(log, "effective kafka configuration: {:?}", &client.config_map());
            client.create()?
        };

        debug!(log, "Connecting to Kafka");
        consumer.subscribe(&[&self.topic])?;

        let mut stream = consumer.stream();

        info!(log, "Consuming from topic {}", &self.topic);
        while let Some(message) = stream.next().await {
            let message = message?;
            trace!(log, "Message received offset={}", message.offset());
            let context = || format!("at offset {}", message.offset());
            let body = message.payload().ok_or(anyhow!("empty body")).with_context(context)?;
            let payload: P = serde_json::from_slice(&body)?;
            consume_fn(payload).await.with_context(context)?;
            consumer.commit_message(&message, CommitMode::Async)?;
        }

        Ok(())
    }
}

pub struct KafkaProducer {
    client: BaseProducer,
    log: slog::Logger,
    pub topic: String,
}

impl KafkaProducer {
    pub async fn new(config: KafkaConfig) -> anyhow::Result<KafkaProducer> {
        let log = get_root_logger();
        let client: BaseProducer = {
            let mut client = ClientConfig::new();
            client.set("bootstrap.servers", &config.brokers);
            debug!(log, "effective kafka configuration: {:?}", &client.config_map());
            client.create()?
        };
        Ok(KafkaProducer {
            client,
            topic: config.topic,
            log,
        })
    }

    pub fn produce<M: Serialize>(&self, key: String, payload: M) -> anyhow::Result<()> {
        let payload = serde_json::to_string(&payload)?;
        let record = BaseRecord::to(&self.topic).key(&key).payload(&payload);
        self.client.send(record).map_err(|(err, message)| {
            error!(self.log, "{message:?}");
            err
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::KafkaConfig;

    #[test]
    fn parse_config() {
        let kafka_conf = KafkaConfig {
            config: Some("opt1=val1,opt2=val2".to_string()),
            topic: "".to_string(),
            group_id: "".to_string(),
            brokers: "".to_string(),
        };

        let config = kafka_conf.config();
        assert_eq!(config[0].0, "opt1");
        assert_eq!(config[0].1, "val1");
        assert_eq!(config[1].0, "opt2");
        assert_eq!(config[1].1, "val2");
    }
}
