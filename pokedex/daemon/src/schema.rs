// @generated automatically by Diesel CLI.

diesel::table! {
    AddressServers (id) {
        id -> Int4,
        address_id -> Inet,
        server_id -> Int4,
        port -> Int4,
    }
}

diesel::table! {
    Addresses (id) {
        id -> Inet,
        state_id -> Int4,
        routed -> Bool,
        online -> Bool,
        rir_id -> Int4,
        asn_id -> Int4,
    }
}

diesel::table! {
    Asns (id) {
        id -> Int4,
    }
}

diesel::table! {
    Rirs (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    Servers (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
    }
}

diesel::table! {
    ServiceUnits (id) {
        #[max_length = 36]
        id -> Varchar,
        service_id -> Int4,
        address -> Nullable<Inet>,
        port -> Nullable<Int4>,
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

diesel::joinable!(AddressServers -> Addresses (address_id));
diesel::joinable!(AddressServers -> Servers (server_id));
diesel::joinable!(Addresses -> Asns (asn_id));
diesel::joinable!(Addresses -> Rirs (rir_id));

diesel::allow_tables_to_appear_in_same_query!(
    AddressServers,
    Addresses,
    Asns,
    Rirs,
    Servers,
    ServiceUnits,
    Services,
);
