use clap::Parser;
use rusqlite::{Connection, Result};
use std::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Parser, Debug)]
enum SubCommand {
    #[command(about = "Add a new task")]
    Add(List),

    #[command(about = "List all tasks")]
    List,

    #[command(about = "Mark a task as complete")]
    Complete(Completed),
}

#[derive(Parser, Debug)]
struct List {
    #[arg(num_args = 1..)]
    tasks: Vec<String>,
}

#[derive(Parser, Debug)]
struct Completed {
    #[arg(num_args = 1..)]
    task_id: usize,
}

#[derive(Debug, Copy)]
struct Task {
    id: usize,
    description: String,
    is_completed: bool,
}

impl Task {
    fn new(id: usize, description: &str) -> Task {
        Task {
            id,
            description: description.to_string(),
            is_completed: false,
        }
    }
}

#[derive(Debug)]
struct TodoList {
    conn: Connection,
}

impl TodoList {
    fn new() -> Result<TodoList> {
        let conn = Connection::open("tasks.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY, description TEXT, completed BOOLEAN)",
            rusqlite::params![],
        )?;
        Ok(TodoList { conn })
    }

    fn add_task(&mut self, tasks: Vec<String>) -> Result<()> {
        for i in tasks {
            self.conn.execute(
                "INSERT INTO tasks (description, completed) VALUES (?1, ?2)",
                rusqlite::params![i, false],
            )?;
        }
        Ok(())
    }

    fn list_tasks(&self) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, description, completed FROM tasks WHERE completed = ?1")?;
        let task_iter = stmt.query_map(rusqlite::params![0], |row| {
            Ok(Task {
                id: row.get(0)?,
                description: row.get(1)?,
                is_completed: row.get(2)?,
            })
        })?;
        for task in task_iter {
            let id = task?.id;
            let desc = task?.description;
            let comp = task?.is_completed;
            println!("id:{:?}, description:{:?}, completed:{:?}", id, desc, comp);
        }

        Ok(())
    }

    fn complete_task(&mut self, task_id: usize) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET completed = 1 WHERE id = ?1",
            rusqlite::params![task_id],
        )?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Process the command-line arguments
    match args.cmd {
        Some(SubCommand::Add(list)) => {
            let todo_list = TodoList::new();
            todo_list?.add_task(list.tasks)?;
        }
        Some(SubCommand::List) => {
            let todo_list = TodoList::new();
            todo_list?.list_tasks()?;
        }
        Some(SubCommand::Complete(completed)) => {
            let todo_list = TodoList::new();
            todo_list?.complete_task(completed.task_id)?;
        }
        None => println!("No subcommand provided. Use --help for usage information."),
    }

    Ok(())
}
