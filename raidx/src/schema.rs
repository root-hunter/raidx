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
    messages_incoming (id) {
        id -> Integer,
        uid -> Text,
        message_type -> Text,
        data -> Nullable<Binary>,
        from -> Text,
        created_at -> Integer,
    }
}

diesel::table! {
    messages_outgoing (id) {
        id -> Integer,
        uid -> Text,
        message_type -> Text,
        data -> Nullable<Binary>,
        to -> Text,
        created_at -> Integer,
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
diesel::joinable!(messages_incoming -> nodes (from));
diesel::joinable!(messages_outgoing -> nodes (to));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    messages_incoming,
    messages_outgoing,
    nodes,
);
