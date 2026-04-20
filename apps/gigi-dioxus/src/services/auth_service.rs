use dirs;
use gigi_auth::{generate_mnemonic, AccountInfo, AuthManager, LoginResult};
use sea_orm::{ConnectionTrait, Database, Statement};
use std::env;
use std::path::PathBuf;

pub struct AuthService {
    auth_manager: AuthManager,
}

impl AuthService {
    pub async fn new() -> anyhow::Result<Self> {
        let data_dir = env::var("GIGI_DATA_DIR")
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("gigi-dioxus")
                    .to_string_lossy()
                    .to_string()
            });
        
        let db_path = PathBuf::from(data_dir).join("gigi.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!(
            "sqlite://{}?mode=rwc",
            db_path.to_str().unwrap_or(":memory:")
        );
        let db = Database::connect(&db_url).await?;

        // Create settings table if it doesn't exist
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT UNIQUE NOT NULL, value TEXT NOT NULL, updated_at BIGINT NOT NULL)".to_string(),
        )).await?;

        let auth_manager = AuthManager::new(db);
        Ok(Self { auth_manager })
    }

    pub async fn generate_mnemonic(&self) -> anyhow::Result<Vec<String>> {
        generate_mnemonic(12).map_err(|e| anyhow::anyhow!("Failed to generate mnemonic: {:?}", e))
    }

    pub async fn create_account(
        &mut self,
        mnemonic: &str,
        password: &str,
        name: &str,
        group_name: Option<&str>,
    ) -> anyhow::Result<AccountInfo> {
        self.auth_manager
            .create_account(
                mnemonic,
                password,
                Some(name.to_string()),
                group_name.map(String::from),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create account: {:?}", e))
    }

    pub async fn login(&mut self, password: &str) -> anyhow::Result<LoginResult> {
        self.auth_manager
            .login(password)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to login: {:?}", e))
    }

    pub async fn has_account(&self) -> anyhow::Result<bool> {
        self.auth_manager
            .has_account()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check account existence: {:?}", e))
    }

    pub async fn get_account_info(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.auth_manager
            .get_account_info()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get account info: {:?}", e))
    }

    pub async fn delete_account(&mut self) -> anyhow::Result<()> {
        self.auth_manager
            .delete_account()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete account: {:?}", e))
    }

    pub async fn verify_password(&self, password: &str) -> anyhow::Result<bool> {
        self.auth_manager
            .verify_password(password)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to verify password: {:?}", e))
    }
}
