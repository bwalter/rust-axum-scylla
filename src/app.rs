use axum::routing::BoxRoute;
use axum::Router;
use std::sync::{Arc, RwLock};

use crate::db::queries::Queries;
use crate::routing;
use crate::state::State;

pub struct App<Q: Queries> {
    pub router: Router<BoxRoute>,

    #[allow(dead_code)]
    shared_state: Arc<RwLock<State>>,

    #[allow(dead_code)]
    queries: Arc<Q>,
}

impl<Q: Queries> App<Q> {
    pub fn new(queries: Arc<Q>) -> Self {
        // Shared state
        let shared_state = Arc::new(RwLock::new(State {}));

        App {
            router: routing::create_router(shared_state.clone(), queries.clone()),
            shared_state,
            queries,
        }
    }
}
