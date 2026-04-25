IF NOT EXISTS (SELECT 1 FROM sys.schemas WHERE name = 'todo')
BEGIN
    EXEC('CREATE SCHEMA [todo]');
END;
GO

IF OBJECT_ID('todo.todo_items', 'U') IS NOT NULL
BEGIN
    DROP TABLE todo.todo_items;
END;
GO

IF OBJECT_ID('todo.audit_events', 'U') IS NOT NULL
BEGIN
    DROP TABLE todo.audit_events;
END;
GO

IF OBJECT_ID('todo.todo_lists', 'U') IS NOT NULL
BEGIN
    DROP TABLE todo.todo_lists;
END;
GO

IF OBJECT_ID('todo.users', 'U') IS NOT NULL
BEGIN
    DROP TABLE todo.users;
END;
GO

CREATE TABLE todo.users (
    id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    email NVARCHAR(180) NOT NULL UNIQUE,
    display_name NVARCHAR(120) NOT NULL,
    created_at NVARCHAR(64) NOT NULL,
    version ROWVERSION NOT NULL
);
GO

CREATE TABLE todo.todo_lists (
    id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    owner_user_id BIGINT NOT NULL,
    title NVARCHAR(160) NOT NULL,
    is_archived BIT NOT NULL CONSTRAINT df_todo_lists_is_archived DEFAULT 0,
    created_at NVARCHAR(64) NOT NULL,
    version ROWVERSION NOT NULL,
    CONSTRAINT fk_todo_lists_owner_user_id_users
        FOREIGN KEY (owner_user_id) REFERENCES todo.users(id) ON DELETE CASCADE
);
GO

CREATE TABLE todo.todo_items (
    id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    list_id BIGINT NOT NULL,
    created_by_user_id BIGINT NOT NULL,
    completed_by_user_id BIGINT NULL,
    title NVARCHAR(200) NOT NULL,
    position INT NOT NULL,
    is_completed BIT NOT NULL CONSTRAINT df_todo_items_is_completed DEFAULT 0,
    completed_at NVARCHAR(64) NULL,
    created_at NVARCHAR(64) NOT NULL,
    version ROWVERSION NOT NULL,
    CONSTRAINT fk_todo_items_list_id_todo_lists
        FOREIGN KEY (list_id) REFERENCES todo.todo_lists(id) ON DELETE CASCADE,
    CONSTRAINT fk_todo_items_created_by_user_id_users
        FOREIGN KEY (created_by_user_id) REFERENCES todo.users(id),
    -- SQL Server rechaza este caso como "multiple cascade paths" si se combina
    -- con users -> todo_lists ON DELETE CASCADE y todo_lists -> todo_items ON DELETE CASCADE.
    -- Para el smoke test del ejemplo solo necesitamos lectura, así que el fixture
    -- operativo usa NO ACTION aquí.
    CONSTRAINT fk_todo_items_completed_by_user_id_users
        FOREIGN KEY (completed_by_user_id) REFERENCES todo.users(id)
);
GO

CREATE INDEX ix_todo_lists_owner_title ON todo.todo_lists(owner_user_id, title);
GO

CREATE INDEX ix_todo_items_list_position ON todo.todo_items(list_id, position);
GO

CREATE TABLE todo.audit_events (
    id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    event_name NVARCHAR(80) NOT NULL,
    subject NVARCHAR(200) NOT NULL,
    created_at DATETIME2 NOT NULL CONSTRAINT df_audit_events_created_at DEFAULT SYSUTCDATETIME(),
    created_by_user_id BIGINT NULL,
    updated_at DATETIME2 NULL CONSTRAINT df_audit_events_updated_at DEFAULT SYSUTCDATETIME(),
    updated_by NVARCHAR(120) NULL
);
GO

SET IDENTITY_INSERT todo.users ON;
INSERT INTO todo.users (id, email, display_name, created_at)
VALUES
    (7, 'ana@example.com', 'Ana', '2026-04-23T00:00:00'),
    (8, 'reviewer@example.com', 'Reviewer', '2026-04-23T00:00:00');
SET IDENTITY_INSERT todo.users OFF;
GO

SET IDENTITY_INSERT todo.todo_lists ON;
INSERT INTO todo.todo_lists (id, owner_user_id, title, is_archived, created_at)
VALUES
    (10, 7, 'Inbox', 0, '2026-04-23T00:00:00'),
    (11, 7, 'Archived', 1, '2026-04-23T00:00:00');
SET IDENTITY_INSERT todo.todo_lists OFF;
GO

SET IDENTITY_INSERT todo.todo_items ON;
INSERT INTO todo.todo_items (
    id, list_id, created_by_user_id, completed_by_user_id, title, position, is_completed, completed_at, created_at
)
VALUES
    (100, 10, 7, NULL, 'Ship release', 1, 0, NULL, '2026-04-23T00:00:00'),
    (101, 10, 7, 7, 'Write docs', 2, 1, '2026-04-23T01:00:00', '2026-04-23T00:00:00'),
    (102, 10, 7, NULL, 'Review PR', 3, 0, NULL, '2026-04-23T00:00:00');
SET IDENTITY_INSERT todo.todo_items OFF;
GO
