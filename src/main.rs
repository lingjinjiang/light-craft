use std::sync::Arc;
use tokio::sync::RwLock;
mod model;

#[tokio::main]
async fn main() {
    match model::new_model_store() {
        Ok(model_store) => {
            let model_store = Arc::new(RwLock::new(model_store));
            let routes = model::routes(model_store);
            warp::serve(routes).run(([127, 0, 0, 1], 3000)).await
        }
        Err(_) => {}
    }
}
