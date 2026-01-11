// src/ui.rs
// GUI components and rendering functions

use crate::database::{format_minutes_to_time, ActivitySummary, Database};
use crate::models::*;
use egui::{Align, Color32, Layout, RichText, Ui, Vec2};

/// Draw the main navigation bar
pub fn draw_nav_bar(ui: &mut Ui, current_view: &mut AppView) {
    ui.horizontal(|ui| {
        ui.selectable_value(current_view, AppView::TimeTracking, "⏱ Time Tracking");
        ui.selectable_value(current_view, AppView::DailySummary, "📊 Daily Summary");
        ui.separator();
        ui.selectable_value(current_view, AppView::ManageProjects, "📁 Projects");
        ui.selectable_value(current_view, AppView::ManageActivities, "📋 Activities");
    });
    ui.separator();
}

/// Draw the date selector
pub fn draw_date_selector(ui: &mut Ui, date_state: &mut DateState, cache: &mut CachedData) {
    ui.horizontal(|ui| {
        if ui.button("◀ Previous").clicked() {
            date_state.previous_day();
            cache.mark_dirty();
        }

        ui.label(
            RichText::new(date_state.selected_date.format("%A, %Y-%m-%d").to_string())
                .size(18.0)
                .strong(),
        );

        if ui.button("Next ▶").clicked() {
            date_state.next_day();
            cache.mark_dirty();
        }

        ui.separator();

        if ui.button("📅 Today").clicked() {
            date_state.today();
            cache.mark_dirty();
        }
    });
}

/// Draw the time tracking view
pub fn draw_time_tracking_view(
    ui: &mut Ui,
    date_state: &mut DateState,
    cache: &mut CachedData,
    dialog: &mut DialogState,
    entry_form: &mut TimeEntryForm,
    db: &Database,
) {
    draw_date_selector(ui, date_state, cache);
    ui.add_space(10.0);

    // Quick add section
    ui.group(|ui| {
        ui.heading("Add Time Entry");
        ui.horizontal(|ui| {
            ui.label("Project/Activity:");

            // Activity selector as combo box
            let activity_label = entry_form
                .activity_type_id
                .and_then(|id| cache.get_activity_by_id(id))
                .map(|a| {
                    let project = cache.get_project_by_id(a.project_id);
                    format!(
                        "{} - {}",
                        project.map(|p| p.name.as_str()).unwrap_or("Unknown"),
                        a.name
                    )
                })
                .unwrap_or_else(|| "Select activity...".to_string());

            egui::ComboBox::from_id_salt("activity_select")
                .selected_text(activity_label)
                .width(300.0)
                .show_ui(ui, |ui| {
                    for project in &cache.projects {
                        if !project.is_active {
                            continue;
                        }
                        let activities = cache.get_activities_for_project(project.id);
                        if activities.is_empty() {
                            continue;
                        }

                        ui.label(RichText::new(&project.name).strong());
                        for activity in activities {
                            ui.selectable_value(
                                &mut entry_form.activity_type_id,
                                Some(activity.id),
                                format!("  {}", activity.name),
                            );
                        }
                        ui.separator();
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Time (HH:MM):");
            ui.add(egui::TextEdit::singleline(&mut entry_form.time_str).desired_width(60.0));

            // Quick time buttons
            if ui.button("+15m").clicked() {
                add_time_to_form(entry_form, 15);
            }
            if ui.button("+30m").clicked() {
                add_time_to_form(entry_form, 30);
            }
            if ui.button("-15m").clicked() {
                add_time_to_form(entry_form, -15);
            }
        });

        let mut submit_entry = false;

        ui.horizontal(|ui| {
            ui.label("Comment *:");
            let comment_response = ui.add(
                egui::TextEdit::singleline(&mut entry_form.comment)
                    .desired_width(400.0)
                    .hint_text("What did you do? (required)"),
            );

            // Check if Enter was pressed in the comment field
            if comment_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if entry_form.is_valid() {
                    submit_entry = true;
                }
            }
        });

        ui.horizontal(|ui| {
            let can_add = entry_form.is_valid();
            if ui
                .add_enabled(can_add, egui::Button::new("➕ Add Entry"))
                .clicked()
            {
                submit_entry = true;
            }

            if ui.button("Clear").clicked() {
                entry_form.clear();
            }
        });

        // Handle submission (either from button or Enter key)
        if submit_entry {
            if let (Some(activity_id), Some(minutes)) =
                (entry_form.activity_type_id, entry_form.get_minutes())
            {
                if let Err(e) = db.create_time_entry(
                    activity_id,
                    date_state.selected_date,
                    minutes,
                    &entry_form.comment,
                ) {
                    eprintln!("Error creating entry: {}", e);
                } else {
                    entry_form.comment.clear();
                    entry_form.time_str = "00:30".to_string();
                    cache.mark_dirty();
                }
            }
        }
    });

    ui.add_space(10.0);

    // Today's entries
    ui.heading("Today's Entries");

    if cache.current_date_entries.is_empty() {
        ui.label("No entries for this date yet.");
    } else {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let mut entry_to_edit: Option<crate::database::TimeEntry> = None;

                for entry in &cache.current_date_entries {
                    let activity = cache.get_activity_by_id(entry.activity_type_id);
                    let project = activity.and_then(|a| cache.get_project_by_id(a.project_id));

                    ui.horizontal(|ui| {
                        // Time
                        ui.label(
                            RichText::new(format_minutes_to_time(entry.minutes))
                                .monospace()
                                .strong(),
                        );

                        // Activity and project
                        ui.label(format!(
                            "{} - {}",
                            project.map(|p| p.name.as_str()).unwrap_or("?"),
                            activity.map(|a| a.name.as_str()).unwrap_or("?"),
                        ));

                        // Comment
                        if !entry.comment.is_empty() {
                            ui.label(format!("\"{}\"", entry.comment));
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("🗑").clicked() {
                                *dialog =
                                    DialogState::ConfirmDelete(DeleteTarget::TimeEntry(entry.id));
                            }
                            if ui.small_button("✏").clicked() {
                                entry_to_edit = Some(entry.clone());
                            }
                        });
                    });
                    ui.separator();
                }

                // Handle edit
                if let Some(entry) = entry_to_edit {
                    *dialog = DialogState::EditTimeEntry(entry);
                }
            });
    }

    // Total for the day
    let total_minutes: i32 = cache.current_date_entries.iter().map(|e| e.minutes).sum();
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Total:").strong());
        ui.label(
            RichText::new(format_minutes_to_time(total_minutes))
                .size(18.0)
                .strong()
                .color(Color32::from_rgb(0, 150, 0)),
        );
    });
}

fn add_time_to_form(form: &mut TimeEntryForm, minutes_to_add: i32) {
    if let Ok(current) = crate::database::parse_time_to_minutes(&form.time_str) {
        let new_minutes = (current + minutes_to_add).max(0);
        form.time_str = format_minutes_to_time(new_minutes);
    }
}

/// Draw the daily summary view
pub fn draw_daily_summary_view(
    ui: &mut Ui,
    date_state: &mut DateState,
    cache: &mut CachedData,
    db: &Database,
) {
    draw_date_selector(ui, date_state, cache);
    ui.add_space(10.0);

    ui.heading("Daily Summary");
    ui.label("Total time per activity (for entering into time management system):");
    ui.add_space(10.0);

    let summaries = db
        .get_daily_summary(date_state.selected_date)
        .unwrap_or_default();

    if summaries.is_empty() {
        ui.label("No entries for this date.");
        return;
    }

    // Group by project
    let mut by_project: std::collections::HashMap<String, Vec<&ActivitySummary>> =
        std::collections::HashMap::new();
    for summary in &summaries {
        by_project
            .entry(summary.project_name.clone())
            .or_default()
            .push(summary);
    }

    let mut total_day_minutes = 0;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (project_name, activities) in by_project.iter() {
            ui.group(|ui| {
                ui.heading(project_name);

                for summary in activities {
                    total_day_minutes += summary.total_minutes;

                    ui.horizontal(|ui| {
                        // Activity name
                        ui.label(RichText::new(&summary.activity_name).strong());

                        // Total time
                        ui.label(
                            RichText::new(format_minutes_to_time(summary.total_minutes))
                                .monospace()
                                .color(Color32::from_rgb(0, 100, 200)),
                        );

                        // Copy button
                        if ui.small_button("📋 Copy").clicked() {
                            ui.output_mut(|o| {
                                o.copied_text = format_minutes_to_time(summary.total_minutes);
                            });
                        }
                    });

                    // Show individual entries
                    ui.indent(summary.activity_type_id, |ui| {
                        for entry in &summary.entries {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format_minutes_to_time(entry.minutes))
                                        .small()
                                        .monospace(),
                                );
                                if !entry.comment.is_empty() {
                                    ui.label(RichText::new(&entry.comment).small().italics());
                                }
                            });
                        }
                    });

                    ui.add_space(5.0);
                }
            });
            ui.add_space(5.0);
        }

        // Grand total
        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new("TOTAL FOR DAY:").strong().size(16.0));
            ui.label(
                RichText::new(format_minutes_to_time(total_day_minutes))
                    .strong()
                    .size(18.0)
                    .color(Color32::from_rgb(0, 150, 0)),
            );
        });
    });
}

/// Draw the projects management view
pub fn draw_projects_view(
    ui: &mut Ui,
    cache: &mut CachedData,
    dialog: &mut DialogState,
    filter: &mut FilterState,
    db: &Database,
) {
    ui.horizontal(|ui| {
        ui.heading("Manage Projects");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("➕ New Project").clicked() {
                *dialog = DialogState::AddProject;
            }
        });
    });

    ui.checkbox(&mut filter.show_inactive, "Show inactive projects");
    ui.add_space(10.0);

    // Clone the data we need to avoid borrow issues
    let projects: Vec<_> = cache
        .projects
        .iter()
        .filter(|p| filter.show_inactive || p.is_active)
        .cloned()
        .collect();

    // Track actions to perform after iteration
    let mut action_deactivate: Option<i64> = None;
    let mut action_activate: Option<i64> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for project in &projects {
            // Count activities for this project
            let activity_count = cache
                .all_activities
                .iter()
                .filter(|a| a.project_id == project.id && a.is_active)
                .count();

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    if project.is_active {
                        ui.label(RichText::new("●").color(Color32::GREEN));
                    } else {
                        ui.label(RichText::new("●").color(Color32::GRAY));
                    }

                    // Project name
                    ui.label(RichText::new(&project.name).strong());

                    // Description
                    if !project.description.is_empty() {
                        ui.label(format!("- {}", project.description));
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Delete button
                        if ui.small_button("🗑").clicked() {
                            *dialog = DialogState::ConfirmDelete(DeleteTarget::Project(
                                project.id,
                                project.name.clone(),
                            ));
                        }

                        // Edit button
                        if ui.small_button("✏").clicked() {
                            *dialog = DialogState::EditProject(project.clone());
                        }

                        // Activate/Deactivate
                        if project.is_active {
                            if ui.small_button("Deactivate").clicked() {
                                action_deactivate = Some(project.id);
                            }
                        } else {
                            if ui.small_button("Activate").clicked() {
                                action_activate = Some(project.id);
                            }
                        }

                        // Add activity button
                        if ui.small_button("+ Activity").clicked() {
                            *dialog = DialogState::AddActivity(project.id);
                        }
                    });
                });

                // Show activities count
                ui.label(format!("Activities: {}", activity_count));
            });
        }
    });

    // Execute deferred actions
    if let Some(id) = action_deactivate {
        if let Err(e) = db.deactivate_project(id) {
            eprintln!("Error: {}", e);
        }
        cache.mark_dirty();
    }
    if let Some(id) = action_activate {
        if let Err(e) = db.reactivate_project(id) {
            eprintln!("Error: {}", e);
        }
        cache.mark_dirty();
    }
}

/// Draw the activities management view
pub fn draw_activities_view(
    ui: &mut Ui,
    cache: &mut CachedData,
    dialog: &mut DialogState,
    filter: &mut FilterState,
    db: &Database,
) {
    ui.horizontal(|ui| {
        ui.heading("Manage Activities");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let has_projects = !cache.projects.is_empty();
            if ui
                .add_enabled(has_projects, egui::Button::new("➕ New Activity"))
                .clicked()
            {
                if let Some(project) = cache.projects.first() {
                    *dialog = DialogState::AddActivity(project.id);
                }
            }
        });
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut filter.show_inactive, "Show inactive");

        ui.separator();

        ui.label("Filter by project:");
        egui::ComboBox::from_id_salt("project_filter")
            .selected_text(
                filter
                    .selected_project_id
                    .and_then(|id| cache.get_project_by_id(id))
                    .map(|p| p.name.as_str())
                    .unwrap_or("All projects"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filter.selected_project_id, None, "All projects");
                for project in &cache.projects {
                    ui.selectable_value(
                        &mut filter.selected_project_id,
                        Some(project.id),
                        &project.name,
                    );
                }
            });
    });

    ui.add_space(10.0);

    // Clone the data we need to avoid borrow issues
    let activities: Vec<_> = cache
        .all_activities
        .iter()
        .filter(|a| {
            (filter.show_inactive || a.is_active)
                && filter
                    .selected_project_id
                    .map(|pid| a.project_id == pid)
                    .unwrap_or(true)
        })
        .cloned()
        .collect();

    // Pre-fetch project names
    let project_names: std::collections::HashMap<i64, String> = cache
        .projects
        .iter()
        .map(|p| (p.id, p.name.clone()))
        .collect();

    // Track actions to perform after iteration
    let mut action_deactivate: Option<i64> = None;
    let mut action_activate: Option<i64> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for activity in &activities {
            let project_name = project_names
                .get(&activity.project_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    if activity.is_active {
                        ui.label(RichText::new("●").color(Color32::GREEN));
                    } else {
                        ui.label(RichText::new("●").color(Color32::GRAY));
                    }

                    // Activity name
                    ui.label(RichText::new(&activity.name).strong());

                    // Project name
                    ui.label(format!("({})", project_name));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Delete button
                        if ui.small_button("🗑").clicked() {
                            *dialog = DialogState::ConfirmDelete(DeleteTarget::Activity(
                                activity.id,
                                activity.name.clone(),
                            ));
                        }

                        // Edit button
                        if ui.small_button("✏").clicked() {
                            *dialog = DialogState::EditActivity(activity.clone());
                        }

                        // Activate/Deactivate
                        if activity.is_active {
                            if ui.small_button("Deactivate").clicked() {
                                action_deactivate = Some(activity.id);
                            }
                        } else {
                            if ui.small_button("Activate").clicked() {
                                action_activate = Some(activity.id);
                            }
                        }
                    });
                });
            });
        }
    });

    // Execute deferred actions
    if let Some(id) = action_deactivate {
        if let Err(e) = db.deactivate_activity_type(id) {
            eprintln!("Error: {}", e);
        }
        cache.mark_dirty();
    }
    if let Some(id) = action_activate {
        if let Err(e) = db.reactivate_activity_type(id) {
            eprintln!("Error: {}", e);
        }
        cache.mark_dirty();
    }
}

/// Draw dialogs
pub fn draw_dialog(
    ctx: &egui::Context,
    dialog: &mut DialogState,
    project_form: &mut ProjectForm,
    activity_form: &mut ActivityForm,
    entry_form: &mut TimeEntryForm,
    cache: &mut CachedData,
    db: &Database,
) {
    let mut should_close = false;

    match dialog.clone() {
        DialogState::None => {}

        DialogState::AddProject => {
            egui::Window::new("New Project")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name *:");
                        ui.text_edit_singleline(&mut project_form.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Description *:");
                        ui.add(
                            egui::TextEdit::singleline(&mut project_form.description)
                                .hint_text("(required)"),
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                            project_form.clear();
                        }

                        let can_save = project_form.is_valid();
                        if ui
                            .add_enabled(can_save, egui::Button::new("Create"))
                            .clicked()
                        {
                            if let Err(e) = db.create_project(
                                project_form.name.trim(),
                                project_form.description.trim(),
                            ) {
                                eprintln!("Error creating project: {}", e);
                            } else {
                                cache.mark_dirty();
                                should_close = true;
                                project_form.clear();
                            }
                        }
                    });
                });
        }

        DialogState::EditProject(project) => {
            egui::Window::new("Edit Project")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name *:");
                        ui.text_edit_singleline(&mut project_form.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Description *:");
                        ui.add(
                            egui::TextEdit::singleline(&mut project_form.description)
                                .hint_text("(required)"),
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                            project_form.clear();
                        }

                        let can_save = project_form.is_valid();
                        if ui
                            .add_enabled(can_save, egui::Button::new("Save"))
                            .clicked()
                        {
                            if let Err(e) = db.update_project(
                                project.id,
                                project_form.name.trim(),
                                project_form.description.trim(),
                            ) {
                                eprintln!("Error updating project: {}", e);
                            } else {
                                cache.mark_dirty();
                                should_close = true;
                                project_form.clear();
                            }
                        }
                    });
                });
        }

        DialogState::AddActivity(project_id) => {
            activity_form.project_id = Some(project_id);

            egui::Window::new("New Activity")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Project:");
                        egui::ComboBox::from_id_salt("project_select_dialog")
                            .selected_text(
                                activity_form
                                    .project_id
                                    .and_then(|id| cache.get_project_by_id(id))
                                    .map(|p| p.name.as_str())
                                    .unwrap_or("Select..."),
                            )
                            .show_ui(ui, |ui| {
                                for project in &cache.projects {
                                    if project.is_active {
                                        ui.selectable_value(
                                            &mut activity_form.project_id,
                                            Some(project.id),
                                            &project.name,
                                        );
                                    }
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut activity_form.name);
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                            activity_form.clear();
                        }

                        let can_save = activity_form.is_valid();
                        if ui
                            .add_enabled(can_save, egui::Button::new("Create"))
                            .clicked()
                        {
                            if let Some(pid) = activity_form.project_id {
                                if let Err(e) =
                                    db.create_activity_type(pid, activity_form.name.trim())
                                {
                                    eprintln!("Error creating activity: {}", e);
                                } else {
                                    cache.mark_dirty();
                                    should_close = true;
                                    activity_form.clear();
                                }
                            }
                        }
                    });
                });
        }

        DialogState::EditActivity(activity) => {
            egui::Window::new("Edit Activity")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut activity_form.name);
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                            activity_form.clear();
                        }

                        let can_save = !activity_form.name.trim().is_empty();
                        if ui
                            .add_enabled(can_save, egui::Button::new("Save"))
                            .clicked()
                        {
                            if let Err(e) =
                                db.update_activity_type(activity.id, activity_form.name.trim())
                            {
                                eprintln!("Error updating activity: {}", e);
                            } else {
                                cache.mark_dirty();
                                should_close = true;
                                activity_form.clear();
                            }
                        }
                    });
                });
        }

        DialogState::EditTimeEntry(entry) => {
            egui::Window::new("Edit Time Entry")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .default_width(400.0)
                .show(ctx, |ui| {
                    // Show which activity this entry is for
                    if let Some(activity) = cache.get_activity_by_id(entry.activity_type_id) {
                        if let Some(project) = cache.get_project_by_id(activity.project_id) {
                            ui.label(
                                RichText::new(format!("{} - {}", project.name, activity.name))
                                    .strong(),
                            );
                            ui.add_space(5.0);
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.label("Time (HH:MM):");
                        ui.add(
                            egui::TextEdit::singleline(&mut entry_form.time_str)
                                .desired_width(80.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Comment *:");
                        ui.add(
                            egui::TextEdit::singleline(&mut entry_form.comment)
                                .desired_width(300.0)
                                .hint_text("(required)"),
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                            entry_form.clear();
                        }

                        let can_save = entry_form.is_valid();
                        if ui
                            .add_enabled(can_save, egui::Button::new("Save"))
                            .clicked()
                        {
                            if let Some(minutes) = entry_form.get_minutes() {
                                if let Err(e) =
                                    db.update_time_entry(entry.id, minutes, &entry_form.comment)
                                {
                                    eprintln!("Error updating entry: {}", e);
                                } else {
                                    cache.mark_dirty();
                                    should_close = true;
                                    entry_form.clear();
                                }
                            }
                        }
                    });
                });
        }

        DialogState::ConfirmDelete(target) => {
            let (title, message) = match &target {
                DeleteTarget::Project(_, name) => (
                    "Delete Project?",
                    format!(
                        "Are you sure you want to permanently delete '{}'?\n\
                         This will also delete all activities and time entries!",
                        name
                    ),
                ),
                DeleteTarget::Activity(_, name) => (
                    "Delete Activity?",
                    format!(
                        "Are you sure you want to permanently delete '{}'?\n\
                         This will also delete all time entries!",
                        name
                    ),
                ),
                DeleteTarget::TimeEntry(_) => (
                    "Delete Entry?",
                    "Are you sure you want to delete this time entry?".to_string(),
                ),
            };

            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(message);
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }

                        if ui
                            .button(RichText::new("Delete").color(Color32::RED))
                            .clicked()
                        {
                            let result = match &target {
                                DeleteTarget::Project(id, _) => db.delete_project(*id),
                                DeleteTarget::Activity(id, _) => db.delete_activity_type(*id),
                                DeleteTarget::TimeEntry(id) => db.delete_time_entry(*id),
                            };

                            if let Err(e) = result {
                                eprintln!("Error deleting: {}", e);
                            } else {
                                cache.mark_dirty();
                            }
                            should_close = true;
                        }
                    });
                });
        }

        DialogState::AddTimeEntry => {
            // Not used directly, time entry is added via the main form
            should_close = true;
        }
    }

    if should_close {
        *dialog = DialogState::None;
    }
}
