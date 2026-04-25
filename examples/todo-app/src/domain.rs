use mssql_orm::prelude::*;

#[derive(AuditFields)]
pub struct TodoAudit {
    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(sql_type = "datetime2")]
    #[orm(updatable = false)]
    pub created_at: String,

    #[orm(column = "created_by_user_id")]
    pub created_by: Option<i64>,

    #[orm(nullable)]
    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(sql_type = "datetime2")]
    pub updated_at: Option<String>,

    #[orm(nullable)]
    #[orm(length = 120)]
    pub updated_by: Option<String>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "todo")]
pub struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(length = 180)]
    #[orm(unique)]
    pub email: String,

    #[orm(length = 120)]
    pub display_name: String,

    #[orm(default_sql = "SYSUTCDATETIME()")]
    pub created_at: String,

    #[orm(rowversion)]
    pub version: Vec<u8>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_lists", schema = "todo")]
#[orm(index(name = "ix_todo_lists_owner_title", columns(owner_user_id, title)))]
pub struct TodoList {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(foreign_key(entity = User, column = id))]
    #[orm(on_delete = "cascade")]
    pub owner_user_id: i64,

    #[orm(length = 160)]
    pub title: String,

    #[orm(nullable)]
    #[orm(length = 500)]
    pub description: Option<String>,

    #[orm(default_sql = "0")]
    pub is_archived: bool,

    #[orm(default_sql = "SYSUTCDATETIME()")]
    pub created_at: String,

    #[orm(rowversion)]
    pub version: Vec<u8>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_items", schema = "todo")]
#[orm(index(name = "ix_todo_items_list_position", columns(list_id, position)))]
pub struct TodoItem {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(foreign_key(entity = TodoList, column = id))]
    #[orm(on_delete = "cascade")]
    pub list_id: i64,

    #[orm(foreign_key(entity = User, column = id))]
    pub created_by_user_id: i64,

    #[orm(nullable)]
    #[orm(foreign_key(entity = User, column = id))]
    pub completed_by_user_id: Option<i64>,

    #[orm(length = 200)]
    pub title: String,

    pub position: i32,

    #[orm(default_sql = "0")]
    pub is_completed: bool,

    #[orm(nullable)]
    pub completed_at: Option<String>,

    #[orm(default_sql = "SYSUTCDATETIME()")]
    pub created_at: String,

    #[orm(rowversion)]
    pub version: Vec<u8>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "audit_events", schema = "todo", audit = TodoAudit)]
pub struct AuditEvent {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(length = 80)]
    pub event_name: String,

    #[orm(length = 200)]
    pub subject: String,
}

#[cfg(test)]
mod tests {
    use super::{AuditEvent, TodoAudit, TodoItem, TodoList, User};
    use mssql_orm::prelude::{Entity, EntityPolicy, ReferentialAction, SqlServerType};

    #[test]
    fn todo_user_metadata_exposes_expected_table_and_columns() {
        let metadata = User::metadata();

        assert_eq!(metadata.schema, "todo");
        assert_eq!(metadata.table, "users");
        assert_eq!(metadata.primary_key.columns, &["id"]);
        assert_eq!(metadata.indexes.len(), 1);
        assert_eq!(metadata.indexes[0].name, "ux_users_email");
        assert_eq!(metadata.indexes[0].columns[0].column_name, "email");
        assert_eq!(
            metadata
                .rowversion_column()
                .expect("todo user rowversion column")
                .column_name,
            "version"
        );
        assert_eq!(
            metadata
                .column("created_at")
                .expect("created_at column")
                .default_sql,
            Some("SYSUTCDATETIME()")
        );
    }

    #[test]
    fn todo_list_metadata_tracks_owner_relationship() {
        let metadata = TodoList::metadata();
        let owner_fk = metadata
            .foreign_key("fk_todo_lists_owner_user_id_users")
            .expect("owner relationship metadata");

        assert_eq!(metadata.schema, "todo");
        assert_eq!(metadata.table, "todo_lists");
        assert_eq!(metadata.foreign_keys.len(), 1);
        assert_eq!(
            metadata
                .column("description")
                .expect("description column")
                .nullable,
            true
        );
        assert_eq!(owner_fk.columns, &["owner_user_id"]);
        assert_eq!(owner_fk.referenced_schema, "todo");
        assert_eq!(owner_fk.referenced_table, "users");
        assert_eq!(owner_fk.referenced_columns, &["id"]);
        assert_eq!(owner_fk.on_delete, ReferentialAction::Cascade);
        assert_eq!(
            metadata.foreign_keys_for_column("owner_user_id")[0].name,
            owner_fk.name
        );
    }

    #[test]
    fn todo_item_metadata_covers_list_and_user_relationships() {
        let metadata = TodoItem::metadata();
        let list_fk = metadata
            .foreign_key("fk_todo_items_list_id_todo_lists")
            .expect("list relationship metadata");
        let created_by_fk = metadata
            .foreign_key("fk_todo_items_created_by_user_id_users")
            .expect("created by relationship metadata");
        let completed_by_fk = metadata
            .foreign_key("fk_todo_items_completed_by_user_id_users")
            .expect("completed by relationship metadata");

        assert_eq!(metadata.foreign_keys.len(), 3);
        assert_eq!(list_fk.columns, &["list_id"]);
        assert_eq!(list_fk.referenced_schema, "todo");
        assert_eq!(list_fk.referenced_table, "todo_lists");
        assert_eq!(list_fk.on_delete, ReferentialAction::Cascade);

        assert_eq!(created_by_fk.columns, &["created_by_user_id"]);
        assert_eq!(created_by_fk.referenced_schema, "todo");
        assert_eq!(created_by_fk.referenced_table, "users");
        assert_eq!(created_by_fk.on_delete, ReferentialAction::NoAction);

        assert_eq!(completed_by_fk.columns, &["completed_by_user_id"]);
        assert_eq!(completed_by_fk.referenced_schema, "todo");
        assert_eq!(completed_by_fk.referenced_table, "users");
        assert_eq!(completed_by_fk.on_delete, ReferentialAction::NoAction);
        assert_eq!(
            metadata
                .column("completed_by_user_id")
                .expect("completed by column")
                .nullable,
            true
        );
        assert_eq!(
            metadata
                .foreign_keys_referencing("todo", "users")
                .iter()
                .map(|foreign_key| foreign_key.name)
                .collect::<Vec<_>>(),
            vec![created_by_fk.name, completed_by_fk.name]
        );
        assert_eq!(metadata.indexes.len(), 1);
        assert_eq!(metadata.indexes[0].name, "ix_todo_items_list_position");
        assert_eq!(
            metadata.indexes[0]
                .columns
                .iter()
                .map(|column| column.column_name)
                .collect::<Vec<_>>(),
            vec!["list_id", "position"]
        );
        assert_eq!(
            TodoItem::completed_by_user_id.column_name(),
            "completed_by_user_id"
        );
    }

    #[test]
    fn audit_event_metadata_expands_reusable_audit_policy_columns() {
        let audit_metadata = TodoAudit::metadata();
        let metadata = AuditEvent::metadata();

        assert_eq!(audit_metadata.name, "audit");
        assert_eq!(
            audit_metadata
                .columns
                .iter()
                .map(|column| column.column_name)
                .collect::<Vec<_>>(),
            vec![
                "created_at",
                "created_by_user_id",
                "updated_at",
                "updated_by"
            ]
        );
        assert_eq!(metadata.schema, "todo");
        assert_eq!(metadata.table, "audit_events");
        assert_eq!(metadata.columns.len(), 7);
        assert_eq!(
            metadata
                .columns
                .iter()
                .map(|column| column.column_name)
                .collect::<Vec<_>>(),
            vec![
                "id",
                "event_name",
                "subject",
                "created_at",
                "created_by_user_id",
                "updated_at",
                "updated_by"
            ]
        );
        assert_eq!(
            metadata
                .column("created_at")
                .expect("created_at audit column")
                .sql_type,
            SqlServerType::DateTime2
        );
        assert_eq!(
            metadata
                .column("created_at")
                .expect("created_at audit column")
                .default_sql,
            Some("SYSUTCDATETIME()")
        );
        assert!(
            metadata
                .column("updated_at")
                .expect("updated_at audit column")
                .nullable
        );
        assert_eq!(
            metadata
                .column("updated_by")
                .expect("updated_by audit column")
                .max_length,
            Some(120)
        );
    }
}
