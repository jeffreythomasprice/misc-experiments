use shared::user::{CreateUserRequest, UpdateUserRequest};
use sqlx::{Pool, Sqlite};

use crate::errors::Error;

use super::models::User;

pub struct Service {
    db: Pool<Sqlite>,
}

impl Service {
    pub fn new(db: Pool<Sqlite>) -> Service {
        Service { db }
    }

    pub async fn list(&self) -> Result<Vec<User>, Error> {
        Ok(
            sqlx::query_as::<_, User>("SELECT name, password, is_admin FROM users")
                .fetch_all(&self.db)
                .await?,
        )
    }

    pub async fn get_by_name(&self, name: &str) -> Result<User, Error> {
        match sqlx::query_as::<_, User>("SELECT name, password, is_admin FROM users WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.db)
            .await?
        {
            Some(result) => Ok(result),
            None => Err(Error::NotFound(name.to_string())),
        }
    }

    pub async fn create(&self, request: &CreateUserRequest) -> Result<(), Error> {
        sqlx::query("INSERT INTO users (name, password, is_admin) VALUES (?, ?, ?)")
            .bind(&request.name)
            .bind(&request.password)
            .bind(request.is_admin)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn update(&self, name: &str, request: &UpdateUserRequest) -> Result<(), Error> {
        let mut query = sqlx::query_builder::QueryBuilder::new("UPDATE users SET ");
        let mut separated = query.separated(", ");
        if let Some(value) = &request.password {
            separated.push("password = ").push_bind_unseparated(value);
        }
        if let Some(value) = request.is_admin {
            separated.push("is_admin = ").push_bind_unseparated(value);
        }
        query.push(" WHERE name = ").push_bind(name);
        let result = query.build().execute(&self.db).await?;
        if result.rows_affected() == 0 {
            Err(Error::NotFound(name.to_string()))
        } else {
            Ok(())
        }
    }

    pub async fn delete_by_name(&self, name: &str) -> Result<(), Error> {
        let result = sqlx::query("DELETE FROM users WHERE name = ?")
            .bind(name)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            Err(Error::NotFound(name.to_string()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::db::create_db_for_test;

    use super::*;

    #[async_test]
    async fn create() -> Result<(), Error> {
        let service = Service::new(create_db_for_test().await?);

        let users = service.list().await?;
        assert_eq!(
            [User {
                name: "admin".to_string(),
                password: "admin".to_string(),
                is_admin: true
            }],
            users.as_slice()
        );

        service
            .create(&CreateUserRequest {
                name: "foo".to_string(),
                password: "bar".to_string(),
                is_admin: true,
            })
            .await?;

        let users = service.list().await?;
        assert_eq!(
            [
                User {
                    name: "admin".to_string(),
                    password: "admin".to_string(),
                    is_admin: true
                },
                User {
                    name: "foo".to_string(),
                    password: "bar".to_string(),
                    is_admin: true
                }
            ],
            users.as_slice()
        );

        Ok(())
    }

    #[async_test]
    async fn create_fail_duplicate() -> Result<(), Error> {
        let service = Service::new(create_db_for_test().await?);

        service
            .create(&CreateUserRequest {
                name: "foo".to_string(),
                password: "bar".to_string(),
                is_admin: true,
            })
            .await?;

        let result = service
            .create(&CreateUserRequest {
                name: "foo".to_string(),
                password: "baz".to_string(),
                is_admin: true,
            })
            .await;
        assert!(result.is_err());

        Ok(())
    }

    #[async_test]
    async fn update() -> Result<(), Error> {
        let service = Service::new(create_db_for_test().await?);

        service
            .create(&CreateUserRequest {
                name: "foo".to_string(),
                password: "asdfasdf".to_string(),
                is_admin: true,
            })
            .await?;
        service
            .create(&CreateUserRequest {
                name: "bar".to_string(),
                password: "asdfasdf".to_string(),
                is_admin: false,
            })
            .await?;

        service
            .update(
                "foo",
                &UpdateUserRequest {
                    password: Some("new_password".to_string()),
                    is_admin: None,
                },
            )
            .await?;

        service
            .update(
                "bar",
                &UpdateUserRequest {
                    password: None,
                    is_admin: Some(true),
                },
            )
            .await?;

        let users = service.list().await?;
        assert_eq!(
            [
                User {
                    name: "admin".to_string(),
                    password: "admin".to_string(),
                    is_admin: true
                },
                User {
                    name: "foo".to_string(),
                    password: "new_password".to_string(),
                    is_admin: true
                },
                User {
                    name: "bar".to_string(),
                    password: "asdfasdf".to_string(),
                    is_admin: true
                }
            ],
            users.as_slice()
        );

        assert_eq!(
            User {
                name: "foo".to_string(),
                password: "new_password".to_string(),
                is_admin: true
            },
            service.get_by_name("foo").await?
        );
        assert_eq!(
            User {
                name: "bar".to_string(),
                password: "asdfasdf".to_string(),
                is_admin: true
            },
            service.get_by_name("bar").await?
        );
        assert!(service.get_by_name("baz").await.is_err());

        Ok(())
    }

    #[async_test]
    async fn update_fail_no_such_user() -> Result<(), Error> {
        let service = Service::new(create_db_for_test().await?);

        let result = service
            .update(
                "foo",
                &UpdateUserRequest {
                    password: Some("new_password".to_string()),
                    is_admin: Some(false),
                },
            )
            .await
            .unwrap_err();
        let expected = Error::NotFound("foo".to_string());
        assert_eq!(expected, result);

        Ok(())
    }

    #[async_test]
    async fn delete() -> Result<(), Error> {
        let service = Service::new(create_db_for_test().await?);

        service
            .create(&CreateUserRequest {
                name: "foo".to_string(),
                password: "bar".to_string(),
                is_admin: true,
            })
            .await?;

        let users = service.list().await?;
        assert_eq!(
            [
                User {
                    name: "admin".to_string(),
                    password: "admin".to_string(),
                    is_admin: true
                },
                User {
                    name: "foo".to_string(),
                    password: "bar".to_string(),
                    is_admin: true
                }
            ],
            users.as_slice()
        );

        let result = service.delete_by_name("foo").await;
        assert!(result.is_ok());

        let users = service.list().await?;
        assert_eq!(
            [User {
                name: "admin".to_string(),
                password: "admin".to_string(),
                is_admin: true
            }],
            users.as_slice()
        );

        Ok(())
    }

    #[async_test]
    async fn delete_fail_no_such_user() -> Result<(), Error> {
        let service = Service::new(create_db_for_test().await?);

        let result = service.delete_by_name("foo").await.unwrap_err();
        let expected = Error::NotFound("foo".to_string());
        assert_eq!(expected, result);

        Ok(())
    }
}
