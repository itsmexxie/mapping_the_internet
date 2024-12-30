// @generated automatically by Diesel CLI.

diesel::table! {
    AddressAllocationStates (id) {
        #[max_length = 16]
        id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    Addresses (id) {
        id -> Inet,
        #[max_length = 16]
        allocation_state_id -> Varchar,
        #[max_length = 255]
        allocation_state_comment -> Nullable<Varchar>,
        routed -> Bool,
        online -> Bool,
        #[max_length = 16]
        top_rir_id -> Nullable<Varchar>,
        #[max_length = 16]
        rir_id -> Nullable<Varchar>,
        asn_id -> Nullable<Int4>,
        ports -> Array<Nullable<Int4>>,
        #[max_length = 3]
        country -> Nullable<Varchar>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    Asns (id) {
        id -> Int4,
    }
}

diesel::table! {
    Rirs (id) {
        #[max_length = 16]
        id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    ServiceUnits (id) {
        #[max_length = 36]
        id -> Varchar,
        service_id -> Int4,
        #[max_length = 255]
        address -> Nullable<Varchar>,
        port -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    Services (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        max_one -> Bool,
    }
}

diesel::joinable!(Addresses -> AddressAllocationStates (allocation_state_id));
diesel::joinable!(Addresses -> Asns (asn_id));

diesel::allow_tables_to_appear_in_same_query!(
    AddressAllocationStates,
    Addresses,
    Asns,
    Rirs,
    ServiceUnits,
    Services,
);
