use crate::handlers::api::groups::handlers::{
    bind_member_with_group_handler, bind_permission_with_group_handler, create_group_handler,
    disable_group_handler, get_group_handler, unbind_member_with_group_handler,
    unbind_permission_with_group_handler,
};
use crate::handlers::api::permissions::handlers::{
    create_permission_handler, disable_permission_handler, get_permission_handler,
    permissions_listing_handler,
};
use crate::handlers::api::users::{
    create_user_handler, delete_user_by_id, get_user_by_id, sign_in_user_handler,
    validate_jwt_handler,
};
use crate::handlers::system::handlers::{ping_handler, ready_handler};

use actix_web::web::ServiceConfig;

pub fn init_api_v1(cfg: &mut ServiceConfig) {
    cfg.service(get_user_by_id)
        .service(create_user_handler)
        .service(delete_user_by_id)
        .service(get_permission_handler)
        .service(create_permission_handler)
        .service(disable_permission_handler)
        .service(permissions_listing_handler)
        .service(get_group_handler)
        .service(create_group_handler)
        .service(disable_group_handler)
        .service(bind_permission_with_group_handler)
        .service(unbind_permission_with_group_handler)
        .service(bind_member_with_group_handler)
        .service(unbind_member_with_group_handler);
}
pub fn init_external_v1(cfg: &mut ServiceConfig) {
    cfg.service(sign_in_user_handler);
}

pub fn init_internal_v1(cfg: &mut ServiceConfig) {
    cfg.service(validate_jwt_handler);
}

pub fn init_system(cfg: &mut ServiceConfig) {
    cfg.service(ready_handler).service(ping_handler);
}
