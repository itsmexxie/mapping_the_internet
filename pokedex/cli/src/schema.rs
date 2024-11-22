// @generated automatically by Diesel CLI.

diesel::table! {
    AddressTypes (id) {
        id -> Int4,
        address_id -> Int4,
        type_id -> Int4,
    }
}

diesel::table! {
    Addresses (id) {
        id -> Int4,
        assigned -> Bool,
        routed -> Bool,
        online -> Bool,
    }
}

diesel::table! {
    Services (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password -> Varchar,
    }
}

diesel::table! {
    Types (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
    }
}

diesel::joinable!(AddressTypes -> Addresses (address_id));
diesel::joinable!(AddressTypes -> Types (type_id));

diesel::allow_tables_to_appear_in_same_query!(
    AddressTypes,
    Addresses,
    Services,
    Types,
);
