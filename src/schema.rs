// @generated automatically by Diesel CLI.

diesel::table! {
    cats (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        locked_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(cats -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    cats,
    users,
);
