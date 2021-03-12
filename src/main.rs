#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use] extern crate rocket;

use rocket_contrib::templates::Template;
use rocket::response::{Redirect, Flash};
use rocket_contrib::serve::StaticFiles;
use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use std::collections::HashMap;
use mongodb::{
    bson::doc,
    bson::Bson,
};
use chrono::{DateTime, Local};
use substring::Substring;

mod model;

const DATABASE: &str = "my_database";
const BLOGPOSTS: &str = "blogposts";
const USERS: &str = "users";

fn get_context() -> Vec<mongodb::bson::Document> {
    let client = model::connect_mongo().unwrap();
    model::find_all(&client, DATABASE, BLOGPOSTS).unwrap()
}

#[get("/", rank = 2)]
fn index() -> Template {
    let mut context:HashMap<String, Vec<model::PostWithId>> = HashMap::new();
    let documents = get_context();
    let mut posts: Vec<model::PostWithId> = Vec::new();
    for doc in documents {
        let pwi = model::PostWithId {
            _id: doc.get_object_id("_id").unwrap().to_hex(),
            title: doc.get("title").and_then(Bson::as_str).unwrap().to_string(),
            body: doc.get("body").and_then(Bson::as_str).unwrap().to_string(),
            author: doc.get("author").and_then(Bson::as_str).unwrap().to_string(),
            date: doc.get("date").and_then(Bson::as_str).unwrap().to_string(),
        };
        posts.push(pwi);
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    let mut new_posts: Vec<model::PostWithId> = Vec::new();
    for post in posts { 
        new_posts.push(
            model::PostWithId {
                _id: post._id,
                title: post.title,
                body: post.body,
                author: post.author,
                date: post.date.as_str().substring(0,10).to_string(),
            }
        );
    };
    context.insert("posts".to_string(), new_posts);
    Template::render("index", &context)
}

#[get("/")]
fn user_index(user: model::LoggedInUser) -> Template {
    let mut context:HashMap<String, Vec<model::PostWithId>> = HashMap::new();
    let documents = get_context();
    let mut posts: Vec<model::PostWithId> = Vec::new();
    for doc in documents {
        let pwi = model::PostWithId {
            _id: doc.get_object_id("_id").unwrap().to_hex(),
            title: doc.get("title").and_then(Bson::as_str).unwrap().to_string(),
            body: doc.get("body").and_then(Bson::as_str).unwrap().to_string(),
            author: doc.get("author").and_then(Bson::as_str).unwrap().to_string(),
            date: doc.get("date").and_then(Bson::as_str).unwrap().to_string(),
        };
        posts.push(pwi);
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    let mut new_posts: Vec<model::PostWithId> = Vec::new();
    for post in posts { 
        new_posts.push(
            model::PostWithId {
                _id: post._id,
                title: post.title,
                body: post.body,
                author: post.author,
                date: post.date.as_str().substring(0,10).to_string(),
            }
        );
    };
    context.insert("posts".to_string(), new_posts);
    context.insert("user_id".to_string(), vec![model::PostWithId {_id: user.0.to_string(), title: "".to_string(), body: "".to_string(), author: "".to_string(), date: "".to_string()}]);
    println!("{:?}", context.get("user_id"));
    Template::render("index", &context)
}

#[get("/about")]
fn about_loggedin(user: model::LoggedInUser) -> Template {
    let mut context:HashMap<String, Vec<model::PostWithId>> = HashMap::new();
    let ids: Vec<model::PostWithId> = Vec::new();
    context.insert("posts".to_string(), ids);
    context.insert("user_id".to_string(), vec![model::PostWithId {_id: user.0.to_string(), title: "".to_string(), body: "".to_string(), author: "".to_string(), date: "".to_string()}]);
    Template::render("about", &context)
}

#[get("/about", rank = 2)]
fn about() -> Template {
    let mut context = HashMap::new();
    context.insert("test", "test");
    Template::render("about", &context)
}

#[get("/contact")]
pub fn contact_loggedin(user: model::LoggedInUser) -> Template {
    let mut context:HashMap<String, Vec<model::PostWithId>> = HashMap::new();
    let ids: Vec<model::PostWithId> = Vec::new();
    context.insert("posts".to_string(), ids);
    context.insert("user_id".to_string(), vec![model::PostWithId {_id: user.0.to_string(), title: "".to_string(), body: "".to_string(), author: "".to_string(), date: "".to_string()}]);
    Template::render("contact", &context)
}

#[get("/contact", rank = 2)]
pub fn contact() -> Template {
    let mut context = HashMap::new();
    context.insert("test", "test");
    Template::render("contact", &context)
}

#[get("/post/<id>")]
pub fn single_post_loggedin(_user: model::LoggedInUser, id: String) -> Template {
    let client = model::connect_mongo().unwrap();
    let post = model::find_by_id(id, &client).unwrap();
    let mut context = HashMap::new();
    context.insert(String::from("post"), post);
    context.insert("user_id".to_string(), model::Post {title: "".to_string(), body: "".to_string(), author: "".to_string(), date: "".to_string()});
    Template::render("contact", &context)
}

#[get("/post/<id>", rank = 2)]
pub fn single_post(id: String) -> Template {
    let client = model::connect_mongo().unwrap();
    let mut post = model::find_by_id(id, &client).unwrap();
    post.date = post.date.as_str().substring(0,10).to_string();
    let mut context = HashMap::new();
    context.insert(String::from("post"), post);
    Template::render("post", &context)
}

#[get("/posts", rank = 2)]
pub fn create_not_loggedin() -> Redirect {
    let mut context = HashMap::new();
    context.insert("test", "test");
    Redirect::to(uri!(index))
}

#[get("/posts")]
pub fn create(_user: model::LoggedInUser) -> Template {
    let mut context = HashMap::new();
    context.insert("test", "test");
    Template::render("create", &context) 
}

#[post("/posts", data = "<new_post>")]
pub fn new_post(user: model::LoggedInUser, new_post: String) -> Redirect {
    let date_time: DateTime<Local> = Local::now();
    let parsed = url::form_urlencoded::parse(new_post.as_bytes()); // parse x-www-url-encoded
    let mut iter = parsed.into_iter(); // iterator for Parsed
    let client = model::connect_mongo().unwrap();
    let post: model::Post = model::Post {
        title: iter.next().unwrap().1.to_string(),
        body: iter.next().unwrap().1.to_string(),
        author: user.0,
        date: date_time.to_string(),
    };
    model::insert_post(&client, post, DATABASE, BLOGPOSTS).unwrap();
    Redirect::to("/")
}

#[get("/register")]
pub fn register() -> Template {
    let mut context = HashMap::new();
    context.insert("test", "test");
    Template::render("register", &context)
}

#[post("/register", data = "<user>")]
pub fn new_user(user: Form<model::User>) -> Redirect {
    let client = model::connect_mongo().unwrap();
    let user: model::User = user.into_inner();
    let user = model::EncryptedUser::encrypt_user(user);
    model::insert_user(&client, user, DATABASE, USERS).unwrap();
    Redirect::to("/")
}

#[get("/login")]
pub fn login() -> Template {
    let mut context = HashMap::new();
    context.insert("test", "test");
    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
fn login_user(mut cookies: Cookies, user: Form<model::User>) -> Result<Redirect, Flash<Redirect>> {
    let client = model::connect_mongo().unwrap();
    let user = user.into_inner();
    if model::successfully_logged_in(&client, &user) {
        cookies.add_private(Cookie::new("user_id", user.username));
        println!("{}",cookies.get_private("user_id").unwrap());
        Ok(Redirect::to(uri!(index)))
    }
    else {
        Err(Flash::error(Redirect::to(uri!(login)), "Invalid username/password."))
    }
}

#[post("/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to(uri!(index)), "Successfully logged out.")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, user_index, about, about_loggedin, contact, contact_loggedin, single_post, create, create_not_loggedin, new_post, register, new_user, login, login_user, logout])
        .mount("/static", StaticFiles::from("static"))
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
