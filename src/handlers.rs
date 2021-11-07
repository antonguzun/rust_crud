use crate::common::Config;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::error;

#[get("/")]
pub async fn hello(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    id: i32,
    name: String,
}

#[post("/echo_event")]
pub async fn echo_event(event: web::Json<Event>) -> impl Responder {
    HttpResponse::Ok().json(Event {
        id: event.id,
        name: event.name.clone(),
    })
}

pub async fn greetings(req: HttpRequest) -> impl Responder {
    let c: &Config = req.app_data().unwrap();
    HttpResponse::Ok().body(format!("it is {}", c.service_name))
}

#[post("/entity")]
pub async fn create_entity(event: web::Json<Event>) -> impl Responder {
    HttpResponse::Ok().json(Event {
        id: event.id,
        name: event.name.clone(),
    })
}

#[delete("/entity/{id}")]
pub async fn delete_entity(id: web::Path<usize>) -> impl Responder {
    error!(target: "entity", "delete entity with id={}", id);
    HttpResponse::Ok().body(format!("deleted {}", id))
}

#[get("/entity/{id}")]
pub async fn get_entity(id: web::Path<usize>) -> impl Responder {
    HttpResponse::Ok().body(format!("ok {}", id))
}

#[derive(Serialize, Deserialize)]
pub struct EntityQuery {
    text: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[get("/entity")]
pub async fn get_entities(q: web::Query<EntityQuery>) -> impl Responder {
    HttpResponse::Ok().body(format!("search by text {}", q.text))
}