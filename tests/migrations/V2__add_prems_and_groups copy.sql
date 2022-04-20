CREATE TABLE IF NOT EXISTS permissions (
    permission_id int GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    permission_name text NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    is_deleted boolean NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS permission_unique_name ON permissions (permission_name);

CREATE TABLE IF NOT EXISTS groups (
    group_id int GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    group_name text NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    is_deleted boolean NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS group_unique_name ON groups (group_name);

CREATE TABLE IF NOT EXISTS group_permissions (
    permission_id int NOT NULL,
    group_id int NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    is_deleted boolean NOT NULL,

    CONSTRAINT fk_permission FOREIGN KEY(permission_id) REFERENCES permissions(permission_id),
    CONSTRAINT fk_group FOREIGN KEY(group_id) REFERENCES groups(group_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS group_permission_unique_binding ON group_permissions (group_id, permission_id);

-- auth service own roles:
INSERT INTO groups (group_name, created_at, updated_at, is_deleted)
VALUES 
('ROLE_AUTH_ADMIN', now(), now(), FALSE),
('ROLE_AUTH_MANAGER', now(), now(), FALSE),
('ROLE_AUTH_STAFF', now(), now(), FALSE)
ON CONFLICT DO NOTHING;
