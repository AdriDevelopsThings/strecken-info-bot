use std::io::{self, Write};

mod reset_disruptions;
#[cfg(feature = "telegram")]
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

pub use reset_disruptions::reset_disruptions;
#[cfg(feature = "telegram")]
pub use show_users::show_users;
