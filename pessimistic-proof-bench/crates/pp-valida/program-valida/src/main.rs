#![no_main]

use std::io::stdin;

#[no_mangle]

pub fn main() {
    println!("Please enter a number from 0 to 46:");
    let n = loop {
        let mut input = String::new();
        // Read a line from stdin and parse it as an u8.
        match stdin().read_line(&mut input) {
            Ok(_) => {
                match input.trim().parse::<u8>() {
                    Ok(num) => {
                        if num == 0 {
                            println!("The 0th fibonacci number is: 0");
                            return;
                        } else if num > 46 {
                            println!("Error: n is too large. Please enter a number no larger than 46.");
                        } else {
                            break num;
                        }
                    },
                    Err(e) => {
                        println!("Error reading input: {}. Please try again:", e);
                    }
                }
            }
            Err(e) => {
                println!("Error reading input: {}. Please try again:", e);
            }
        }
    };
    let mut a: u32 = 0;
    let mut b: u32 = 1;
    let mut sum: u32;
    for _ in 1..n {
        sum = a + b;
        a = b;
        b = sum;
    }
    println!("The {}-th fibonacci number is: {}", n, b);
}
