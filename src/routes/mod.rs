mod forecast;
mod handle_404;
mod health_check;
mod latest;
mod root;

pub use forecast::forecast;
pub use handle_404::handle_404;
pub use health_check::health_check;
pub use latest::latest;
pub use root::root;
