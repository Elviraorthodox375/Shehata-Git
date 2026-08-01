//! User-controlled retention for the local, redacted activity history.

use shehata_storage::{queries, Database};

use crate::{Result, ShehataError};

pub fn delete_event(db: &Database, id: i64) -> Result<bool> {
    if id <= 0 {
        return Err(ShehataError::InvalidInput(
            "activity event id must be positive".to_string(),
        ));
    }
    Ok(queries::delete_audit_event(db, id)? == 1)
}

pub fn clear_history(db: &Database) -> Result<usize> {
    Ok(queries::clear_audit_events(db)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shehata_storage::NewAuditEvent;

    #[test]
    fn rejects_invalid_event_id() {
        let db = Database::open_in_memory().unwrap();
        assert!(delete_event(&db, 0).is_err());
    }

    #[test]
    fn deletes_one_or_all_events() {
        let db = Database::open_in_memory().unwrap();
        for summary in ["One", "Two"] {
            queries::insert_audit_event(
                &db,
                &NewAuditEvent {
                    event_type: "test",
                    repository_id: None,
                    account_login: None,
                    summary,
                    result: "success",
                    exit_code: Some(0),
                    duration_ms: None,
                },
            )
            .unwrap();
        }
        let events = queries::list_audit_events(&db, 10).unwrap();
        assert!(delete_event(&db, events[0].id).unwrap());
        assert_eq!(clear_history(&db).unwrap(), 1);
    }
}
