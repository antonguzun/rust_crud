use actix_http::Request;
use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{http::header, test, web, App};
use actix_web_httpauth::middleware::HttpAuthentication;

use authust::apps::{init_api_v1, init_external_v1, init_internal_v1, init_system};
use authust::common::{Config, Resources};
use authust::middlewares::bearer_validator;
use authust::usecases::user::crypto::generate_jwt;

use std::fs;
use std::str::FromStr;

#[path = "./constants.rs"]
mod constants;

const USERS_FIXTURE: &str = "INSERT INTO users 
    (user_id, username, password_hash, enabled, created_at, updated_at, is_deleted)
    VALUES 
    (1, 'Ivan', '1234', TRUE, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (2, 'test_user', '$argon2i$v=19$m=4096,t=3,p=1$MjJmNjVlNzktNDk2YS00YjQ4LThhYmMtZjgzZTFlNTJhYTRl$GrBGOuJ9PznSgBOp0e5sdkMf2KAfgnubSh37Oq0HAzw', TRUE, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (3, 'Godzilla', '1234', TRUE, now(), now(), FALSE)";
const PERMISSIONS_FIXTURE: &str =
    "INSERT INTO permissions (permission_id, permission_name, created_at, updated_at, is_deleted)
    VALUES 
    (1, 'PERM_1', '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (2, 'PERM_2', '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (3, 'PERM_3', now(), now(), TRUE)";
const GROUPS_FUXTURE: &str =
    "INSERT INTO groups (group_id, group_name, created_at, updated_at, is_deleted)
    VALUES 
    (1, 'GROUP_1', '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (2, 'GROUP_2', '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (3, 'GROUP_3', now(), now(), TRUE),
    (4, 'ROLE_AUTH_ADMIN', now(), now(), FALSE),
    (5, 'ROLE_AUTH_MANAGER', now(), now(), FALSE),
    (6, 'ROLE_AUTH_STAFF', now(), now(), FALSE)";
const GROUPS_PERMISSIONS_BINDING_FUXTURE: &str =
    "INSERT INTO group_permissions (permission_id, group_id, created_at, updated_at, is_deleted)
    VALUES 
    (1, 1, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (2, 1, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (3, 1, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (1, 2, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (2, 2, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (3, 2, now(), now(), TRUE),
    (3, 3, now(), now(), FALSE)";
const GROUPS_MEMBERS_BINDING_FUXTURE: &str =
    "INSERT INTO group_members (user_id, group_id, created_at, updated_at, is_deleted)
    VALUES 
    (1, 1, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (2, 1, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (3, 1, '2016-06-22 22:10:25+03', '2016-06-22 22:10:25+03', FALSE), 
    (1, 2, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (2, 2, '2022-06-22 22:10:25+00', '2022-06-22 22:10:25+00', FALSE), 
    (2, 4, now(), now(), FALSE), 
    (3, 2, now(), now(), TRUE),
    (3, 3, now(), now(), FALSE)";

pub enum IntenalRoles {
    RoleAdmin,
    RoleManager,
    RoleStaff,
}

async fn refresh_db(resources: &Resources) -> () {
    let client = resources.db_pool.get().await.unwrap();

    let migration_paths = fs::read_dir("./tests/migrations").unwrap();
    for path in migration_paths {
        let filename = path.unwrap().path().display().to_string();
        let query = &fs::read_to_string(&filename).unwrap();
        client.batch_execute(query).await.unwrap();
    }
    client
        .simple_query("TRUNCATE TABLE users, permissions, groups, group_permissions CASCADE")
        .await
        .unwrap();
    let fixtures_queries = [
        USERS_FIXTURE,
        PERMISSIONS_FIXTURE,
        GROUPS_FUXTURE,
        GROUPS_MEMBERS_BINDING_FUXTURE,
        GROUPS_PERMISSIONS_BINDING_FUXTURE,
    ];
    for query in fixtures_queries.into_iter() {
        client.simple_query(query).await.unwrap();
    }
}

pub async fn init_test_service(
) -> impl Service<Request, Response = ServiceResponse<BoxBody>, Error = actix_web::Error> {
    let config = Config::create_config();
    let resources = Resources::create_resources(&config).await;
    refresh_db(&resources).await;
    let auth = HttpAuthentication::bearer(bearer_validator);
    test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .data(resources.clone())
            .service(web::scope("api/v1").configure(init_api_v1).wrap(auth))
            .service(web::scope("srv/v1").configure(init_internal_v1))
            .service(web::scope("auth/v1").configure(init_external_v1))
            .service(web::scope("").configure(init_system)),
    )
    .await
}

use actix_web::http::header::{HeaderName, HeaderValue};
#[allow(dead_code)]
pub fn create_test_jwt() -> String {
    let config = Config::create_config().security_config;
    let jwt = generate_jwt(&config, constants::TEST_USER_ID, vec!["fake".to_string()])
        .expect("can not create jwt for tests");
    jwt
}

fn create_bearer_header<'a>(role: IntenalRoles) -> (HeaderName, HeaderValue) {
    let config = Config::create_config().security_config;
    let roles = match role {
        IntenalRoles::RoleAdmin => vec!["ROLE_AUTH_ADMIN".to_string()],
        IntenalRoles::RoleManager => vec!["ROLE_AUTH_MANAGER".to_string()],
        IntenalRoles::RoleStaff => vec!["ROLE_AUTH_STAFF".to_string()],
    };
    let jwt = generate_jwt(&config, constants::TEST_USER_ID, roles)
        .expect("can not create jwt for tests");
    let token = format!("Bearer {}", jwt);
    (
        HeaderName::from_str("Authorization").unwrap(),
        HeaderValue::from_str(&token).unwrap(),
    )
}

pub fn test_post(url: &str, role: IntenalRoles) -> test::TestRequest {
    test::TestRequest::post()
        .insert_header(header::ContentType::json())
        .insert_header(create_bearer_header(role))
        .uri(url)
}

pub fn test_get(url: &str, role: IntenalRoles) -> test::TestRequest {
    test::TestRequest::get()
        .insert_header(create_bearer_header(role))
        .uri(url)
}

pub fn test_delete(url: &str, role: IntenalRoles) -> test::TestRequest {
    test::TestRequest::delete()
        .insert_header(create_bearer_header(role))
        .uri(url)
}

#[allow(dead_code)]
pub fn test_put(url: &str, role: IntenalRoles) -> test::TestRequest {
    test::TestRequest::put()
        .insert_header(header::ContentType::json())
        .insert_header(create_bearer_header(role))
        .uri(url)
}
