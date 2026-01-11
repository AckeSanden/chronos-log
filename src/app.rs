// src/app.rs
// Main application structure and logic

use crate::database::Database;
use crate::models::*;
use crate::ui;
use eframe::egui;
use std::path::PathBuf;

/// Main application struct
pub struct WorkTrackerApp {
    db: Database,
    current_view: AppView,
    dialog_state: DialogState,
    previous_dialog_state: Option<DialogState>,
    date_state: DateState,
    cache: CachedData,
    filter_state: FilterState,

    // Form data
    project_form: ProjectForm,
    activity_form: ActivityForm,
    entry_form: TimeEntryForm,

    // Messages
    messages: Vec<UserMessage>,
}

impl WorkTrackerApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts for better appearance
        configure_fonts(&cc.egui_ctx);

        // Determine database path
        let db_path = get_database_path();

        // Create database connection
        let db = match Database::new(&db_path) {
            Ok(db) => {
                println!("Database opened at: {:?}", db_path);
                db
            }
            Err(e) => {
                eprintln!("Failed to open database: {}. Using in-memory database.", e);
                Database::new_in_memory().expect("Failed to create in-memory database")
            }
        };

        let mut app = Self {
            db,
            current_view: AppView::default(),
            dialog_state: DialogState::default(),
            previous_dialog_state: None,
            date_state: DateState::default(),
            cache: CachedData::new(),
            filter_state: FilterState::new(),
            project_form: ProjectForm::new(),
            activity_form: ActivityForm::new(),
            entry_form: TimeEntryForm::new(),
            messages: Vec::new(),
        };

        // Initial data load
        app.refresh_cache();

        // Create example data if database is empty
        if app.cache.projects.is_empty() {
            app.create_example_data();
            app.refresh_cache();
        }

        app
    }

    /// Refresh cached data from database
    fn refresh_cache(&mut self) {
        // Load projects
        self.cache.projects = self.db.get_all_projects(false).unwrap_or_default();

        // Load all activities
        self.cache.all_activities = self.db.get_all_activity_types(false).unwrap_or_default();

        // Load entries for current date
        self.cache.current_date_entries = self
            .db
            .get_time_entries_for_date(self.date_state.selected_date)
            .unwrap_or_default();

        self.cache.needs_refresh = false;
    }

    /// Create example data for first run
    fn create_example_data(&mut self) {
        // Create example project
        if let Ok(project_id) = self
            .db
            .create_project("33 - IT-Support", "IT Support activities across locations")
        {
            // Create example activities
            let _ = self
                .db
                .create_activity_type(project_id, "IT-Support - Trollhättan");
            let _ = self
                .db
                .create_activity_type(project_id, "IT-Support - Göteborg");
            let _ = self
                .db
                .create_activity_type(project_id, "IT-Support - Västerås");
            let _ = self
                .db
                .create_activity_type(project_id, "IT-Support - Östersund");
        }

        // Create another example project
        if let Ok(project_id) = self
            .db
            .create_project("40 - Development", "Software development tasks")
        {
            let _ = self
                .db
                .create_activity_type(project_id, "Development - Feature work");
            let _ = self
                .db
                .create_activity_type(project_id, "Development - Bug fixes");
            let _ = self
                .db
                .create_activity_type(project_id, "Development - Code review");
        }

        self.add_message(UserMessage::info("Created example projects and activities"));
    }

    /// Add a message to display
    fn add_message(&mut self, msg: UserMessage) {
        self.messages.push(msg);
    }

    /// Clean up expired messages
    fn cleanup_messages(&mut self) {
        self.messages.retain(|m| !m.is_expired());
    }

    /// Prepare form data when opening dialogs (only on dialog state change)
    fn prepare_dialog_forms_if_changed(&mut self) {
        // Check if dialog state has changed
        let dialog_changed = match (&self.previous_dialog_state, &self.dialog_state) {
            (None, DialogState::None) => false,
            (Some(DialogState::None), DialogState::None) => false,
            (Some(prev), current) => {
                // Compare discriminants to see if dialog type changed
                std::mem::discriminant(prev) != std::mem::discriminant(current)
            }
            _ => true, // Changed from None to something, or first time
        };

        if !dialog_changed {
            return;
        }

        // Prepare forms based on new dialog state
        match &self.dialog_state {
            DialogState::EditProject(project) => {
                self.project_form = ProjectForm::from_project(project);
            }
            DialogState::EditActivity(activity) => {
                self.activity_form = ActivityForm::from_activity(activity);
            }
            DialogState::EditTimeEntry(entry) => {
                self.entry_form = TimeEntryForm::from_entry(entry);
            }
            DialogState::AddProject => {
                self.project_form.clear();
            }
            DialogState::AddActivity(project_id) => {
                self.activity_form.clear();
                self.activity_form.project_id = Some(*project_id);
            }
            _ => {}
        }

        // Update previous state
        self.previous_dialog_state = Some(self.dialog_state.clone());
    }
}

impl eframe::App for WorkTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Refresh cache if needed
        if self.cache.needs_refresh {
            self.refresh_cache();
        }

        // Clean up old messages
        self.cleanup_messages();

        // Prepare form data when dialog state changes (before drawing)
        self.prepare_dialog_forms_if_changed();

        // Draw main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Navigation bar
            ui::draw_nav_bar(ui, &mut self.current_view);

            // Messages area
            if !self.messages.is_empty() {
                for msg in &self.messages {
                    let color = if msg.is_error {
                        egui::Color32::RED
                    } else {
                        egui::Color32::from_rgb(0, 150, 0)
                    };
                    ui.colored_label(color, &msg.text);
                }
                ui.add_space(5.0);
            }

            // Main content based on current view
            match self.current_view {
                AppView::TimeTracking => {
                    ui::draw_time_tracking_view(
                        ui,
                        &mut self.date_state,
                        &mut self.cache,
                        &mut self.dialog_state,
                        &mut self.entry_form,
                        &self.db,
                    );
                }
                AppView::DailySummary => {
                    ui::draw_daily_summary_view(
                        ui,
                        &mut self.date_state,
                        &mut self.cache,
                        &self.db,
                    );
                }
                AppView::ManageProjects => {
                    ui::draw_projects_view(
                        ui,
                        &mut self.cache,
                        &mut self.dialog_state,
                        &mut self.filter_state,
                        &self.db,
                    );
                }
                AppView::ManageActivities => {
                    ui::draw_activities_view(
                        ui,
                        &mut self.cache,
                        &mut self.dialog_state,
                        &mut self.filter_state,
                        &self.db,
                    );
                }
            }
        });

        // Draw dialogs
        ui::draw_dialog(
            ctx,
            &mut self.dialog_state,
            &mut self.project_form,
            &mut self.activity_form,
            &mut self.entry_form,
            &mut self.cache,
            &self.db,
        );
    }
}

/// Configure egui fonts
fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Increase default font size slightly
    style
        .text_styles
        .get_mut(&egui::TextStyle::Body)
        .unwrap()
        .size = 14.0;
    style
        .text_styles
        .get_mut(&egui::TextStyle::Button)
        .unwrap()
        .size = 14.0;
    style
        .text_styles
        .get_mut(&egui::TextStyle::Heading)
        .unwrap()
        .size = 20.0;

    ctx.set_style(style);
}

/// Get the database file path
fn get_database_path() -> PathBuf {
    // Try to use user's data directory
    if let Some(data_dir) = dirs::data_local_dir() {
        let app_dir = data_dir.join("chronos-log");
        if std::fs::create_dir_all(&app_dir).is_ok() {
            return app_dir.join("chronos_log.db");
        }
    }

    // Fallback to current directory
    PathBuf::from("chronos_log.db")
}
