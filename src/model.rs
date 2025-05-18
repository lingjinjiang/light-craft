use chrono::{DateTime, Local};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::Filter;

#[derive(Clone, Debug, Serialize)]
pub struct Model {
    id: String,
    name: String,
    version: String,
    data: String,
    create_time: i64,
}

#[derive(Clone)]
pub struct ModelStore {
    conn: Arc<Mutex<Connection>>,
}

const CREATE_MODEL_TABLE: &str = "CREATE TABLE IF NOT EXISTS models (id TEXT PRIMARY KEY, name TEXT, version TEXT, data BLOB, create_time INTEGER)";
const DELETE_BY_ID: &str = "DELETE FROM models WHERE id=:id";
const INSERT_MODEL: &str = "INSERT INTO models (id, name, version, data, create_time) VALUES (:id, :name, :version, :data, :create_time)";
const SELECT_ALL: &str = "SELECT * FROM models";

pub fn new_model_store() -> Result<ModelStore, rusqlite::Error> {
    match Connection::open("data.db") {
        Ok(conn) => match conn.execute(CREATE_MODEL_TABLE, ()) {
            Ok(_) => Ok(ModelStore {
                conn: Arc::new(Mutex::new(conn)),
            }),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

impl ModelStore {
    pub fn add_model(&mut self, name: String, version: String, data: String) {
        let id: Uuid = Uuid::new_v4();
        let now: DateTime<Local> = Local::now();
        let conn = self.conn.lock().unwrap();
        match conn.execute(
            INSERT_MODEL,
            &[
                (":id", id.to_string().as_str()),
                (":name", name.as_str()),
                (":version", version.as_str()),
                (":data", data.as_str()),
                (":create_time", &(now.timestamp_millis().to_string())),
            ],
        ) {
            Ok(updated) => println!("{} rows were updated", updated),
            Err(e) => println!("insert error, err={:?}", e),
        }
    }

    pub fn delete_model(&mut self, id: String) {
        let conn = self.conn.lock().unwrap();
        match conn.execute(DELETE_BY_ID, &[(":id", &id)]) {
            Ok(deleted) => println!("{} rows were deleted", deleted),
            Err(e) => println!("delete error, err={:?}", e),
        }
    }

    pub fn get_models(&self) -> Vec<Model> {
        let conn = self.conn.lock().unwrap();
        let mut models: Vec<Model> = vec![];
        match conn.prepare(SELECT_ALL) {
            Ok(mut stmt) => match stmt.query_map([], |row| {
                Ok(Model {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    data: row.get(3)?,
                    create_time: row.get(4)?,
                })
            }) {
                Ok(model_iter) => {
                    for model in model_iter {
                        models.push(model.unwrap());
                    }
                }

                Err(_) => {}
            },
            Err(_) => {}
        }
        models
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
    let store = model_store.read().await;
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

#[derive(Debug, Deserialize)]
pub struct DeleteModelRequest {
    pub id: String,
}

async fn delete_model_handler(
    req: DeleteModelRequest,
    model_store: Arc<RwLock<ModelStore>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut store = model_store.write().await;
    store.delete_model(req.id);
    Ok(warp::reply::json(&"delete success"))
}

fn route_delete_model(
    model_store: Arc<RwLock<ModelStore>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("model")
        .and(warp::delete())
        .and(warp::body::json())
        .and(with_model_store(model_store))
        .and_then(delete_model_handler)
}

pub fn routes(
    model_store: Arc<RwLock<ModelStore>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    route_get_models(model_store.clone())
        .or(route_create_model(model_store.clone()))
        .or(route_delete_model(model_store.clone()))
}
