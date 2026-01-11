// src/models.rs
// Shared data structures and application state

use crate::database::{ActivityType, Project, TimeEntry};
use chrono::NaiveDate;

/// Current view/tab in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    TimeTracking,
    ManageProjects,
    ManageActivities,
    DailySummary,
}

impl Default for AppView {
    fn default() -> Self {
        AppView::TimeTracking
    }
}

/// Dialog state for editing/creating items
#[derive(Debug, Clone)]
pub enum DialogState {
    None,
    AddProject,
    EditProject(Project),
    AddActivity(i64), // project_id
    EditActivity(ActivityType),
    AddTimeEntry,
    EditTimeEntry(TimeEntry),
    ConfirmDelete(DeleteTarget),
}

impl Default for DialogState {
    fn default() -> Self {
        DialogState::None
    }
}

/// Target for deletion confirmation
#[derive(Debug, Clone)]
pub enum DeleteTarget {
    Project(i64, String),
    Activity(i64, String),
    TimeEntry(i64),
}

/// Form data for creating/editing a project
#[derive(Debug, Clone, Default)]
pub struct ProjectForm {
    pub name: String,
    pub description: String,
}

impl ProjectForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_project(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            description: project.description.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.description.clear();
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.description.trim().is_empty()
    }
}

/// Form data for creating/editing an activity type
#[derive(Debug, Clone, Default)]
pub struct ActivityForm {
    pub name: String,
    pub project_id: Option<i64>,
}

impl ActivityForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_activity(activity: &ActivityType) -> Self {
        Self {
            name: activity.name.clone(),
            project_id: Some(activity.project_id),
        }
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.project_id = None;
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && self.project_id.is_some()
    }
}

/// Form data for creating/editing a time entry
#[derive(Debug, Clone)]
pub struct TimeEntryForm {
    pub activity_type_id: Option<i64>,
    pub time_str: String,
    pub comment: String,
}

impl Default for TimeEntryForm {
    fn default() -> Self {
        Self {
            activity_type_id: None,
            time_str: "00:30".to_string(),
            comment: String::new(),
        }
    }
}

impl TimeEntryForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_entry(entry: &TimeEntry) -> Self {
        Self {
            activity_type_id: Some(entry.activity_type_id),
            time_str: crate::database::format_minutes_to_time(entry.minutes),
            comment: entry.comment.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.activity_type_id = None;
        self.time_str = "00:30".to_string();
        self.comment.clear();
    }

    pub fn is_valid(&self) -> bool {
        self.activity_type_id.is_some()
            && crate::database::parse_time_to_minutes(&self.time_str).is_ok()
            && !self.comment.trim().is_empty()
    }

    pub fn get_minutes(&self) -> Option<i32> {
        crate::database::parse_time_to_minutes(&self.time_str).ok()
    }
}

/// Cached data for display
#[derive(Debug, Clone, Default)]
pub struct CachedData {
    pub projects: Vec<Project>,
    pub all_activities: Vec<ActivityType>,
    pub current_date_entries: Vec<TimeEntry>,
    pub daily_summary: Vec<crate::database::ActivitySummary>,
    pub summary_date: Option<chrono::NaiveDate>,
    pub needs_refresh: bool,
}

impl CachedData {
    pub fn new() -> Self {
        Self {
            needs_refresh: true,
            ..Default::default()
        }
    }

    pub fn mark_dirty(&mut self) {
        self.needs_refresh = true;
    }

    pub fn get_activity_by_id(&self, id: i64) -> Option<&ActivityType> {
        self.all_activities.iter().find(|a| a.id == id)
    }

    pub fn get_project_by_id(&self, id: i64) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }

    pub fn get_activities_for_project(&self, project_id: i64) -> Vec<&ActivityType> {
        self.all_activities
            .iter()
            .filter(|a| a.project_id == project_id && a.is_active)
            .collect()
    }
}

/// Message/notification to display to user
#[derive(Debug, Clone)]
pub struct UserMessage {
    pub text: String,
    pub is_error: bool,
    pub timestamp: std::time::Instant,
}

impl UserMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed().as_secs() > 5
    }
}

/// Filter state for lists
#[derive(Debug, Clone, Default)]
pub struct FilterState {
    pub show_inactive: bool,
    pub selected_project_id: Option<i64>,
    pub search_text: String,
}

impl FilterState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Date selection state
#[derive(Debug, Clone)]
pub struct DateState {
    pub selected_date: NaiveDate,
}

impl Default for DateState {
    fn default() -> Self {
        Self {
            selected_date: chrono::Local::now().date_naive(),
        }
    }
}

impl DateState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn today(&mut self) {
        self.selected_date = chrono::Local::now().date_naive();
    }

    pub fn previous_day(&mut self) {
        if let Some(new_date) = self.selected_date.pred_opt() {
            self.selected_date = new_date;
        }
    }

    pub fn next_day(&mut self) {
        if let Some(new_date) = self.selected_date.succ_opt() {
            self.selected_date = new_date;
        }
    }
}
