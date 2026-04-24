use mssql_orm::prelude::*;
use mssql_orm::query::{Expr, Predicate};

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "todo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 180)]
    email: String,
}

impl FromRow for User {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            email: row.get_required_typed::<String>("email")?,
        })
    }
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_lists", schema = "todo")]
struct TodoList {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    owner_user_id: i64,
    title: String,
    is_archived: bool,
}

impl FromRow for TodoList {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            owner_user_id: row.get_required_typed::<i64>("owner_user_id")?,
            title: row.get_required_typed::<String>("title")?,
            is_archived: row.get_required_typed::<bool>("is_archived")?,
        })
    }
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_items", schema = "todo")]
struct TodoItem {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    list_id: i64,
    completed_by_user_id: Option<i64>,
    title: String,
    position: i32,
    is_completed: bool,
}

impl FromRow for TodoItem {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            list_id: row.get_required_typed::<i64>("list_id")?,
            completed_by_user_id: row.try_get_typed::<i64>("completed_by_user_id")?,
            title: row.get_required_typed::<String>("title")?,
            position: row.get_required_typed::<i32>("position")?,
            is_completed: row.get_required_typed::<bool>("is_completed")?,
        })
    }
}

#[derive(DbContext, Debug, Clone)]
struct TodoAppDbContext {
    users: DbSet<User>,
    todo_lists: DbSet<TodoList>,
    todo_items: DbSet<TodoItem>,
}

fn accept_item_query(_query: DbSetQuery<TodoItem>) {}
fn accept_list_query(_query: DbSetQuery<TodoList>) {}

async fn count_open_items(db: &TodoAppDbContext) -> Result<i64, OrmError> {
    db.todo_items
        .query()
        .filter(
            TodoItem::list_id
                .eq(9_i64)
                .and(TodoItem::is_completed.eq(false)),
        )
        .count()
        .await
}

fn main() {
    let _build_queries = |db: &TodoAppDbContext| {
        accept_list_query(
            db.todo_lists
                .query()
                .filter(
                    TodoList::owner_user_id
                        .eq(7_i64)
                        .and(TodoList::is_archived.eq(false)),
                )
                .order_by(TodoList::title.asc())
                .paginate(PageRequest::new(2, 10)),
        );

        accept_item_query(
            db.todo_items
                .query()
                .inner_join::<TodoList>(Predicate::eq(
                    Expr::from(TodoItem::list_id),
                    Expr::from(TodoList::id),
                ))
                .left_join::<User>(Predicate::eq(
                    Expr::from(TodoItem::completed_by_user_id),
                    Expr::from(User::id),
                ))
                .filter(
                    TodoList::owner_user_id
                        .eq(7_i64)
                        .and(TodoItem::is_completed.eq(false)),
                )
                .order_by(TodoItem::position.asc())
                .limit(20),
        );

        accept_item_query(
            db.todo_items
                .query()
                .filter(TodoItem::list_id.eq(5_i64))
                .order_by(TodoItem::position.asc())
                .take(5),
        );
    };

    let _count_open_items = count_open_items;
}
