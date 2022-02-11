-- Add down migration script here
DROP FUNCTION manage_updated_at(_tbl regclass);

DROP FUNCTION set_updated_at();
