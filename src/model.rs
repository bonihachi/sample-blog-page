use mongodb::{
    sync::Client,
    options::FindOptions,
    bson,
    bson::{doc, Bson},
    bson::oid::ObjectId,
};
use serde::{Serialize, Deserialize};
use std::str;
use argon2::{self, Config};
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::IntoOutcome;
const SALT: &[u8; 10] = b"randomsalt";

#[derive(Debug)]
pub struct LoggedInUser(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for LoggedInUser {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<LoggedInUser, !> {
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| LoggedInUser(id))
            .or_forward(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
pub struct Post {
    pub title: String,
    pub body: String,
    pub author: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
pub struct PostWithId {
    pub _id: String,
    pub title: String,
    pub body: String,
    pub author: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
pub struct EncryptedUser {
    pub username: String,
    pub encrypted_password: String,
}

impl EncryptedUser {
    pub fn encrypt_user(user: User) -> EncryptedUser {
        let config = Config::default();
        let hash = argon2::hash_encoded(user.password.as_bytes(), SALT, &config).unwrap();
        EncryptedUser {
            username: user.username,
            encrypted_password: hash,
        }
    }

    pub fn compare(self: &Self, password: &str) -> bool {
        println!("{}", self.encrypted_password.as_str());
        println!("{}", password);
        argon2::verify_encoded(self.encrypted_password.as_str(), password.as_bytes()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
pub struct UserWithId {
    pub _id: String,
    pub username: String,
    pub password: String,
}

pub fn connect_mongo() -> Result<Client, mongodb::error::Error> {
    let client = Client::with_uri_str("mongodb://localhost:27017")?;
    Ok(client)
}

pub fn list_db(client: &Client) -> Result<(), mongodb::error::Error> {
    println!("db:");
    for db_name in client.list_database_names(None, None)? {
        println!("{}", db_name);
    }
    Ok(())
}

pub fn list_collections(client: &Client, dbname: &str) -> Result<(), mongodb::error::Error> {
    let db = client.database(dbname);
    println!("collections ({}):", dbname);
    for collection_name in db.list_collection_names(None)? {
        println!("{}", collection_name);
    }
    Ok(())
}

//takes a reference for a client, post object, name of database, and name of collection as arguments
pub fn insert_post(client: &Client, post: Post, dbname: &str, coll_name: &str) -> Result<(), mongodb::error::Error> {
    let db = client.database(dbname);
    let collection = db.collection(coll_name);
    let new_doc = bson::to_bson(&post)?;
    if let bson::Bson::Document(document) = new_doc {
        collection.insert_one(document, None)?;
    }
    else {
        println!("Error convertintg the BSON object into a MongDB document");
    }
    Ok(())
}

pub fn insert_user(client: &Client, user: EncryptedUser, dbname: &str, coll_name: &str) -> Result<(), mongodb::error::Error> {
    let db = client.database(dbname);
    let collection = db.collection(coll_name);
    let new_doc = bson::to_bson(&user)?;
    if let bson::Bson::Document(document) = new_doc {
        collection.insert_one(document, None)?;
    }
    else {
        println!("Error convertintg the BSON object into a MongDB document");
    }
    Ok(())
}

pub fn find_all(client: &Client, dbname: &str, coll_name: &str) -> Result<Vec<mongodb::bson::Document>, mongodb::error::Error> {
    let mut v: Vec<mongodb::bson::Document> = Vec::new();
    let db = client.database(dbname);
    let collection = db.collection(coll_name);
    let find_options = FindOptions::builder().sort(doc! {"title": 1}).build();
    let cursor = collection.find(None, find_options)?;
    for doc in cursor {
        v.push(doc.unwrap());
    }
    Ok(v)
}

pub fn delete_all(client: &Client, dbname: &str, coll: &str) -> Result<(), mongodb::error::Error> {
    let db = client.database(dbname);
    let collection = db.collection(coll);
    collection.drop(None)?;
    Ok(())
}

pub fn delete_by_id(id: ObjectId, client: &Client) -> Result<(), mongodb::error::Error> {
    let db = client.database("my_database");
    let collection = db.collection("blogposts");
    let filter = doc! { "_id": id};
    let find_options = FindOptions::builder().sort(doc! {"_id": 1}).build();
    let cursor = collection.find(filter, find_options)?;
    for doc in cursor {
        collection.delete_one(doc?, None).unwrap();
    }
    Ok(())
}

pub fn find_by_id(id: String, client: &Client) -> Option<Post> {
    let id = ObjectId::with_string(id.as_str()).unwrap();
    let db = client.database("my_database");
    let collection = db.collection("blogposts");
    let filter = doc! { "_id": id};
    let result = collection.find_one(filter, None).unwrap();
    match result {
        Some(doc) => {
            return Some(
            Post {
                title: doc.get("title").and_then(Bson::as_str).unwrap().to_string(),
                body: doc.get("body").and_then(Bson::as_str).unwrap().to_string(),
                author: doc.get("author").and_then(Bson::as_str).unwrap().to_string(),
                date: doc.get("date").and_then(Bson::as_str).unwrap().to_string(),
            })
        },
        None => None
    }
}

pub fn find_by_username(username: &String, client: &Client) -> Option<String> {
    let db = client.database("my_database");
    let collection = db.collection("users");
    let filter = doc! {"username": username};
    println!("{}", username);
    let result = collection.find_one(filter, None).unwrap();
    match result {
        Some(doc) => {
            Some(doc.get_object_id("_id").unwrap().to_hex())
        },
        None => None
    }
}

pub fn successfully_logged_in(client: &Client, user: &User) -> bool {
    let db = client.database("my_database");
    let collection = db.collection("users");
    let filter = doc! { "username": &user.username};
    let result = collection.find_one(filter, None).unwrap();
    match result {
        Some(doc) => {
            let eu = EncryptedUser {
                username: doc.get("username").and_then(Bson::as_str).unwrap().to_string(),
                encrypted_password: doc.get("encrypted_password").and_then(Bson::as_str).unwrap().to_string(),
            };
            return EncryptedUser::compare(&eu, &user.password.as_str())
        },
        None => {
            return false
        }
    }
}