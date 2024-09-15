// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Integer,
        uid -> Text,
        node -> Text,
        folder -> Text,
        filename -> Text,
        size -> Integer,
        status -> Text,
        sync -> Bool,
        created_at -> Integer,
        modified_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    nodes (uid) {
        uid -> Text,
        host -> Text,
        port -> Integer,
        local -> Bool,
    }
}

diesel::joinable!(files -> nodes (node));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    nodes,
);
