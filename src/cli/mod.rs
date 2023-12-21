use std::io::{self, Write};

#[cfg(feature = "mastodon")]
mod clear_toots;
mod reset_disruptions;
mod show_users;

fn ask_are_u_sure(question: &str) -> bool {
    print!("{question} [y/n]");
    io::stdout().flush().unwrap();
    let mut user_input = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut user_input).unwrap();
    if user_input == "y\n" {
        true
    } else {
        println!("Aborted");
        false
    }
}

#[cfg(feature = "mastodon")]
pub use clear_toots::clear_toots;
pub use reset_disruptions::reset_disruptions;
pub use show_users::show_users;
