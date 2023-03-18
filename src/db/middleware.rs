use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    serde::Deserialize,
    Build, Rocket,
};
use surrealdb::{sql::Value, Datastore, Error, Session};

pub struct DbInstance {
    session: Session,
    datastore: Datastore,
}

impl DbInstance {
    pub async fn new_instance(namespace: String, database: String, datastore: String) -> Self {
        Self {
            session: Session::for_db(namespace, database),
            datastore: Datastore::new(&datastore).await.unwrap(),
        }
    }

    pub async fn query(&self, statement: &str) -> Result<Vec<Value>, Error> {
        let responses = self
            .datastore
            .execute(statement, &self.session, None, false)
            .await?;

        let mut results = Vec::new();

        for response in responses {
            results.push(response.result?.first());
        }

        Ok(results)
    }
}

pub struct DbMiddleware;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DbConfig {
    namespace: String,
    database: String,
    datastore: String,
}

#[rocket::async_trait]
impl Fairing for DbMiddleware {
    fn info(&self) -> Info {
        Info {
            name: "Database Middleware",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        let figment = rocket.figment().clone();

        let db_config: DbConfig = figment.select("database").extract().unwrap();

        let db = DbInstance::new_instance(
            db_config.namespace.clone(),
            db_config.database.clone(),
            db_config.datastore.clone(),
        )
        .await;

        // db.query("CREATE user;").await.unwrap();
        db.query(format!("CREATE permissions SET name = 'Viewer', users = []; CREATE permissions SET name = 'Admin', users = [];").as_str()).await.unwrap();

        Ok(rocket.manage(db))
    }
}
