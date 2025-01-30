mod models;
mod cli;
mod tui;
mod actions;

use clap::Parser;
use models::Task;
use actions::Action;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

const MAX_DATETIME: DateTime<Utc> = DateTime::from_naive_utc_and_offset(NaiveDateTime::MAX, Utc);

struct AppState {
    tasks: Vec<Task>,
    archived_tasks: Vec<Task>,
    last_action: Option<Action>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let (active_tasks, archived_tasks) = models::load_tasks();
    let mut state = AppState {
        tasks: active_tasks,
        archived_tasks,
        last_action: None,
    };

    // Match on the Option<Commands>
    match cli.command {
        Some(cli::Commands::Add {
            description,
            due_date,
            tags,
            priority,
        }) => {
            let due_date_parsed = due_date.map(|d| {
                NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                    .expect("Invalid date format. Use YYYY-MM-DD.")
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            });

            let priority_enum = match priority.expect("Priority must be specified").to_lowercase().as_str() {
                "low" => models::Priority::Low,
                "medium" => models::Priority::Medium,
                "high" => models::Priority::High,
                _ => models::Priority::Low,
            };

            let new_task = Task {
                id: state.tasks.len() as u32 + 1,
                description,
                tags,
                due_date: due_date_parsed,
                priority: priority_enum,
                completed: false,
            };

            state.last_action = Some(Action::Add(new_task.clone()));
            state.tasks.push(new_task);
            models::save_tasks(&state.tasks, &state.archived_tasks)?;
        }

        Some(cli::Commands::List {
            sort_by_due_date,
            tags,
            sort_by_priority,
        }) => {
            let mut tasks = state.tasks.clone();

            if let Some(filter_tags) = &tags {
                tasks.retain(|task| filter_tags.iter().all(|tag| task.tags.contains(tag)));
            }

            if sort_by_due_date {
                tasks.sort_by(|a, b| {
                    a.due_date
                        .unwrap_or(MAX_DATETIME)
                        .cmp(&b.due_date.unwrap_or(MAX_DATETIME))
                });
            }

            if sort_by_priority {
                tasks.sort_by(|a, b| {
                    let a_pri = a.priority.clone() as u8;
                    let b_pri = b.priority.clone() as u8;
                    a_pri.cmp(&b_pri)
                });
            }

            println!("Active tasks:");
            for task in tasks {
                let status = if task.completed { "[✓]" } else { "[ ]" };
                let due_date = task
                    .due_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "No due date".into());
                let overdue = task.due_date.map(|d| d < Utc::now()).unwrap_or(false);

                println!(
                    "{} {}: {} (Due: {}{}, Tags: {:?}, Priority: {:?})",
                    status,
                    task.id,
                    task.description,
                    due_date,
                    if overdue { " (OVERDUE!)" } else { "" },
                    task.tags,
                    task.priority
                );
            }
        }

        Some(cli::Commands::Done { id }) => {
            if let Some(task) = state.tasks.iter_mut().find(|t| t.id == id) {
                task.completed = true;
                state.last_action = Some(Action::Done(id));
                println!("Marked task {} as done", id);
            } else {
                println!("Task with ID {} not found", id);
            }
            models::save_tasks(&state.tasks, &state.archived_tasks)?;
        }

        Some(cli::Commands::Delete { id }) => {
            if let Some(task) = state.tasks.iter().find(|t| t.id == id).cloned() {
                state.last_action = Some(Action::Delete(task.clone()));
                state.tasks.retain(|t| t.id != id);

                // Reassign IDs
                for (idx, task) in state.tasks.iter_mut().enumerate() {
                    task.id = (idx + 1) as u32;
                }

                println!("Deleted task {}", id);
            } else {
                println!("Task with ID {} not found", id);
            }
            models::save_tasks(&state.tasks, &state.archived_tasks)?;
        }

        Some(cli::Commands::Archive) => {
            let mut archived = Vec::new();
            state.tasks.retain(|task| {
                if task.completed {
                    archived.push(task.clone());
                    false
                } else {
                    true
                }
            });

            state.archived_tasks.extend(archived.clone());
            state.last_action = Some(Action::Archive(archived.len()));
            println!("Archived {} tasks", archived.len());

            for (index, task) in state.tasks.iter_mut().enumerate() {
                task.id = (index + 1) as u32;
            }

            for (index, task) in state.archived_tasks.iter_mut().enumerate() {
                task.id = (index + 1) as u32;
            }

            models::save_tasks(&state.tasks, &state.archived_tasks)?;
        }

        Some(cli::Commands::ListArchived) => {
            println!("Archived tasks:");
            for task in &state.archived_tasks {
                let due_date = task
                    .due_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "No due date".into());

                println!(
                    "[✓] {}: {} (Due: {}, Tags: {:?})",
                    task.id, task.description, due_date, task.tags
                );
            }
        }

        Some(cli::Commands::Undo) => {
            if let Some(action) = state.last_action.take() {
                match action {
                    Action::Add(task) => {
                        state.tasks.retain(|t| t.id != task.id);
                        println!("Undone: Removed added task '{}'", task.description);
                    }
                    Action::Delete(task) => {
                        state.tasks.push(task);
                        println!("Undone: Restored deleted task");
                    }
                    Action::Done(id) => {
                        if let Some(task) = state.tasks.iter_mut().find(|t| t.id == id) {
                            task.completed = false;
                            println!("Undone: Marked task {} as incomplete", id);
                        }
                    }
                    Action::Archive(count) => {
                        let mut restored = Vec::new();
                        state.archived_tasks.retain(|task| {
                            if task.completed {
                                restored.push(task.clone());
                                false
                            } else {
                                true
                            }
                        });
                        state.tasks.extend(restored);
                        println!("Undone: Restored {} archived tasks", count);
                    }
                }
                models::save_tasks(&state.tasks, &state.archived_tasks)?;
            } else {
                println!("Nothing to undo!");
            }
        }

        // Handle TUI mode (either via --tui or default)
        Some(cli::Commands::Tui) | None => {
            tui::run_tui(&mut state.tasks, &mut state.archived_tasks)?;
            // Save any changes made in the TUI
            models::save_tasks(&state.tasks, &state.archived_tasks)?;
        }
    }

    Ok(())
}
