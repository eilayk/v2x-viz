use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct TrackedObjects<T> {
    objects: HashMap<u32, TrackedObject<T>>,
}

struct TrackedObject<T> {
    pub item: T,
    pub last_seen: Instant,
}

impl<T> TrackedObjects<T> {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u32, item: T) {
        self.objects.insert(
            id,
            TrackedObject {
                item,
                last_seen: Instant::now(),
            },
        );
    }

    pub fn clean_expired(&mut self, timeout: Duration) {
        let now = Instant::now();
        self.objects.retain(|_, obj| now.duration_since(obj.last_seen) <= timeout);
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.objects.values().map(|obj| &obj.item)
    }
}
