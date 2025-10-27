use bincode::config::{Config, standard};

pub const fn format_config() -> impl Config {
    standard().with_fixed_int_encoding()
}
