use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Tabs, Paragraph, Clear},
    Terminal, layout::{Layout, Constraint, Direction},
    style::{Style, Color},
    text::{Text, Line, Span},
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode}
};
use crate::models::{Task, Priority};
use chrono::NaiveDate;

#[derive(PartialEq)]
enum TabMode {
    Active,
    Archived,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    AddingTask(AddTaskState),
}

#[derive(PartialEq)]
enum AddTaskState {
    Description,
    Tags,
    DueDate,
    Priority,
}


pub fn run_tui(tasks: &mut Vec<Task>, archived: &mut Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // clear the terminal
    terminal.clear()?;

    let mut selected_tab = TabMode::Active;
    let mut selected_index = 0;
    let mut input_mode = InputMode::Normal;

    // Task creation state
    let mut new_task_description = String::new();
    let mut new_task_tags = String::new();
    let mut new_task_due_date = String::new();
    let mut new_task_priority = Priority::Low;

    loop {
        let (current_list, list_len) = match selected_tab {
            TabMode::Active => (&tasks, tasks.len()),
            TabMode::Archived => (&archived, archived.len()),
        };

        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(f.size());

            // Tabs
            let tabs = Tabs::new(vec!["Active", "Archived"])
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(match selected_tab {
                    TabMode::Active => 0,
                    TabMode::Archived => 1,
                });
            f.render_widget(tabs, main_chunks[0]);

            // Content
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(main_chunks[1]);

            // Task List
            let items: Vec<ListItem> = current_list
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    let style = if i == selected_index {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!(
                        "{} [{}] {}",
                        if task.completed { "✓" } else { " " },
                        task.id,
                        task.description
                    )).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Tasks"));
            f.render_widget(list, content_chunks[0]);

            // Task Details
            if let Some(task) = current_list.get(selected_index) {
                let mut details = Vec::new();
                
                if let Some(due_date) = &task.due_date {
                    details.push(Line::from(vec![
                        Span::styled("Due: ", Style::default().fg(Color::Magenta)),
                        Span::raw(due_date.format("%Y-%m-%d %H:%M").to_string()),
                    ]));
                }

                if !task.tags.is_empty() {
                    details.push(Line::from(vec![
                        Span::styled("Tags: ", Style::default().fg(Color::Cyan)),
                        Span::raw(task.tags.join(", ")),
                    ]));
                }

                details.push(Line::from(vec![
                    Span::styled("Priority: ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("{:?}", task.priority)),
                ]));

                details.push(Line::from(vec![
                    Span::styled("Description:\n", Style::default().fg(Color::Green)),
                    Span::raw(&task.description),
                ]));

                let details_block = Paragraph::new(details)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(ratatui::widgets::Wrap { trim: true });
                f.render_widget(details_block, content_chunks[1]);
            }

            // Add Task Form
            if let InputMode::AddingTask(state) = &input_mode {
                let form_content = match state {
                    AddTaskState::Description => vec![
                        Line::from("Add New Task (Press Enter to skip)"),
                        Line::from("Description:"),
                        Line::from(new_task_description.as_str()),
                    ],
                    AddTaskState::Tags => vec![
                        Line::from("Add Tags (comma separated, Enter to skip):"),
                        Line::from(new_task_tags.as_str()),
                    ],
                    AddTaskState::DueDate => vec![
                        Line::from("Due Date (YYYY-MM-DD, Enter to skip):"),
                        Line::from(new_task_due_date.as_str()),
                    ],
                    AddTaskState::Priority => vec![
                        Line::from("Priority (1=Low, 2=Medium, 3=High):"),
                        Line::from(format!("{:?}", new_task_priority)),
                    ],
                };

                let popup = Paragraph::new(Text::from(form_content))
                    .block(Block::default().borders(Borders::ALL).title("New Task"))
                    .style(Style::default().bg(Color::DarkGray));

                let area = Layout::default()
                    .constraints([Constraint::Percentage(50)])
                    .split(f.size())[0];

                f.render_widget(Clear, area);
                f.render_widget(popup, area);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match &mut input_mode {
                InputMode::Normal => match key.code {
                    // Navigation
                    KeyCode::Char('j') | KeyCode::Down => {
                        selected_index = (selected_index + 1).min(list_len.saturating_sub(1));
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        selected_index = selected_index.saturating_sub(1);
                    }
                    KeyCode::Char('h') => {
                        selected_tab = TabMode::Active;
                        selected_index = 0;
                    }
                    KeyCode::Char('l') => {
                        selected_tab = TabMode::Archived;
                        selected_index = 0;
                    }

                    // Task Management
                    KeyCode::Char('d') => { // Delete
                        if let TabMode::Active = selected_tab {
                            if selected_index < tasks.len() {
                                tasks.remove(selected_index);
                                selected_index = selected_index.saturating_sub(1);
                                for (index, task) in tasks.iter_mut().enumerate() {                 
                                  task.id = (index + 1) as u32;                                     
                                }                                                                   
                            }
                        }
                    }
                    KeyCode::Char('D') => { // Mark as Done
                        if let TabMode::Active = selected_tab {
                            if let Some(task) = tasks.get_mut(selected_index) {
                                task.completed = true;
                            }
                        }
                    }
                    KeyCode::Char('a') => { // Add Task
                        input_mode = InputMode::AddingTask(AddTaskState::Description);
                        new_task_description.clear();
                        new_task_tags.clear();
                        new_task_due_date.clear();
                        new_task_priority = Priority::Low;
                    }

                    KeyCode::Char('r') => {
                      let mut new_archived = Vec::new();
                      tasks.retain(|task| {
                        if task.completed {
                          new_archived.push(task.clone());
                          false
                        } else {
                          true
                        }
                      });
                      archived.extend(new_archived.clone());
                      for (index, task) in tasks.iter_mut().enumerate() {          
                        task.id = (index + 1) as u32;                                  
                      }                                                                  
                      for (index, task) in archived.iter_mut().enumerate() { 
                        task.id = (index + 1) as u32;                                  
                      }                                             
                    }

                    // Quit
                    KeyCode::Char('q') => break,
                    _ => {}
                },

                InputMode::AddingTask(state) => match key.code {
                    KeyCode::Enter => {
                        match state {
                            AddTaskState::Description => {
                                input_mode = InputMode::AddingTask(AddTaskState::Tags);
                            }
                            AddTaskState::Tags => {
                                input_mode = InputMode::AddingTask(AddTaskState::DueDate);
                            }
                            AddTaskState::DueDate => {
                                input_mode = InputMode::AddingTask(AddTaskState::Priority);
                            }
                            AddTaskState::Priority => {
                                // Create new task
                                let new_task = Task {
                                    id: tasks.len() as u32 + 1,
                                    description: new_task_description.clone(),
                                    tags: new_task_tags.split(',').map(|s| s.trim().to_string()).collect(),
                                    due_date: NaiveDate::parse_from_str(&new_task_due_date, "%Y-%m-%d")
                                        .ok()
                                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
                                    priority: new_task_priority.clone(),
                                    completed: false,
                                };
                                tasks.push(new_task);
                                input_mode = InputMode::Normal;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('1') if *state == AddTaskState::Priority => {
                        new_task_priority = Priority::Low;
                    }
                    KeyCode::Char('2') if *state == AddTaskState::Priority => {
                        new_task_priority = Priority::Medium;
                    }
                    KeyCode::Char('3') if *state == AddTaskState::Priority => {
                        new_task_priority = Priority::High;
                    }
                    KeyCode::Char(c) => match state {
                        AddTaskState::Description => new_task_description.push(c),
                        AddTaskState::Tags => new_task_tags.push(c),
                        AddTaskState::DueDate => new_task_due_date.push(c),
                        _ => {}
                    },
                    KeyCode::Backspace => match state {
                        AddTaskState::Description => { new_task_description.pop(); }
                        AddTaskState::Tags => { new_task_tags.pop(); }
                        AddTaskState::DueDate => { new_task_due_date.pop(); }
                        _ => {}
                    },
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
