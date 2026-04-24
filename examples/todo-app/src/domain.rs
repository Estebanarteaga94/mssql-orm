use mssql_orm::prelude::*;

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
    #[orm(on_delete = "set null")]
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

#[cfg(test)]
mod tests {
    use super::{TodoItem, TodoList, User};
    use mssql_orm::prelude::{Entity, ReferentialAction};

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
        assert_eq!(completed_by_fk.on_delete, ReferentialAction::SetNull);
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
}
