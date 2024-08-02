use std::io::Result as IoResult;

use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpResponse};
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::database::{Db, DbConfig};
use crate::kafka::KafkaConfig;
use crate::utils::logging::Logger;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub db_config: DbConfig,
    #[command(flatten)]
    pub kafka_config: KafkaConfig,
}

pub struct AppConfig {
    pub db: Db,
    pub log: Logger,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FlagRequest {
    pub id: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NewFlagRequest {
    pub flag: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FlagResponse {
    pub id: i32,
    pub creation_timestamp: DateTime<Utc>,
    pub update_timestamp: DateTime<Utc>,
    pub flag: bool,
}

#[derive(Serialize)]
pub enum ErrorKind {
    DatabaseError,
}

#[derive(Serialize)]
pub struct InternalError {
    pub kind: ErrorKind,
    pub reason: String,
}

#[get("/api/v1/flag")]
async fn get_flag(app_config: Data<AppConfig>, req: Json<FlagRequest>) -> IoResult<HttpResponse> {
    let res = app_config.db.get_flag(req.id);

    match res {
        Ok(flag) => Ok(HttpResponse::Ok().json(FlagResponse {
            id: flag.id,
            creation_timestamp: flag.creation_timestamp,
            update_timestamp: flag.update_timestamp,
            flag: flag.flag,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(InternalError {
            kind: ErrorKind::DatabaseError,
            reason: format!("{}", e),
        })),
    }
}

#[post("/api/v1/flag")]
pub async fn insert_flag(app_config: Data<AppConfig>, req: Json<NewFlagRequest>) -> IoResult<HttpResponse> {
    let res = app_config.db.create_flag(&req.flag);

    match res {
        Ok(flag) => Ok(HttpResponse::Ok().json(FlagResponse {
            id: flag.id,
            creation_timestamp: flag.creation_timestamp,
            update_timestamp: flag.update_timestamp,
            flag: flag.flag,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(InternalError {
            kind: ErrorKind::DatabaseError,
            reason: format!("{}", e),
        })),
    }
}
