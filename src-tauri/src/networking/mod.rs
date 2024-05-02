/*
Module for housing all network related resources, which are used to connect
to and communicate with Sophos
*/

// Contains data structs related to all network related calls
pub mod data_types;

// Manages saving and loading of users' credentials
pub mod credentials;

// Manages user login and logout
pub mod user;

// Manages remaining traffic details of the user
pub mod traffic;
