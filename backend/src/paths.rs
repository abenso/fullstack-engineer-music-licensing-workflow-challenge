//! Route path patterns, kept in one place so they're defined once and
//! reused wherever a route is registered.

pub const HEALTH: &str = "/health";
pub const EVENTS: &str = "/events";

pub const MOVIES: &str = "/movies";
pub const MOVIE_BY_ID: &str = "/movies/{id}";
pub const MOVIE_SCENES: &str = "/movies/{id}/scenes";
pub const MOVIE_TRACKS: &str = "/movies/{id}/tracks";

pub const SONGS: &str = "/songs";

pub const SCENE_TRACKS: &str = "/scenes/{id}/tracks";

pub const TRACK_BY_ID: &str = "/tracks/{id}";
pub const TRACK_LICENSE: &str = "/tracks/{id}/license";
