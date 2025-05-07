use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::Filter;

#[derive(Clone, Debug, Serialize)]
pub struct Model {
    id: String,
    name: String,
    version: String,
    data: Vec<u8>,
    create_time: i64,
}

#[derive(Clone)]
pub struct ModelStore {
    models: HashMap<String, Model>,
}

pub fn new_model_store() -> ModelStore {
    ModelStore {
        models: HashMap::new(),
    }
}

impl ModelStore {
    pub fn add_model(&mut self, name: String, version: String, data: String) {
        let id: Uuid = Uuid::new_v4();
        let now: DateTime<Local> = Local::now();
        self.models.insert(
            id.to_string(),
            Model {
                id: id.to_string(),
                name: name,
                version: version,
                data: data.into_bytes(),
                create_time: now.timestamp_millis(),
            },
        );
    }

    pub fn delete_model(&mut self, id: String) {
        self.models.remove(&id);
    }

    pub fn get_models(&self) -> HashMap<String, Model> {
        self.models.clone()
    }
}

fn with_model_store(
    model_store: Arc<RwLock<ModelStore>>,
) -> impl Filter<Extract = (Arc<RwLock<ModelStore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || model_store.clone())
}

#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub version: String,
    pub data: String,
}

async fn create_model_handler(
    req: CreateModelRequest,
    model_store: Arc<RwLock<ModelStore>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut store = model_store.write().await;
    store.add_model(req.name, req.version, req.data);
    Ok(warp::reply::json(&"create success"))
}

fn route_create_model(
    model_store: Arc<RwLock<ModelStore>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("model")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_model_store(model_store))
        .and_then(create_model_handler)
}

async fn get_model_handler(
    model_store: Arc<RwLock<ModelStore>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut store = model_store.read().await;
    Ok(warp::reply::json(&store.get_models()))
}

fn route_get_models(
    model_store: Arc<RwLock<ModelStore>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("model")
        .and(warp::get())
        .and(with_model_store(model_store))
        .and_then(get_model_handler)
}

pub fn routes(
    model_store: Arc<RwLock<ModelStore>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    route_get_models(model_store.clone()).or(route_create_model(model_store.clone()))
}
