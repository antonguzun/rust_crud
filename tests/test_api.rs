use actix_http::Request;
use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{http::header, test, web, App};
use rust_crud::apps::init_api_v1;
use rust_crud::common::{Config, Resources};
use rust_crud::usecases::user::entities::{SingnedInfo, User};
use serde_json::json;
use std::fs;

mod constants;

async fn refresh_db(resources: &Resources) -> () {
    let client = resources.db_pool.get().await.unwrap();

    let migration_paths = fs::read_dir("./tests/migrations").unwrap();

    for path in migration_paths {
        let filename = path.unwrap().path().display().to_string();
        let query = &fs::read_to_string(&filename).unwrap();
        client.query(query, &[]).await.unwrap();
    }

    client.query("TRUNCATE TABLE users;", &[]).await.unwrap();
    client
        .query(
            "INSERT INTO users 
        (user_id, username, password_hash, enabled, created_at, updated_at, is_deleted)
        VALUES 
        (1, 'Ivan', '1234', TRUE, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
        (2, 'Anton', $1, TRUE, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
        (3, 'Godzilla', '1234', TRUE, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE)
        ON CONFLICT DO NOTHING;",
            &[&constants::TEST_PASSWORD_HASH],
        )
        .await
        .unwrap();
}

async fn init_test_service(
) -> impl Service<Request, Response = ServiceResponse<BoxBody>, Error = actix_web::Error> {
    let config = Config::create_config();
    let resources = Resources::create_resources(&config).await;
    refresh_db(&resources).await;
    test::init_service(
        App::new()
            .app_data(config.clone())
            .data(resources.clone())
            .service(web::scope("/api/v1").configure(init_api_v1)),
    )
    .await
}

#[actix_web::test]
async fn test_get_user() {
    let mut app = init_test_service().await;
    let req = test::TestRequest::get().uri("/api/v1/user/1").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200)
}

#[actix_web::test]
async fn test_get_user_not_found() {
    let mut app = init_test_service().await;
    let req = test::TestRequest::get()
        .uri("/api/v1/user/999991")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_get_user_wrong_params() {
    let mut app = init_test_service().await;
    let req = test::TestRequest::get()
        .uri("/api/v1/user/sadf")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    // странно что web::Path приводит к 404 ошибке, а не к 400
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_delete_user() {
    let mut app = init_test_service().await;

    let req = test::TestRequest::get().uri("/api/v1/user/3").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::delete()
        .uri("/api/v1/user/3")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 204);

    let req = test::TestRequest::get().uri("/api/v1/user/3").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_delete_user_what_doesnt_exist() {
    let mut app = init_test_service().await;
    let req = test::TestRequest::delete()
        .uri("/api/v1/user/999")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 204)
}

#[actix_web::test]
async fn test_create_new_user() {
    let mut app = init_test_service().await;
    let request_body = json!({
        "username": "tester",
        "password": "test",
    });
    let req = test::TestRequest::post()
        .insert_header(header::ContentType::json())
        .uri("/api/v1/user/")
        .set_json(request_body)
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    let status = resp.status();
    let user: User = test::read_body_json(resp).await;
    assert_eq!(user.username, "tester");
    assert_eq!(status, 201)
}

#[actix_web::test]
async fn test_sign_in_forbidden() {
    let mut app = init_test_service().await;
    let request_body = json!({
        "username": "keker",
        "password": "wrong_passord",
    });
    let req = test::TestRequest::post()
        .insert_header(header::ContentType::json())
        .uri("/api/v1/user/sign_in")
        .set_json(request_body)
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 403)
}

#[actix_web::test]
async fn test_sign_in() {
    let mut app = init_test_service().await;
    let request_body = json!({
        "username": "Anton",
        "password": constants::TEST_PASSWORD,
    });
    let req = test::TestRequest::post()
        .insert_header(header::ContentType::json())
        .uri("/api/v1/user/sign_in")
        .set_json(request_body)
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    let status = resp.status();
    let signed_info: SingnedInfo = test::read_body_json(resp).await;
    assert_eq!(status, 200);
    assert_eq!(signed_info.jwt_token, "test_token");
    assert_eq!(signed_info.user_id, 2);
}
