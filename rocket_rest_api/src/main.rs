#[macro_use]
extern crate rocket;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use std::sync::Mutex;

// Define the structure for a Task (same as above)
#[derive(Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
}

// Shared state to store the tasks (thread-safe using Mutex)
struct TaskList {
    tasks: Mutex<Vec<Task>>,
}

// POST /tasks - Create a new task
#[post("/tasks", format = "json", data = "<task>")]
fn create_task(task: Json<Task>, state: &State<TaskList>) -> Json<&'static str> {
    let mut tasks = state.tasks.lock().unwrap();
    tasks.push(task.into_inner());
    Json("Task created")
}

// GET /tasks - Retrieve all tasks
#[get("/tasks")]
fn get_tasks(state: &State<TaskList>) -> Json<Vec<Task>> {
    let tasks = state.tasks.lock().unwrap();
    Json(tasks.clone())
}

// GET /tasks/<id> - Retrieve a task by ID
#[get("/tasks/<id>")]
fn get_task(id: usize, state: &State<TaskList>) -> Option<Json<Task>> {
    let tasks = state.tasks.lock().unwrap();
    tasks
        .iter()
        .find(|task| task.id == id)
        .map(|task| Json(task.clone()))
}

// PUT /tasks/<id> - Update an existing task
#[put("/tasks/<id>", format = "json", data = "<updated_task>")]
fn update_task(
    id: usize,
    updated_task: Json<Task>,
    state: &State<TaskList>,
) -> Option<Json<&'static str>> {
    let mut tasks = state.tasks.lock().unwrap();
    let task = tasks.iter_mut().find(|task| task.id == id)?;
    *task = updated_task.into_inner();
    Some(Json("Task updated"))
}

// DELETE /tasks/<id> - Delete a task by ID
#[delete("/tasks/<id>")]
fn delete_task(id: usize, state: &State<TaskList>) -> Option<Json<&'static str>> {
    let mut tasks = state.tasks.lock().unwrap();
    let index = tasks.iter().position(|task| task.id == id)?;
    tasks.remove(index);
    Some(Json("Task deleted"))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(TaskList {
            tasks: Mutex::new(vec![]),
        }) // Initialize with an empty task list
        .mount(
            "/",
            routes![create_task, get_tasks, get_task, update_task, delete_task],
        )
}
