use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueElement<T: Clone + Serialize> {
    id: String,
    data: T,
    next: Option<String>
}


impl<T: Clone + Serialize> QueueElement<T> {
    pub fn new(data: T) -> Self {
        QueueElement {
            id: uuid::Uuid::new_v4().to_string(),
            data,
            next: None
        }
    }
    
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    
    pub fn get_data(&self) -> T {
        self.data.clone()
    }
    
    pub fn set_next(&mut self, next: Option<String>) {
        self.next = next;
    }
    
    pub fn get_next(&self) -> Option<String> {
        self.next.clone()
    }
}