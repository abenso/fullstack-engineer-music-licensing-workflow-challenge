//! Route path patterns, kept in one place so they're defined once and
//! reused wherever a route is registered.

pub const HEALTH: &str = "/health";

pub const MOVIES: &str = "/movies";
pub const MOVIE_BY_ID: &str = "/movies/{id}";
pub const MOVIE_SCENES: &str = "/movies/{id}/scenes";

pub const SONGS: &str = "/songs";
