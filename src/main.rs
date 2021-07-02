#![feature(pattern)]
use std::{
    
    io::{
        self,
        prelude::*,
    },
    
};
use nix::unistd::Uid;
#[macro_use] extern crate scan_fmt;

// import our local stuff
mod definitions;
use definitions::{
    DiskData,
    parse_partitions,
    print_top_levels,
    zero_drive
};






/// define the default number of passes you want to go over the drive with
static DEFAULT_PASS_NUM: u32 = 5;



/// prints a welcome message to the user
fn print_welcome(){
    println!("Welcome to Ch3cked W1pe");
}




fn main() {
    print_welcome();
    // check we are running as root
    if !Uid::effective().is_root() {
        panic!("[-] This program must be run as root");
    }


    let mut drives_vec: Vec<DiskData> = Vec::new();
    parse_partitions(&mut drives_vec).expect("Failed to read drives");

    println!("All Drives ___________________________________________________");
    for drive in drives_vec.iter(){
        println!("\t{}", drive);
        for partition in drive.partitions.iter(){
            println!("\t\t{}", partition);
        }
    }

    println!("\nAll Drives Currently Unmounted _______________________________");
    let mut umount_idx_vec: Vec<usize> = Vec::new();
    let mut ctr = 0;
    let mut idx = 0;
    for drive in drives_vec.iter(){
        let mut is_drive_mounted: bool = false;
        for partition in drive.partitions.iter(){
            if partition.is_mounted {
                is_drive_mounted = true ;
            }
        }

        // if the drive is not mounted, print it and save the index
        if !is_drive_mounted {
            println!("{}\t{}", ctr+1, drive);
            umount_idx_vec.push(idx);

            ctr += 1;
        }
        idx += 1;
    }

    println!("______________________________________________________________");
    println!("Select the drive you would like to format (`q` to quit)");
    let mut user_selection = -1;
    let mut is_done = false;

    // get the user's desired drive, either quitting or looping on character input
    while !is_done {
        let mut input_text = String::new();
        print!(" > ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input_text).expect("failed to read from stdin");

        let trimmed = input_text.trim();
        match trimmed.parse::<i32>() {
            // user gave proper selection
            Ok(i) => {
                // make sure the user isnt being an idiot
                if i < 1 || i > umount_idx_vec.len() as i32 {
                    println!("[-] Not a valid drive index. Please try again");
                } else {
                    user_selection = i;
                    is_done = true
                }
                
            },
            Err(..) => {
                if &trimmed[0..1] == "q" {
                    println!("[ ] Caught quitting input. Doing so...");
                    std::process::exit(0);
                } else {
                    println!("[-] Not a valid drive index. Please try again");
                }
            },
        };
    }

    
    // print drive partition information
    println!("______________________________________________________________");
    println!("You have selected disk # {}", user_selection);
    println!("{}", drives_vec[umount_idx_vec[user_selection as usize-1]]);
    match print_top_levels(&drives_vec[umount_idx_vec[user_selection as usize-1]]){
        Ok(_) => (),
        Err(e) => println!("Failed to print all the things: {}", e)
    };

    // make sure the user wants to continue
    println!("Does this information look correct? (y/N)");
    let mut input_text = String::new();
    print!(" > ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input_text).expect("failed to read from stdin");
    
    let trimmed = input_text.trim();
    if trimmed.to_lowercase() != "y" {
        println!("[-] Caught non-affirmative. Quitting...");
        std::process::exit(0);
    }

    // final safety check. is the user really sure they want to format everything???
    println!("______________________________________________________________");
    println!("WARNING WARNING WARNING WARNING WARNING WARNING WARNING WARNING");
    println!("______________________________________________________________");
    println!("");
    println!("YOU ARE ABOUT TO PERMANENTLY DELETE ALL INFORMATION FROM THIS DISK.");
    println!("ARE YOU SURE YOU WISH TO CONTINUE? THERE IS NO GOING BACK AFTER THIS");
    println!("y/N");

    let mut input_text = String::new();
    print!(" > ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input_text).expect("failed to read from stdin");
    
    let trimmed = input_text.trim();
    if trimmed.to_lowercase() != "y" {
        println!("[-] Caught non-affirmative. Quitting...");
        std::process::exit(0);
    }

    println!("______________________________________________________________");
    println!("Securing formatting drive ({} passes of zeros). This will take a while...", DEFAULT_PASS_NUM);
    for i in 0..DEFAULT_PASS_NUM {
        println!("On pass #{}                                                       ", i+1);
        match zero_drive(&drives_vec[umount_idx_vec[user_selection as usize-1]]){
            Ok(_) => (),
            Err(e) => println!("Zero drive issue hit: {}", e)
        }
    }

    println!("______________________________________________________________");
    println!("[+] Wipe complete!");
    
}
