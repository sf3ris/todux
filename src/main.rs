pub mod database;
pub mod command;
pub mod utils;
pub mod todo;
pub mod workspace;

use todo::{TodoData, Todo};
use structopt::StructOpt;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use std::io::{self, Read};
use std::error::Error;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, List, ListItem};
use termion::{event::Key, raw::IntoRawMode, input::TermRead};

use utils::{StatefulList};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt, Debug)]
enum Cli {
    Add {
        todo_name: String
    },
    List, 
    Workspace(WorkspaceCommand)
}

#[derive(StructOpt, Debug)]
enum WorkspaceCommand {
    Set {
        workspace_name: String
    },
    Unset
}

struct TodoList {
    items: StatefulList<Todo>
}

impl<'a> TodoList {
    fn new(data: TodoData) -> TodoList {
        TodoList {
            items: StatefulList::with_items(data.todos)
        }
    }
}

fn get_db_filename_from_workspace_name(workspace_name: String) -> String {
    let mut db_filename = String::from("db.").to_owned();
    db_filename.push_str(&workspace_name);
    db_filename.push_str(".json");

    db_filename
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut db_filename = match workspace::get_workspace() {
        Ok(workspace_name) => get_db_filename_from_workspace_name(workspace_name),
        _ => String::from("db.json")
    };


    match StructOpt::from_args() {
        Cli::Workspace(ws_command) => {
            match ws_command {
                WorkspaceCommand::Set { workspace_name} => {
                    workspace::set_workspace(&workspace_name);
                    db_filename = get_db_filename_from_workspace_name(workspace_name);
                },
                WorkspaceCommand::Unset => {
                    workspace::unset_workspace();
                    db_filename = String::from("db.json");
                }
            }
        },
        Cli::Add {todo_name} => {
            command::add(todo_name, &db_filename);
            return Ok(());
        }
        _ => println!("Continuing to list")
    }

    let stdout = io::stdout().into_raw_mode().expect("asd");
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("asd");
    let mut asi = termion::async_stdin();

    let db = database::read(&db_filename);
    let mut todo_list = TodoList::new(db);
    todo_list.items.state.select(Some(0));

    terminal.clear().expect("Error clearing terminal");
    loop {
        terminal.draw(|f| {
            // Iterate through all elements in the `items` app and append some debug text to it.
            let items: Vec<ListItem> = todo_list
                .items
                .items
                .iter()
                .map(|i| {
                    let content = Span::from(String::from(i.title.as_str()));
                    let checkbox = match i.done {
                        true => Span::from("[x] "),
                        false => Span::from("[ ] ")
                    };
                    let spans = Spans::from(vec![checkbox, content]);
                    ListItem::new(spans).style(Style::default().fg(Color::Black).bg(Color::White))
                })
                .collect();

            // Create a List from all list items and highlight the currently selected one
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("TODO"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(items, f.size(), &mut todo_list.items.state);
        })?;

        for k in asi.by_ref().keys() {
            match k.unwrap() {
                Key::Char('q') => {
                    terminal.clear()?;
                    // Save current todo list before exit
                    database::save(&TodoData {
                        todos: todo_list.items.items
                    }, &db_filename);
                    return Ok(());
                },
                Key::Up => {
                    todo_list.items.previous()
                },
                Key::Down => {
                    todo_list.items.next()
                },
                Key::Char('t') => {
                    &todo_list.items.items[todo_list.items.state.selected().unwrap()].toggle();
                },
                Key::Char('d') => {
                    todo_list.items.remove()
                }
                _ => (),
            }
        }
    }
}
