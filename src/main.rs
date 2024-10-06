use std::io::{self, Write}; // For manual flushing
use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello from Docker!!!!!"); // Print to console
    io::stdout().flush().unwrap();  // Ensure output is flushed immediately

    // Keep the container alive for debugging purposes
    thread::sleep(Duration::from_secs(3600));  // Sleep for 1 hour
}