// @generated automatically by Diesel CLI.

diesel::table! {
    clients (id) {
        id -> Uuid,
        #[max_length = 255]
        client_id -> Varchar,
        #[max_length = 255]
        client_secret -> Varchar,
        #[max_length = 255]
        redirect_uri -> Varchar,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        activated_at -> Nullable<Timestamptz>,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    clients,
    users,
);
