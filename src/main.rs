use surrealdb::sql::{Value, Uuid};

use db::middleware::{DbInstance, DbMiddleware};
use rocket::{figment::{Figment, providers::{Toml, Format}}, Config, State, serde::{json::Json, Deserialize}};

#[macro_use] extern crate rocket;

mod db;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    name: String,
}

#[get("/get-users")]
async fn index(db: &State<DbInstance>) -> Json<Vec<Value>> {
    let query = db.query("SELECT * FROM user;").await.unwrap();

    Json(query.clone().into_iter().collect::<Vec<Value>>())
}

#[post("/create-user", data = "<user>")]
async fn create_user(user: Json<User>, db: &State<DbInstance>) -> Json<Value> {
    let uuid = Uuid::new().to_string();

    let create_user = db.query(format!("CREATE user:`{}` SET name = '{}';", uuid, user.name).as_str()).await.unwrap();

    let create_user_permission = db.query(format!("INSERT INTO permissions (name) VALUES ('Viewer') ON DUPLICATE KEY UPDATE users += `user:{}`;", uuid).as_str()).await.unwrap();

    let created_user = create_user.clone().into_iter().nth(0).unwrap();
    let created_user_permission = create_user_permission.clone().into_iter().nth(0).unwrap();

    println!("{:?}", &created_user);
    println!("{:?}", &created_user_permission);

    Json(created_user)
}

#[get("/get-permissions")]
async fn get_permissions(db: &State<DbInstance>) -> Json<Vec<Value>> {
    let query = db.query("SELECT users FROM permissions;").await.unwrap();

    Json(query.clone().into_iter().collect::<Vec<Value>>())
}

#[launch]
async fn rocket() -> _ {
    let figment = Figment::from(Config::default())
      .merge(Toml::file("Rocket.toml").nested())
      .merge(Toml::file("App.toml").nested());

    rocket::custom(figment).mount("/", routes![index, create_user, get_permissions]).attach(DbMiddleware)
}