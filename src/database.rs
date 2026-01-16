// src/database.rs
// All database operations for the work tracker application

use chrono::NaiveDate;
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Project not found: {0}")]
    ProjectNotFound(i64),
    #[error("Activity not found: {0}")]
    ActivityNotFound(i64),
    #[error("Invalid time format")]
    InvalidTimeFormat,
    #[error("Activity has time entries and cannot be deleted")]
    ActivityHasEntries,
    #[error("Project has activities and cannot be deleted")]
    ProjectHasActivities,
}

pub type DbResult<T> = Result<T, DatabaseError>;

/// Represents a project in the database
#[derive(Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_active: bool,
}

/// Represents an activity type linked to a project
#[derive(Debug, Clone)]
pub struct ActivityType {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub is_active: bool,
}

/// Represents a time entry for an activity
#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub id: i64,
    pub activity_type_id: i64,
    #[allow(dead_code)]
    pub date: NaiveDate,
    pub minutes: i32,
    pub comment: String,
}

/// Summary of time spent on an activity type for a specific day
#[derive(Debug, Clone)]
pub struct ActivitySummary {
    pub activity_type_id: i64,
    pub activity_name: String,
    pub project_name: String,
    pub total_minutes: i32,
    pub entries: Vec<TimeEntry>,
}

/// Database manager handling all database operations
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize tables
    pub fn new<P: AsRef<Path>>(path: P) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.initialize_tables()?;
        Ok(db)
    }

    /// Create an in-memory database (useful for testing)
    #[allow(dead_code)]
    pub fn new_in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn };
        db.initialize_tables()?;
        Ok(db)
    }

    /// Initialize all database tables
    fn initialize_tables(&self) -> DbResult<()> {
        self.conn.execute_batch(
            r#"
            -- Projects table
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT DEFAULT '',
                is_active INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Activity types table (linked to projects)
            CREATE TABLE IF NOT EXISTS activity_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                is_active INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                UNIQUE(project_id, name)
            );

            -- Time entries table
            CREATE TABLE IF NOT EXISTS time_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                activity_type_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                minutes INTEGER NOT NULL,
                comment TEXT DEFAULT '',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (activity_type_id) REFERENCES activity_types(id) ON DELETE CASCADE
            );

            -- Index for faster date queries
            CREATE INDEX IF NOT EXISTS idx_time_entries_date ON time_entries(date);
            CREATE INDEX IF NOT EXISTS idx_time_entries_activity ON time_entries(activity_type_id);
            "#,
        )?;
        Ok(())
    }

    // ==================== Project Operations ====================

    /// Create a new project
    pub fn create_project(&self, name: &str, description: &str) -> DbResult<i64> {
        self.conn.execute(
            "INSERT INTO projects (name, description) VALUES (?1, ?2)",
            params![name, description],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all projects (optionally only active ones)
    pub fn get_all_projects(&self, only_active: bool) -> DbResult<Vec<Project>> {
        let sql = if only_active {
            "SELECT id, name, description, is_active FROM projects WHERE is_active = 1 ORDER BY name"
        } else {
            "SELECT id, name, description, is_active FROM projects ORDER BY name"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    is_active: row.get::<_, i32>(3)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    }

    /// Get a single project by ID
    #[allow(dead_code)]
    pub fn get_project(&self, id: i64) -> DbResult<Project> {
        self.conn
            .query_row(
                "SELECT id, name, description, is_active FROM projects WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        is_active: row.get::<_, i32>(3)? == 1,
                    })
                },
            )
            .map_err(|_| DatabaseError::ProjectNotFound(id))
    }

    /// Update a project
    pub fn update_project(&self, id: i64, name: &str, description: &str) -> DbResult<()> {
        let rows = self.conn.execute(
            "UPDATE projects SET name = ?1, description = ?2 WHERE id = ?3",
            params![name, description, id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::ProjectNotFound(id));
        }
        Ok(())
    }

    /// Deactivate a project (soft delete)
    pub fn deactivate_project(&self, id: i64) -> DbResult<()> {
        let rows = self.conn.execute(
            "UPDATE projects SET is_active = 0 WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::ProjectNotFound(id));
        }
        Ok(())
    }

    /// Reactivate a project
    pub fn reactivate_project(&self, id: i64) -> DbResult<()> {
        let rows = self.conn.execute(
            "UPDATE projects SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::ProjectNotFound(id));
        }
        Ok(())
    }

    /// Delete a project permanently (only if no activities exist)
    pub fn delete_project(&self, id: i64) -> DbResult<()> {
        // Check if project has any activities
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM activity_types WHERE project_id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Err(DatabaseError::ProjectHasActivities);
        }

        let rows = self
            .conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DatabaseError::ProjectNotFound(id));
        }
        Ok(())
    }

    // ==================== Activity Type Operations ====================

    /// Create a new activity type for a project
    pub fn create_activity_type(&self, project_id: i64, name: &str) -> DbResult<i64> {
        self.conn.execute(
            "INSERT INTO activity_types (project_id, name) VALUES (?1, ?2)",
            params![project_id, name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all activity types for a project
    #[allow(dead_code)]
    pub fn get_activity_types_for_project(
        &self,
        project_id: i64,
        only_active: bool,
    ) -> DbResult<Vec<ActivityType>> {
        let sql = if only_active {
            "SELECT id, project_id, name, is_active FROM activity_types
             WHERE project_id = ?1 AND is_active = 1 ORDER BY name"
        } else {
            "SELECT id, project_id, name, is_active FROM activity_types
             WHERE project_id = ?1 ORDER BY name"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let activities = stmt
            .query_map(params![project_id], |row| {
                Ok(ActivityType {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    is_active: row.get::<_, i32>(3)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(activities)
    }

    /// Get all activity types (optionally only active ones)
    pub fn get_all_activity_types(&self, only_active: bool) -> DbResult<Vec<ActivityType>> {
        let sql = if only_active {
            "SELECT id, project_id, name, is_active FROM activity_types
             WHERE is_active = 1 ORDER BY name"
        } else {
            "SELECT id, project_id, name, is_active FROM activity_types ORDER BY name"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let activities = stmt
            .query_map([], |row| {
                Ok(ActivityType {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    is_active: row.get::<_, i32>(3)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(activities)
    }

    /// Get a single activity type by ID
    #[allow(dead_code)]
    pub fn get_activity_type(&self, id: i64) -> DbResult<ActivityType> {
        self.conn
            .query_row(
                "SELECT id, project_id, name, is_active FROM activity_types WHERE id = ?1",
                params![id],
                |row| {
                    Ok(ActivityType {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        name: row.get(2)?,
                        is_active: row.get::<_, i32>(3)? == 1,
                    })
                },
            )
            .map_err(|_| DatabaseError::ActivityNotFound(id))
    }

    /// Update an activity type
    pub fn update_activity_type(&self, id: i64, name: &str) -> DbResult<()> {
        let rows = self.conn.execute(
            "UPDATE activity_types SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::ActivityNotFound(id));
        }
        Ok(())
    }

    /// Deactivate an activity type (soft delete)
    pub fn deactivate_activity_type(&self, id: i64) -> DbResult<()> {
        let rows = self.conn.execute(
            "UPDATE activity_types SET is_active = 0 WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::ActivityNotFound(id));
        }
        Ok(())
    }

    /// Reactivate an activity type
    pub fn reactivate_activity_type(&self, id: i64) -> DbResult<()> {
        let rows = self.conn.execute(
            "UPDATE activity_types SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::ActivityNotFound(id));
        }
        Ok(())
    }

    /// Delete an activity type permanently (only if no time entries exist)
    pub fn delete_activity_type(&self, id: i64) -> DbResult<()> {
        // Check if activity has any time entries
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM time_entries WHERE activity_type_id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Err(DatabaseError::ActivityHasEntries);
        }

        let rows = self
            .conn
            .execute("DELETE FROM activity_types WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DatabaseError::ActivityNotFound(id));
        }
        Ok(())
    }

    // ==================== Time Entry Operations ====================

    /// Create a new time entry
    pub fn create_time_entry(
        &self,
        activity_type_id: i64,
        date: NaiveDate,
        minutes: i32,
        comment: &str,
    ) -> DbResult<i64> {
        self.conn.execute(
            "INSERT INTO time_entries (activity_type_id, date, minutes, comment)
             VALUES (?1, ?2, ?3, ?4)",
            params![activity_type_id, date.to_string(), minutes, comment],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all time entries for a specific date
    pub fn get_time_entries_for_date(&self, date: NaiveDate) -> DbResult<Vec<TimeEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, activity_type_id, date, minutes, comment
             FROM time_entries WHERE date = ?1 ORDER BY id",
        )?;
        let entries = stmt
            .query_map(params![date.to_string()], |row| {
                let date_str: String = row.get(2)?;
                Ok(TimeEntry {
                    id: row.get(0)?,
                    activity_type_id: row.get(1)?,
                    date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
                    minutes: row.get(3)?,
                    comment: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Get time entries for a date range
    #[allow(dead_code)]
    pub fn get_time_entries_for_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> DbResult<Vec<TimeEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, activity_type_id, date, minutes, comment
             FROM time_entries WHERE date >= ?1 AND date <= ?2 ORDER BY date, id",
        )?;
        let entries = stmt
            .query_map(
                params![start_date.to_string(), end_date.to_string()],
                |row| {
                    let date_str: String = row.get(2)?;
                    Ok(TimeEntry {
                        id: row.get(0)?,
                        activity_type_id: row.get(1)?,
                        date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
                        minutes: row.get(3)?,
                        comment: row.get(4)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Update a time entry
    pub fn update_time_entry(&self, id: i64, minutes: i32, comment: &str) -> DbResult<()> {
        self.conn.execute(
            "UPDATE time_entries SET minutes = ?1, comment = ?2 WHERE id = ?3",
            params![minutes, comment, id],
        )?;
        Ok(())
    }

    /// Delete a time entry
    pub fn delete_time_entry(&self, id: i64) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM time_entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== Summary Operations ====================

    /// Get activity summaries for a specific date (total time per activity)
    pub fn get_daily_summary(&self, date: NaiveDate) -> DbResult<Vec<ActivitySummary>> {
        // First get all entries for the date with activity and project info
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                at.id as activity_type_id,
                at.name as activity_name,
                p.name as project_name,
                te.id as entry_id,
                te.minutes,
                te.comment
            FROM time_entries te
            JOIN activity_types at ON te.activity_type_id = at.id
            JOIN projects p ON at.project_id = p.id
            WHERE te.date = ?1
            ORDER BY p.name, at.name, te.id
            "#,
        )?;

        let rows: Vec<(i64, String, String, i64, i32, String)> = stmt
            .query_map(params![date.to_string()], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Group by activity type
        let mut summaries: Vec<ActivitySummary> = Vec::new();
        for (activity_type_id, activity_name, project_name, entry_id, minutes, comment) in rows {
            if let Some(summary) = summaries
                .iter_mut()
                .find(|s| s.activity_type_id == activity_type_id)
            {
                summary.total_minutes += minutes;
                summary.entries.push(TimeEntry {
                    id: entry_id,
                    activity_type_id,
                    date,
                    minutes,
                    comment,
                });
            } else {
                summaries.push(ActivitySummary {
                    activity_type_id,
                    activity_name,
                    project_name,
                    total_minutes: minutes,
                    entries: vec![TimeEntry {
                        id: entry_id,
                        activity_type_id,
                        date,
                        minutes,
                        comment,
                    }],
                });
            }
        }

        Ok(summaries)
    }

    /// Get total time for a specific date
    #[allow(dead_code)]
    pub fn get_total_time_for_date(&self, date: NaiveDate) -> DbResult<i32> {
        let total: i32 = self.conn.query_row(
            "SELECT COALESCE(SUM(minutes), 0) FROM time_entries WHERE date = ?1",
            params![date.to_string()],
            |row| row.get(0),
        )?;
        Ok(total)
    }
}

// ==================== Utility Functions ====================

/// Parse time string in format "HH:MM" to minutes
pub fn parse_time_to_minutes(time_str: &str) -> Result<i32, DatabaseError> {
    let parts: Vec<&str> = time_str.trim().split(':').collect();
    if parts.len() != 2 {
        return Err(DatabaseError::InvalidTimeFormat);
    }

    let hours: i32 = parts[0]
        .parse()
        .map_err(|_| DatabaseError::InvalidTimeFormat)?;
    let minutes: i32 = parts[1]
        .parse()
        .map_err(|_| DatabaseError::InvalidTimeFormat)?;

    if hours < 0 || minutes < 0 || minutes >= 60 {
        return Err(DatabaseError::InvalidTimeFormat);
    }

    Ok(hours * 60 + minutes)
}

/// Format minutes to "HH:MM" string
pub fn format_minutes_to_time(total_minutes: i32) -> String {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{:02}:{:02}", hours, minutes)
}

/// Format minutes to decimal hours with Swedish comma separator (e.g., "1,5" for 90 minutes)
pub fn format_minutes_to_decimal(total_minutes: i32) -> String {
    let hours = total_minutes as f64 / 60.0;
    // Format with 2 decimal places and replace . with ,
    format!("{:.2}", hours).replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_to_minutes() {
        assert_eq!(parse_time_to_minutes("00:30").unwrap(), 30);
        assert_eq!(parse_time_to_minutes("01:00").unwrap(), 60);
        assert_eq!(parse_time_to_minutes("02:30").unwrap(), 150);
        assert!(parse_time_to_minutes("invalid").is_err());
    }

    #[test]
    fn test_format_minutes_to_time() {
        assert_eq!(format_minutes_to_time(30), "00:30");
        assert_eq!(format_minutes_to_time(60), "01:00");
        assert_eq!(format_minutes_to_time(150), "02:30");
    }

    #[test]
    fn test_format_minutes_to_decimal() {
        assert_eq!(format_minutes_to_decimal(30), "0,50");
        assert_eq!(format_minutes_to_decimal(60), "1,00");
        assert_eq!(format_minutes_to_decimal(90), "1,50");
        assert_eq!(format_minutes_to_decimal(150), "2,50");
        assert_eq!(format_minutes_to_decimal(480), "8,00");
        assert_eq!(format_minutes_to_decimal(15), "0,25");
        assert_eq!(format_minutes_to_decimal(45), "0,75");
    }

    #[test]
    fn test_database_operations() {
        let db = Database::new_in_memory().unwrap();

        // Create project
        let project_id = db.create_project("Test Project", "Description").unwrap();
        assert!(project_id > 0);

        // Create activity type
        let activity_id = db
            .create_activity_type(project_id, "Test Activity")
            .unwrap();
        assert!(activity_id > 0);

        // Create time entry
        let today = chrono::Local::now().date_naive();
        let entry_id = db
            .create_time_entry(activity_id, today, 30, "Test comment")
            .unwrap();
        assert!(entry_id > 0);

        // Get summary
        let summaries = db.get_daily_summary(today).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].total_minutes, 30);
    }
}
