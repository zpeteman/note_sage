// src/actions.rs
use crate::models::Task; // Add this line

#[derive(Debug)]
pub enum Action {
    Add(Task),
    Delete(Task),
    Done(u32),
    Archive(usize),
}
