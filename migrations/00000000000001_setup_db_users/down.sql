-- This file should undo anything in `up.sql`

CREATE OR REPLACE FUNCTION revoke_all_privileges_to_role(role_name text) RETURNS VOID AS $$
BEGIN
  EXECUTE format('REVOKE ALL ON ALL TABLES IN SCHEMA public FROM %I', role_name);
  EXECUTE format('REVOKE ALL ON ALL FUNCTIONS IN SCHEMA public FROM %I', role_name);
  EXECUTE format('REVOKE ALL ON ALL SEQUENCES IN SCHEMA public FROM %I', role_name);
  EXECUTE format('REVOKE ALL ON SCHEMA public FROM %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON TABLES FROM %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON FUNCTIONS FROM %I', role_name);
  EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON SEQUENCES FROM %I', role_name);
END
$$ LANGUAGE plpgsql;

SELECT revoke_all_privileges_to_role('DB_USER_rw');
SELECT revoke_all_privileges_to_role('DB_USER_ro');

DROP ROLE DB_USER_rw;
DROP ROLE DB_USER_ro;

DROP USER IF EXISTS DB_USER;
DROP USER IF EXISTS DB_USER_reader;

DROP FUNCTION create_role(text);
DROP FUNCTION grant_read_only_privileges_to_role(text);
DROP FUNCTION grant_read_write_privileges_to_role(text);
DROP FUNCTION grant_role_to_user(text, text);
DROP FUNCTION revoke_all_privileges_to_role(text);
