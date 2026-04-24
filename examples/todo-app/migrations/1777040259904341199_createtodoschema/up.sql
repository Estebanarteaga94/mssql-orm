IF SCHEMA_ID(N'todo') IS NULL EXEC(N'CREATE SCHEMA [todo]');

CREATE TABLE [todo].[todo_items] (
    [id] bigint IDENTITY(1, 1) NOT NULL,
    [list_id] bigint NOT NULL,
    [created_by_user_id] bigint NOT NULL,
    [completed_by_user_id] bigint NULL,
    [title] nvarchar(200) NOT NULL,
    [position] int NOT NULL,
    [is_completed] bit NOT NULL DEFAULT 0,
    [completed_at] nvarchar(255) NULL,
    [created_at] nvarchar(255) NOT NULL DEFAULT SYSUTCDATETIME(),
    [version] rowversion,
    PRIMARY KEY ([id])
);

CREATE TABLE [todo].[todo_lists] (
    [id] bigint IDENTITY(1, 1) NOT NULL,
    [owner_user_id] bigint NOT NULL,
    [title] nvarchar(160) NOT NULL,
    [description] nvarchar(500) NULL,
    [is_archived] bit NOT NULL DEFAULT 0,
    [created_at] nvarchar(255) NOT NULL DEFAULT SYSUTCDATETIME(),
    [version] rowversion,
    PRIMARY KEY ([id])
);

CREATE TABLE [todo].[users] (
    [id] bigint IDENTITY(1, 1) NOT NULL,
    [email] nvarchar(180) NOT NULL,
    [display_name] nvarchar(120) NOT NULL,
    [created_at] nvarchar(255) NOT NULL DEFAULT SYSUTCDATETIME(),
    [version] rowversion,
    PRIMARY KEY ([id])
);

CREATE INDEX [ix_todo_items_list_position] ON [todo].[todo_items] ([list_id] ASC, [position] ASC);

ALTER TABLE [todo].[todo_items] ADD CONSTRAINT [fk_todo_items_list_id_todo_lists] FOREIGN KEY ([list_id]) REFERENCES [todo].[todo_lists] ([id]) ON DELETE CASCADE ON UPDATE NO ACTION;

ALTER TABLE [todo].[todo_items] ADD CONSTRAINT [fk_todo_items_created_by_user_id_users] FOREIGN KEY ([created_by_user_id]) REFERENCES [todo].[users] ([id]) ON DELETE NO ACTION ON UPDATE NO ACTION;

ALTER TABLE [todo].[todo_items] ADD CONSTRAINT [fk_todo_items_completed_by_user_id_users] FOREIGN KEY ([completed_by_user_id]) REFERENCES [todo].[users] ([id]) ON DELETE NO ACTION ON UPDATE NO ACTION;

CREATE INDEX [ix_todo_lists_owner_title] ON [todo].[todo_lists] ([owner_user_id] ASC, [title] ASC);

ALTER TABLE [todo].[todo_lists] ADD CONSTRAINT [fk_todo_lists_owner_user_id_users] FOREIGN KEY ([owner_user_id]) REFERENCES [todo].[users] ([id]) ON DELETE CASCADE ON UPDATE NO ACTION;

CREATE UNIQUE INDEX [ux_users_email] ON [todo].[users] ([email] ASC);
