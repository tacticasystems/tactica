//! Diesel schema definitions for the database tables.

use diesel::table;

table! {
    refresh_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        expires_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

table! {
    refresh_tokens (token_hash) {
        token_hash -> Text,
        session_id -> Uuid,
        consumed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Uuid,

        // identification
        username -> Text,
        email -> Text,

        // profile
        display_name -> Nullable<Text>,
        icon_url -> Nullable<Text>,
        banner_url -> Nullable<Text>,
        biography -> Nullable<Text>,

        // permissions
        is_active -> Bool,
        is_superuser -> Bool,

        // credentials
        password_hash -> Nullable<Text>,
        totp_secret -> Nullable<Text>,

        // audit
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    units (id) {
        id -> Uuid,

        // identification
        slug -> Text,

        // profile
        display_name -> Nullable<Text>,
        icon_url -> Nullable<Text>,
        banner_url -> Nullable<Text>,
        biography -> Nullable<Text>,
        owner_id -> Uuid,

        // audit
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    unit_ranks (id) {
        id -> Uuid,

        // foreign keys
        unit_id -> Uuid,

        // identification
        slug -> Text,

        // profile
        display_name -> Nullable<Text>,
        icon_url -> Nullable<Text>,
        description -> Nullable<Text>,

        position -> BigInt,

        // audit
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    unit_memberships (id) {
        id -> Uuid,

        // foreign keys
        user_id -> Uuid,
        unit_id -> Uuid,
        rank_id -> Uuid,

        // audit
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        display_name -> Nullable<Text>,
    }
}

table! {
    unit_roles (id) {
        id -> Uuid,

        unit_id -> Uuid,
        display_name -> Text,
        description -> Nullable<Text>,

        permissions -> BigInt,
        position -> BigInt,
        kind -> Text,

        // audit
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    unit_member_roles (member_id, role_id) {
        member_id -> Uuid,
        role_id -> Uuid,
        unit_id -> Uuid,
        created_at -> Timestamptz,
    }
}

table! {
    unit_settings (unit_id) {
        // id
        unit_id -> Uuid,

        // connections
        discord_guild_id -> Nullable<Text>,
        discord_guild_joined_at -> Nullable<Timestamp>,

        // settings
        initial_rank_id -> Uuid,

        // audit
        updated_at -> Timestamptz,
        updated_by -> Nullable<Uuid>,
    }
}
