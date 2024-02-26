use std::collections::HashMap;
use std::sync::Arc;
use axum::{routing::{get, post}, Router, Json};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use time::{Date, macros::date};
use tokio::sync::{RwLock};
use uuid::Uuid;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PersonName(String);
impl TryFrom<String> for PersonName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            Ok(Self(value))
        } else {
            Err("Nome eh muito grande.")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Tech(String);
impl TryFrom<String> for crate::Tech {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            Ok(Self(value))
        } else {
            Err("Tech eh muito grande.")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Nick(String);
impl TryFrom<String> for crate::Nick {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 100 {
            Ok(Self(value))
        } else {
            Err("Nick eh muito grande.")
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: PersonName,
    #[serde(rename = "apelido")]
    pub nick: Nick,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<Tech>>,
}

type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let mut people: HashMap<Uuid, Person> = HashMap::new();


    let person = Person {
        id: Uuid::now_v7(),
        name: String::from("Guilherme"),
        nick: String::from("castro13"),
        birth_date: date!(1995 - 04 - 13),
        stack: None
    };
    println!("{}",person.id);
    people.insert(person.id, person);

    let app_state: AppState = Arc::new(RwLock::new(people));

    let app = Router::new()
        .route("/pessoas", get( search_people ))
        .route("/pessoas/:id", get(find_person))
        .route("/pessoas", post(create_person))
        .route("/contagem-pessoas", get(count_people))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_people(
    State(_people): State<AppState>
) -> impl IntoResponse {

}

async fn find_person(
    State(people): State<AppState>,
    Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
    match people.read().await.get(&person_id) {
        Some(person) => Ok((StatusCode::OK, Json(person.clone()))),
        None => Err((StatusCode::NOT_FOUND, "Pessoa nao encontrada."))
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    let person = Person {
        id,
        name: new_person.name.0,
        nick: new_person.nick.0,
        birth_date: new_person.birth_date,
        stack: new_person.stack
    };

    people.write().await.insert(id, person.clone());
    (StatusCode::CREATED, Json(person))
}

async fn count_people(
    State(people): State<AppState>
) -> impl IntoResponse {
    let count = people.read().await.len();
    (StatusCode::OK, Json(count))
}