#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::RawStr;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::io::SeekFrom;
use std::sync::Mutex;

const TODO_FILE: &'static str = "foo.json";

#[derive(Serialize, Deserialize)]
struct Todo {
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct TodoFile {
    todo_list: Vec<Todo>,
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
    let file_handle = new_state.todo_file.lock().unwrap().try_clone().unwrap();
    let mut buf_reader = BufReader::new(file_handle);
    let data: TodoFile = read_data_to_json(&mut buf_reader).unwrap();
    match &query {
        None => {
            let string_json = serde_json::to_string(&data).unwrap();
            println!("isi string_json: {}", string_json);
            return format!("{}", &string_json);
        }
        Some(query) => {
            let mut res: Option<String> = None;
            for todo in &data.todo_list {
                if query.to_string() == todo.title.to_string() {
                    res = Some(todo.content.to_string());
                }
            }
            match res {
                None => format!("No todo with such title found.\n"),
                Some(cont) => format!("{}\n", cont),
            }
        }
    }
}

#[post("/api/post", format = "application/json", data = "<todo>")]
fn post_data<'r>(todo: Json<Todo>, new_state: State<'r, NewAppState>) -> String {
    let (title, content) = (&todo.0.title, &todo.0.content);

    let mut file_handle = new_state.todo_file.lock().unwrap().try_clone().unwrap();
    let mut data: TodoFile = read_data_to_json(&mut file_handle).unwrap();
    // Check if title already exist to prevent duplicate
    for todo in &data.todo_list {
        if title.to_string() == todo.title.to_string() {
            return format!("title already exist. Can't save the item.\n");
        }
    }

    let new_todo: Todo = Todo {
        title: title.clone(),
        content: content.clone(),
    };
    data.todo_list.push(new_todo);
    let string_json = serde_json::to_string(&data).unwrap();
    println!("isi baru: {}", &string_json);
    let mut new_handler = File::create(TODO_FILE).unwrap();
    match new_handler.write(string_json.as_bytes()).unwrap() {
        0 => format!("Error saving"),
        _ => format!("Success saving\n"),
    }
}

#[delete("/api/delete?<query>")]
fn delete_item(query: String, new_state: State<NewAppState>) -> String {
    let mut file_handle = new_state.todo_file.lock().unwrap().try_clone().unwrap();
    let data: TodoFile = read_data_to_json(&mut file_handle).unwrap();

    let mut temp_vec: Vec<Todo> = Vec::new();
    for item in &data.todo_list {
        if item.title.to_string() != query {
            let todo = Todo {
                title: item.title.to_string(),
                content: item.content.to_string(),
            };
            temp_vec.push(todo);
        }
    }
    let updated_list = TodoFile {
        todo_list: temp_vec,
    };
    let string_json = serde_json::to_string(&updated_list).unwrap();
    let mut new_handler = File::create(TODO_FILE).unwrap();
    match new_handler.write(string_json.as_bytes()).unwrap() {
        0 => format!("Error.\n"),
        _ => format!("Success deleting.\n"),
    }
}

fn read_data_to_json<'a, T>(file: &'a mut T) -> Result<TodoFile, Error>
where
    T: Seek + Read,
{
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0)).unwrap(); // -> reset position to the start of file
    file.read_to_string(&mut contents).unwrap();
    let data: TodoFile = serde_json::from_str(&contents).unwrap();
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
