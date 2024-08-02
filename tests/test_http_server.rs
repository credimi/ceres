mod tests {
    use actix_http::Request;
    use actix_service::Service;
    use actix_web::dev::ServiceResponse;
    use actix_web::test::init_service;
    use actix_web::{test, web, App, Error};
    use clap::Parser;
    use diesel::RunQueryDsl;
    use rust_microservice::database::schema::flags;
    use rust_microservice::database::{Db, DbConfig};
    use rust_microservice::routes::{get_flag, insert_flag, AppConfig, FlagRequest, FlagResponse, NewFlagRequest};
    use rust_microservice::utils::logging::get_root_logger;

    fn clean_database(db: &Db) {
        diesel::delete(flags::table)
            .execute(&mut db.connection.get().unwrap())
            .unwrap();
    }

    fn init_database() -> Db {
        DbConfig::parse().connect().unwrap()
    }

    async fn init_server(db: Db) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
        init_service(
            App::new()
                .app_data(web::Data::new(AppConfig {
                    db,
                    log: get_root_logger(),
                }))
                .service(insert_flag)
                .service(get_flag),
        )
        .await
    }

    #[actix_rt::test]
    async fn post_and_get() {
        let db = init_database();
        clean_database(&db);
        let mut app = init_server(db).await;

        let req1 = test::TestRequest::post()
            .uri("/api/v1/flag")
            .set_json(&NewFlagRequest { flag: true })
            .to_request();

        let resp1: FlagResponse = test::call_and_read_body_json(&mut app, req1).await;

        let req2 = test::TestRequest::get()
            .uri("/api/v1/flag")
            .set_json(&FlagRequest { id: resp1.id })
            .to_request();

        let resp2: FlagResponse = test::call_and_read_body_json(&mut app, req2).await;

        assert!(resp1 == resp2);
    }
}
