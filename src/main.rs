pub mod database;
pub mod command;
pub mod utils;
pub mod todo;
pub mod workspace;
pub mod cli;
pub mod sys;
pub mod key_binding;
pub mod log;
mod ui {
    pub mod todolist;
    pub mod view_list;
}

use structopt::StructOpt;
use std::error::Error;
use ui::todolist::TodoList;
use ui::view_list::show_list;
use cli::{ Cli, WorkspaceCommand };

fn main() -> Result<(), Box<dyn Error>> {

    sys::initialize();

    let workspace_name = match workspace::get_workspace() {
        Ok(workspace_name) => workspace_name,
        _ => String::from("Default")
    };
    let db_filename = database::get_db_filename_from_workspace_name(&workspace_name);

    match StructOpt::from_args() {
        Cli::Workspace(ws_command) => {
            match ws_command {
                WorkspaceCommand::Set { name } => {
                    workspace::set_workspace(&name);
                    println!("Workspace set to \"{}\" \u{2714}", name);
                    return Ok(());
                },
                WorkspaceCommand::Unset => {
                    workspace::unset_workspace();
                    database::get_db_filename_from_workspace_name("Default");
                },
                WorkspaceCommand::List => {
                    let entries = workspace::list_workspaces().expect("Error listing workspaces");
                    entries
                        .into_iter()
                        .for_each(|e| {
                            println!("\u{25FD} {}", e);
                        });
                    return Ok(());
                },
                WorkspaceCommand::Remove { name } => {
                    workspace::remove_workspace(&name);
                    println!("Workspace \"{}\" removed \u{1F5D1}", name);
                    return Ok(());
                }
            }
        },
        Cli::Add {todo_name} => {
            command::add(todo_name, &db_filename);
            return Ok(());
        },
        Cli::Version => {
            let version = env!("CARGO_PKG_VERSION");
            println!("version: {}", version);
            return Ok(());
        },
        _ => println!("Continuing to list")
    }

    let db = database::read(&db_filename);
    let mut todo_list = TodoList::new(db);
    todo_list.items.state.select(Some(0));

    return show_list(
        todo_list, 
        &db_filename,
        &workspace_name
    );
}
