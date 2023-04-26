use async_trait::async_trait;
use google_youtube3::oauth2::storage::{TokenInfo, TokenStorage};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct JsonTokenStore {
    program_name: String,
    db_dir: String,
}

impl JsonTokenStore {
    pub fn new(program_name: String, db_dir: String) -> Self {
        Self {
            program_name,
            db_dir,
        }
    }
    fn path(&self, scopes: &[&str]) -> PathBuf {
        let mut s = DefaultHasher::new();
        scopes.hash(&mut s);

        let scope_hash = s.finish();
        Path::new(&self.db_dir).join(format!("{}-token-{}.json", self.program_name, scope_hash))
    }
}

#[async_trait]
impl TokenStorage for JsonTokenStore {
    async fn set(&self, scopes: &[&str], token: TokenInfo) -> anyhow::Result<()> {
        let data = serde_json::to_string(&token).unwrap();

        log::trace!("Storing token for scopes {:?}", scopes);

        let res = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.path(scopes));

        if let Ok(mut f) = res {
            f.write(data.as_bytes()).ok();
        }

        Ok(())
    }

    async fn get(&self, target_scopes: &[&str]) -> Option<TokenInfo> {
        if let Ok(mut f) = fs::File::open(self.path(target_scopes)) {
            let mut json_string = String::new();
            if f.read_to_string(&mut json_string).is_ok() {
                if let Ok(token) = serde_json::from_str::<TokenInfo>(&json_string) {
                    log::trace!("Reading token for scopes {:?}", target_scopes);
                    return Some(token);
                }
            }
        }
        None
    }
}
