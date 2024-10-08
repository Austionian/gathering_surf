mod forecast;
mod glimpse;
mod handle_404;
mod health_check;
mod realtime;
mod root;
#[cfg(debug_assertions)]
mod watch;

pub use forecast::forecast;
pub use glimpse::glimpse;
pub use handle_404::handle_404;
pub use health_check::health_check;
pub use realtime::realtime;
pub use root::*;
#[cfg(debug_assertions)]
pub use watch::watch;
