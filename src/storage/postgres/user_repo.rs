use crate::usecases::user::crypto::SignInVerification;
use crate::usecases::user::entities::{User, UserForCreation};
use crate::usecases::user::errors::AccessModelError;
use crate::usecases::user::get_user::{FindUserById, RemoveUserById};
use crate::usecases::user::user_creator::CreateUser;
use async_trait::async_trait;
use chrono;
use deadpool_postgres::Pool;
use log::error;

pub struct UserRepo {
    db_pool: Pool,
}

impl UserRepo {
    pub fn new(db_pool: Pool) -> UserRepo {
        UserRepo { db_pool }
    }
}

#[async_trait]
impl FindUserById for UserRepo {
    async fn find_user_by_id(&self, user_id: i32) -> Result<User, AccessModelError> {
        let client = match self.db_pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::TemporaryError);
            }
        };

        let stmt = match client
            .prepare("SELECT user_id, username, enabled, created_at, updated_at FROM users where user_id=$1")
            .await
        {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        let rows = match client.query(&stmt, &[&user_id]).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        match rows.len() {
            0 => Err(AccessModelError::NotFoundError),
            _ => Ok(User::new(
                rows[0].get(0),
                rows[0].get(1),
                rows[0].get(2),
                rows[0].get(3),
                rows[0].get(4),
            )),
        }
    }
}

#[async_trait]
impl RemoveUserById for UserRepo {
    async fn remove_user_by_id(&self, user_id: i32) -> Result<(), AccessModelError> {
        let client = match self.db_pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::TemporaryError);
            }
        };
        let stmt = match client.prepare("DELETE FROM users where user_id=$1").await {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        let res = match client.execute(&stmt, &[&user_id]).await {
            Ok(res) if res == 0 => return Err(AccessModelError::NotFoundError),
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        res
    }
}

#[async_trait]
impl CreateUser for UserRepo {
    async fn save_user_in_storage(&self, user: UserForCreation) -> Result<User, AccessModelError> {
        let client = match self.db_pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::TemporaryError);
            }
        };
        let query = "INSERT INTO users (username, password_hash, enabled, created_at, updated_at, is_deleted)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING user_id, username, enabled, created_at, updated_at";
        let stmt = match client.prepare(query).await {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        let now = chrono::Utc::now();
        let user = match client
            .query(
                &stmt,
                &[
                    &user.username,
                    &user.password_hash,
                    &true,
                    &now,
                    &now,
                    &false,
                ],
            )
            .await
        {
            Ok(rows) if rows.len() == 1 => User::new(
                rows[0].get(0),
                rows[0].get(1),
                rows[0].get(2),
                rows[0].get(3),
                rows[0].get(4),
            ),
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
            Ok(_) => {
                return Err(AccessModelError::FatalError);
            }
        };
        Ok(user)
    }
}

#[async_trait]
impl SignInVerification for UserRepo {
    async fn verificate_default(
        &self,
        username: &str,
        hash: &str,
    ) -> Result<i32, AccessModelError> {
        let client = match self.db_pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::TemporaryError);
            }
        };

        let stmt = match client
            .prepare("SELECT user_id FROM users where username=$1 AND password_hash=$2")
            .await
        {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        let rows = match client.query(&stmt, &[&username, &hash]).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("{}", e);
                return Err(AccessModelError::FatalError);
            }
        };
        match rows.len() {
            0 => Err(AccessModelError::NotFoundError),
            _ => Ok(rows[0].get(0)),
        }
    }
}
