use std::io::Result as IoResult;

use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpResponse};
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::utils::logging::Logger;

#[derive(Debug, Parser)]
pub struct Cli {

}

pub struct AppConfig {
    pub log: Logger,
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

