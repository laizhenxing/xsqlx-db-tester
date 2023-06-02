use sqlx::{migrate::Migrator, Connection, Executor, PgConnection, PgPool};
use std::{path::Path, thread};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub struct TestDB {
    pub server_url: String,
    pub dbname: String,
}

impl TestDB {
    pub fn new(server_url: impl Into<String>, miration_path: impl Into<String>) -> TestDB {
        let uuid = Uuid::new_v4();
        let dbname = format!("testdb_{}", uuid);
        let tdb = TestDB {
            server_url: server_url.into(),
            dbname: dbname.clone(),
        };

        let server_url = tdb.server_url();
        let url = tdb.url();
        let migration_path = miration_path.into();

        // create database with dbname
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                // use server url to create database
                let mut conn = PgConnection::connect(&server_url).await.unwrap();
                conn.execute(format!(r#"CREATE DATABASE "{}""#, dbname.clone()).as_ref())
                    .await
                    .unwrap_or_else(|_| panic!("Failed to create database {}", dbname));

                // create a new connection for migration
                let mut conn = PgConnection::connect(&url).await.unwrap();
                let m = Migrator::new(Path::new(&migration_path)).await.unwrap();
                m.run(&mut conn)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to migrate"));
            });
        })
        .join()
        .expect("Failed to execute database operation");

        tdb
    }

    pub fn url(&self) -> String {
        format!("{}/{}", self.server_url.clone(), self.dbname.clone())
    }

    pub fn server_url(&self) -> String {
        self.server_url.clone()
    }

    pub async fn get_pool(&self) -> PgPool {
        PgPool::connect(&self.url()).await.unwrap()
    }
}

impl Drop for TestDB {
    fn drop(&mut self) {
        let url = self.server_url();
        let dbname = self.dbname.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut conn = PgConnection::connect(&url).await.unwrap();
                // terminate all other connections
                sqlx::query(&format!(
                    r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity
                    WHERE pid <> pg_backend_pid() AND datname = '{}'"#,
                    dbname
                ))
                .execute(&mut conn)
                .await
                .expect("Terminate all other connections");

                // drop test database
                conn.execute(format!(r#"DROP DATABASE "{}""#, dbname).as_str())
                    .await
                    .expect("Error while querying the drop database");
            });
        })
        .join()
        .expect("failed to drop database");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::Row;

    #[tokio::test]
    async fn test_db_should_create_and_drop() {
        let tdb = TestDB::new(
            "postgres://postgres:postgres@localhost:5432",
            "./fitures/migrations",
        );
        let url = tdb.url();
        let mut conn = PgConnection::connect(&url).await.unwrap();
        let row = sqlx::query("SELECT 1")
            .fetch_one(&mut conn)
            .await
            .expect("Failed to query");
        assert_eq!(row.get::<i32, _>(0), 1);
    }
}
