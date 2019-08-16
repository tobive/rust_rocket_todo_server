#![feature(proc_macro_hygiene, decl_macro, drain_filter)]

#[macro_use]
extern crate rocket;

use rocket::http::RawStr;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, Error, SeekFrom};
use std::sync::Mutex;

const TODO_FILE: &'static str = "foo.json";

#[derive(Serialize, Deserialize, Debug)]
struct Todo {
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct TodoFile {
    todo_list: Vec<Todo>,
}

impl TodoFile {
    fn to_json_string(&self) -> Result<String, serde::errors::Error> {
        serde_json::to_string(self)
    }
}

struct NewAppState {
    todo_file: Mutex<File>,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, todo list!"
}

#[get("/api/list?<query>")]
fn return_list(query: Option<&RawStr>, new_state: State<NewAppState>) -> String {
    let file_handle = new_state.todo_file.lock().unwrap();
    let mut buf_reader = BufReader::new(&*file_handle);

    let data: TodoFile = read_data_to_json(&mut buf_reader).unwrap();
    match &query {
        None => {
            let string_json = data.to_json_string().unwrap();
            format!("{}", &string_json)
        }
        Some(query) => {
            let query = query.to_string();
            match &data.todo_list.iter().filter(|todo| query == todo.title).nth(0) {
                Some(todo) => format!("{:?}\n", todo),
                None => format!("No todo with such title found.\n"),
            }
        }
    }
}

#[post("/api/post", format = "application/json", data = "<todo>")]
fn post_data(todo: Json<Todo>, new_state: State<NewAppState>) -> String {
    let todo = todo.into_inner();
    let Todo {title, content} = todo;

    let mut file_handle = new_state.todo_file.lock().unwrap();
    let mut data = read_data_to_json(&mut *file_handle).unwrap();
    // Check if title already exist to prevent duplicate

    if let Some(_) = &data.todo_list.iter().find(|todo| title == todo.title) {
        return format!("title already exist. Can't save the item.\n");
    };

    let new_todo = Todo {
        title: title.clone(),
        content: content.clone(),
    };
    data.todo_list.push(new_todo);

    let string_json = data.to_json_string().unwrap();
    println!("isi baru: {}", &string_json);
    let mut new_handler = File::create(TODO_FILE).unwrap();
    match new_handler.write(string_json.as_bytes()).unwrap() {
        0 => format!("Error saving"),
        _ => format!("Success saving\n"),
    }
}

#[delete("/api/delete?<query>")]
fn delete_item(query: String, new_state: State<NewAppState>) -> String {
    let mut file_handle = new_state.todo_file.lock().unwrap();
    let mut data: TodoFile = read_data_to_json(&mut *file_handle).unwrap();

    let new_vec: Vec<Todo> = data.todo_list.drain_filter(|todo| todo.title == query).collect();

    let updated_list = TodoFile {
        todo_list: new_vec,
    };
    let string_json = updated_list.to_json_string().unwrap();

    let mut new_handler = File::create(TODO_FILE).unwrap();
    match new_handler.write(string_json.as_bytes()).unwrap() {
        0 => format!("Error.\n"),
        _ => format!("Success deleting.\n"),
    }
}

fn read_data_to_json<T>(file: &mut T) -> Result<TodoFile, Error>
where
    T: Seek + Read,
{
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0))?; // -> reset position to the start of file
    file.read_to_string(&mut contents)?;
    let data= serde_json::from_str(&contents)?;
    Ok(data)
}

fn main() {
    // let file: File = File::open("foo.txt").unwrap(); // --> Won't work because LIFETIME
    rocket::ignite()
        .mount("/", routes![index, return_list, post_data, delete_item])
        .manage(NewAppState {
            todo_file: Mutex::new(
                // File::open("foo.txt").unwrap()
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(TODO_FILE)
                    .unwrap(),
            ),
        })
        .launch();
}

// PR
// Todo list
//  - GET list?query="string"
//  - POST data=string
//  - DELETE by query="string"
// Bonus:
// - use json
// - save list in file
