use std::{collections::HashMap, time::Duration, sync::{Arc}, ops::Deref};

// I want to synchronize access to cache items for serveral tasks,
// so it seems like I should not be using std::sync
use tokio::{task::JoinHandle, time, sync::{Mutex, RwLock}};
// use anyhow::anyhow;

#[derive(Clone)]
pub struct Cache(Arc<InnerCache>);

impl Deref for Cache {
    type Target = Arc<InnerCache>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//  just for lols, I would be doing this with actors and messaging via channels in real life,
//  because I do not want to deal with locks and related issues EVER
pub struct InnerCache {
  items: RwLock<HashMap<String, String>>,
  refreshers: Mutex<HashMap<String, Refresher>>,
}

pub struct Refresher {
    key: String,
    ttl: u64,
    interval: u64,
    handler: JoinHandle<()>
}

impl Cache {
    pub fn new() -> Self {
        Self(
            Arc::new(
                InnerCache {
                    items: Default::default(),
                    refreshers: Default::default()
                }
            )
        )
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let guard = self.items.read().await;
        guard.get(key).map(|item| item.to_owned())
    }

    pub async fn register<F>(&mut self, mut fun: F, key: &str, ttl: u64, refresh_interval: u64) -> anyhow::Result<()>
       where F: FnMut() -> String + Send + 'static
    {

        let mut interval = time::interval(Duration::from_secs(refresh_interval));

        let moved_key = key.to_owned();
        let cache = self.clone();
        let task = tokio::spawn(async move {
            println!("Registering task with key: {}", moved_key);

            loop {
                interval.tick().await;
                let result = fun();
                {
                    let mut guard = cache.items.write().await;
                    println!("Inserting an updated value '{}' for '{}' key", result, moved_key);
                    guard.insert(moved_key.clone(), result);
                }
            }
        });

        let refresher = Refresher { key: key.to_owned(), ttl, interval: refresh_interval, handler: task };

        let mut guard = self.refreshers.lock().await;
        guard.insert(key.to_owned(), refresher);

        Ok(())
    }
}