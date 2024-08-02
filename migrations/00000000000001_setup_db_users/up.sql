/*
This migration creates two types of users and grant them the proper role:
- one with read-write privileges
- one with read-only privileges
*/

CREATE OR REPLACE FUNCTION create_role(role_name text) RETURNS VOID AS $$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_roles WHERE rolname=role_name)
  THEN
    EXECUTE format('CREATE ROLE %I WITH INHERIT', role_name);
  END IF;
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_user(user_name text, user_passw text) RETURNS VOID AS $$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_user WHERE usename=user_name)
  THEN
    EXECUTE format('CREATE USER %I WITH PASSWORD %L', user_name, user_passw);
  END IF;
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION grant_role_to_user(role_name text, user_name text) RETURNS VOID AS $$
BEGIN
  IF EXISTS (SELECT * FROM pg_roles WHERE rolname=role_name)
    AND EXISTS (SELECT * FROM pg_roles WHERE rolname=user_name)
  THEN
    EXECUTE format('GRANT %I TO %I', role_name, user_name);
  END IF;
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION grant_read_write_privileges_to_role(role_name text) RETURNS VOID AS $$
BEGIN
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT EXECUTE ON FUNCTIONS TO %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO %I', role_name);
  EXECUTE format('GRANT USAGE ON SCHEMA public TO %I', role_name);
  EXECUTE format('REVOKE ALL ON ALL TABLES IN SCHEMA public FROM %I', role_name);
  EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO %I', role_name);
  EXECUTE format('GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO %I', role_name);
  EXECUTE format('GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO %I', role_name);
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION grant_read_only_privileges_to_role(role_name text) RETURNS VOID AS $$
BEGIN
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT EXECUTE ON FUNCTIONS TO %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON SEQUENCES TO %I', role_name);
  EXECUTE format('GRANT USAGE ON SCHEMA public TO %I', role_name);
  EXECUTE format('REVOKE ALL ON ALL TABLES IN SCHEMA public FROM %I', role_name);
  EXECUTE format('GRANT SELECT ON ALL TABLES IN SCHEMA public TO %I', role_name);
  EXECUTE format('GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO %I', role_name);
  EXECUTE format('GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO %I', role_name);
END
$$ LANGUAGE plpgsql;

SELECT create_role('DB_USER_rw');
SELECT create_role('DB_USER_ro');

SELECT grant_read_write_privileges_to_role('DB_USER_rw');
SELECT grant_read_only_privileges_to_role('DB_USER_ro');

SELECT create_user('DB_USER', 'DB_USER_PASS_RW');
SELECT create_user('DB_USER_reader', 'DB_USER_PASS_RO');

SELECT grant_role_to_user('DB_USER_rw', 'DB_USER');
SELECT grant_role_to_user('DB_USER_ro', 'DB_USER_reader');
