use std::collections::HashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use telegram_bot::ChatId;
use tokio::io::AsyncReadExt;
use crate::{model::ServiceHandle, strategy::{Strategy, StrategyKind}, trader::{Trader, TraderConf}};
use crate::trader::entities as t;

use super::entities::{Context, Storage};

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedState<S> {
    token: String,
    strategies: HashMap<t::Key, S>,
}

impl <S: Strategy + Send + Clone + 'static> SavedState<S> {
    pub fn new(token: String, strategies: HashMap<t::Key, S>) -> Self {
        Self { token, strategies}
    }
    pub fn token(&self) -> String {
        self.token.clone()
    }
    pub fn strategies(&self) -> &HashMap<t::Key, S> {
        &self.strategies
    }
}

impl <S: Serialize + DeserializeOwned > SavedState<S> {
//TODO: это вообще здесь не нужно
    pub async fn save(&self, path: &str) -> Result<(), SaveError> {
        use tokio::io::AsyncWriteExt;
        let json = serde_json::to_string(&self)?;
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }

    pub async fn restore(path: &str) -> Result<HashMap<ChatId, SavedState<S>>, SaveError> {
        let mut dir = tokio::fs::read_dir(path).await?;
        let mut map = HashMap::new();
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().into_string().unwrap();
            let mut file = tokio::fs::File::open(entry.path()).await?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;
            let s = String::from_utf8(buf).unwrap();
            let state = serde_json::from_str(s.as_ref())?;
            let key = ChatId::new(name.parse().unwrap());
            map.insert(key, state);
        }
        Ok(map)
    }
}

#[derive(Debug)]
pub struct SaveError(String);

impl From<tokio::io::Error> for SaveError {
    fn from(e: tokio::io::Error) -> Self {
        Self(format!("{}", e))
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(e: serde_json::Error) -> Self {
        Self(format!("{}", e))
    }
}

#[cfg(test)]
mod test {
    use tokio::{fs::File, io::AsyncWriteExt};
    use crate::strategy::StrategyKind;

    use super::*;
    use serde_json;

    #[tokio::test]
    async fn test_fs() {
        let mut file = File::create("test.json").await.unwrap();
        let state = make_state();
        let json = serde_json::to_string(&state).unwrap();
        file.write_all(json.as_bytes()).await.unwrap();
    }

    fn make_state() -> SavedState<StrategyKind> {
        let mut strategies = HashMap::new();
        strategies.insert("test1".to_owned(), StrategyKind::FixedAmount(Default::default()));
        SavedState::new("token".to_owned(), strategies)
    }

    #[test]
    fn test_handle() {
        let state = make_state();
        let json = serde_json::to_string(&state).unwrap();
        println!("json: {}", json);
        let des: SavedState<StrategyKind> = serde_json::from_str(&json).unwrap();
        println!("deser: {:?}", des);
    }
}

pub enum Request {
    Get,
    Update(ChatId, SavedState<StrategyKind>),
}

pub enum Response {
    //TODO: ugly name
    Saved(HashMap<ChatId, SavedState<StrategyKind>>),
}

pub type CacheHandle = ServiceHandle<Request, Response>;

pub fn start() -> CacheHandle {
    let (sender,r) = async_channel::bounded(100);
    let (s, receiver) = async_channel::bounded(100);
    let folder = ".trader-cache";
    tokio::spawn(async move {
        while let Ok(msg) = receiver.recv().await {
            match msg {
                Request::Get => {
                    let saved = SavedState::restore(folder).await.unwrap();
                    let response = Response::Saved(saved);
                    sender.send(response).await;
                }
                Request::Update(chat, state) => {
                    let id: i64 = chat.into();
                    let path = format!("{}/{}", folder, id);
                    state.save(path.as_ref()).await;
                }
            }
        }
    });
    ServiceHandle::new(s,r)
}