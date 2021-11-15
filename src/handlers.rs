use crate::common::Resources;
use crate::services::{get_entity_by_id, Entity};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use log::error;
use serde::{Deserialize, Serialize};
use web::Data;

#[derive(Serialize, Deserialize)]
pub struct EntityCreate {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct EntityQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[post("/entity")]
pub async fn create_entity(
    entity: web::Json<EntityCreate>,
    resources: Data<Resources>,
) -> impl Responder {
    let client = resources.db_pool.get().await.unwrap();
    let stmt = client
        .prepare("INSERT INTO entities (name) VALUES ($1) RETURNING entity_id")
        .await
        .unwrap();
    let rows = client.query(&stmt, &[&entity.name]).await.unwrap();
    let entity_id: i32 = rows[0].get(0);
    HttpResponse::Created().json(Entity::new(entity_id, entity.name.clone()))
}

#[delete("/entity/{entity_id}")]
pub async fn delete_entity(
    entity_id: web::Path<u32>,
    resources: Data<Resources>,
) -> impl Responder {
    let entity_id = entity_id.into_inner() as i32;
    let client = resources.db_pool.get().await.unwrap();
    let stmt = client
        .prepare("DELETE FROM entities where entity_id=$1")
        .await
        .unwrap();
    client.execute(&stmt, &[&entity_id]).await;
    HttpResponse::NoContent().body("success")
}

#[get("/entity/{entity_id}")]
pub async fn get_entity(entity_id: web::Path<u32>, resources: Data<Resources>) -> impl Responder {
    let entity_id = entity_id.into_inner() as i32;
    let result = get_entity_by_id(resources.db_pool.clone(), entity_id).await;
    match result {
        Ok(data) => match data {
            Some(data) => HttpResponse::Ok().json(data),
            None => HttpResponse::NotFound().body("Not Found"),
        },
        Err(e) => {
            error!("{}", e);
            HttpResponse::InternalServerError().body("internal error")
        }
    }
}

#[get("/entity")]
pub async fn get_entities(
    q: web::Query<EntityQuery>,
    resources: Data<Resources>,
) -> impl Responder {
    let offset = q.offset.unwrap_or(0);
    let limit = q.limit.unwrap_or(100);
    let client = resources.db_pool.get().await.unwrap();
    let stmt = client
        .prepare("SELECT entity_id, name FROM entities LIMIT $1 OFFSET $2")
        .await
        .unwrap();
    let rows = client.query(&stmt, &[&limit, &offset]).await.unwrap();
    let entities: Vec<Entity> = rows
        .iter()
        .map(|row| Entity::new(row.get(0), row.get(1)))
        .collect();
    HttpResponse::Ok().json(entities)
}
