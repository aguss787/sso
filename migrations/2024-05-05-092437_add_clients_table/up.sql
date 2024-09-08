-- Your SQL goes here
CREATE TABLE clients
(
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id     VARCHAR(255) NOT NULL,
    client_secret VARCHAR(255) NOT NULL,
    redirect_uri  VARCHAR(255) NOT NULL,
    created_at    TIMESTAMP        DEFAULT now(),
    updated_at    TIMESTAMP        DEFAULT now()
);

CREATE TRIGGER set_clients_updated_at
    BEFORE UPDATE
    ON clients
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
