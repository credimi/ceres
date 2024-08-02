use chrono::{DateTime, Utc};

#[derive(Queryable)]
pub struct Flag {
    pub id: i32,
    pub creation_timestamp: DateTime<Utc>,
    pub update_timestamp: DateTime<Utc>,
    pub flag: bool,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::flags)]
pub struct NewFlag {
    pub flag: bool,
}
