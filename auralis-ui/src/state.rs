use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use auralis_core::Orb;

#[derive(Clone)]
pub struct AppState {
    pub orbs: HashMap<Uuid, Orb>,
    pub dragged_orb_id: Option<Uuid>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            orbs: HashMap::new(),
            dragged_orb_id: None,
        }
    }
}

pub type SharedState = Rc<RefCell<AppState>>;
