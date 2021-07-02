#![feature(pattern)]
use std::{
    process::Command,
    str::{
        from_utf8,
        pattern::Pattern,
    },
    io::{
        self,
        prelude::*, 
        BufRead,
    },
    path::Path,
    fs::File,
    fmt::Display
};
use nix::unistd::Uid;
#[macro_use] extern crate scan_fmt;


/// define the default number of passes you want to go over the drive with
static default_pass_num: u32 = 5;



/// define our structure for partition data
#[derive (Clone)]
struct PartitionData{
    id: String,
    start: u64,
    end: u64,
    size: String,
    fstype: String,
    is_mounted: bool
}

/// define our structure for disk information
#[derive (Clone)]
struct DiskData {
    path: String,
    size: f64,
    size_unit: String,
    partitions: Vec<PartitionData>
}





/// helper function for reading lines from a file
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// checks to see if the partition is currently mounted
fn is_mounted(id: String) -> bool {
    if let Ok(proc_mnts) = read_lines("/proc/mounts"){
        for line in proc_mnts {
            // check if the line begins with our path
            if let Ok(l) = line {
                if id.is_prefix_of(&l[..]){
                    // it is mounted
                    return true;
                }
            }
        }
    }
    
    // it is not mounted anywhere
    false
}

/// attempts to mount the partition at `PATH` to `/tmp/mnt`
fn try_mount(path: String) -> Result<u32, String> {
    // make sure the directory `/tmp/mnt` exists, if not create it
    match std::fs::read_dir("/tmp/mnt") {
        Ok(_) => (),
        Err(_) => match std::fs::create_dir("/tmp/mnt"){
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to mount drive: {}", e))
        } 
    }

    // try to mount the drive to the point
    match Command::new("mount").arg(&path[..]).arg("/tmp/mnt").status() {
        Ok(a) => {
            if !a.success(){
                match a.code(){
                    Some(code) => {
                        // 32 means unknown fs, we dont want to die on this, 
                        // so we will just print that we dont know the fs
                        if code != 32 {
                            return Err(format!("mount failed with {}", a));
                        } else {
                            println!("\t    Unknown partition type");
                            return Ok(1)
                        }
                    },
                    None => return Err("mount was terminated by a signal".to_string())
                }
            }
        },
        Err(e) => return Err(format!("Failed to run mount command: {}", e))
    };
    
    Ok(0)   
}

/// unmounts the partition on `/tmp/mnt`
fn unmount() -> Result<(), String> {
    // try to unmount the drive
    match Command::new("umount").arg("/tmp/mnt").status() {
        Ok(a) => {
            if !a.success(){
                return Err(format!("umount failed with {}", a));
            }
        },
        Err(e) => return Err(format!("Failed to run umount command: {}", e))
    };
    
    Ok(())
}

/// prints all of the top level directories of each partition on `disk`
fn print_top_levels(disk: &DiskData) -> Result<(), String>{
    let mut ctr = 1;
    for partition in disk.partitions.iter(){
        println!("\tPartition #{}", ctr);
        let status = match try_mount(partition.id.clone()){
            Ok(a) => a,
            Err(e) => return Err(e)
        };
        
        // we ignore partitions that return 1, as they are unknown and 
        // therefore not mounted to `/tmp/mnt`
        if status == 0 {
            let paths = std::fs::read_dir("/tmp/mnt").unwrap();
            for path in paths {
                println!("\t    {:?}", path.unwrap().file_name());
            }
            unmount()?;
        }
        ctr += 1;
        
    }

    Ok(())
}

/// zeros the drive referred to by `disk
fn zero_drive(disk: &DiskData) -> Result<(), String> {
    let drive_handle = std::fs::File::
    
    Ok(())
}


/// define functions for our structures
impl PartitionData {
    fn new(part_line: String) -> Self {
        let (id, start, end, sectors, size, fstype) = scan_fmt!(&part_line[..], 
                                                        "{}\t{}\t{}\t{}\t{}\t{}", 
                                                        String, u64, u64, u64, 
                                                        String, String).unwrap();
        
        let is_mounted = is_mounted(id.clone());

        PartitionData {id,start,end,size,fstype, is_mounted}
    }

}
impl Display for PartitionData {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}, Start: {}, End: {}, Size: {}, File system: {}, Mount state: {}",
            self.id, self.start, self.end, self.size, self.fstype, self.is_mounted
        )
    }
}

//Disk /dev/nvme1n1: 931.51 GiB, 1000204886016 bytes, 1953525168 sectors

impl DiskData {
    /// creates a new `DiskData` instance
    fn new(disk_line: String) -> Self {
        let (path, size, size_unit, _bytes) = scan_fmt!(&disk_line[..], "Disk {}: {} {}, {} bytes", String, f64, String, u64).unwrap();
        DiskData { path,size,size_unit,partitions: Vec::new() }
    }

    /// adds a partition to the disk structure
    fn add_partition(&mut self, part: PartitionData ) -> Result<u32,u32> {
        self.partitions.push(part);
        Ok(0)
    }
}

impl Display for DiskData {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Path: {}, Size: {} {}, # Partitions: {}",
            self.path, self.size, self.size_unit, self.partitions.len()
        )
    }
}


/// prints a welcome message to the user
fn print_welcome(){
    println!("Welcome to Ch3cked W1pe");
}

/// populated `drives_vec` with the currently unmounted available drives
fn parse_partitions(drives_vec: &mut Vec<DiskData>) -> Result<(), String> {
    println!("Reading partition information...");
    // get fdisk's drive listing
    let system_output = match Command::new("fdisk").arg("-l").output() {
        Ok(a) => a.stdout,
        Err(e) => return Err(format!("Failed to execute fdisk: {}", e))
    };
                                
    // loop over each line of the output
    for line in system_output.split(|c| (c == &('\n' as u8))){
        let tmp_str = match from_utf8(line){
            Ok(a) => a,
            Err(_) => return Err("Failed to convert string".to_string())
        };
        // check if the string defines a disk line
        if String::from("Disk /").is_prefix_of(tmp_str){
            let diskdat = DiskData::new(tmp_str.to_string());
            drives_vec.push(diskdat.to_owned())
        }

        // check if the string defines a partition line
        for disk in drives_vec.iter_mut(){
            if disk.path.is_prefix_of(tmp_str){
                let partition = PartitionData::new(tmp_str.to_string());
                // disregard any partitions that are mounted
                match disk.add_partition(partition.clone()){
                    Ok(_) => (),
                    Err(e) => panic!("Failed to add partition to structure: {}", e)
                };
                
            }
        }

        // note we disregard all other lines at the moment, including sector
        // information, disklabel types, and other misc information
    }

    Ok(())
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
    for drive in drives_vec.iter(){
        let mut is_drive_mounted: bool = false;
        for partition in drive.partitions.iter(){
            if partition.is_mounted {
                is_drive_mounted = true ;
            }
        }

        // if the drive is not mounted, print it and save the index
        if !is_drive_mounted {
            println!("{}\t{}", ctr, drive);
            umount_idx_vec.push(ctr);
        }
        ctr += 1;
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
    println!("Securing formatting drive ({} passes of zeros)...", default_pass_num);
    for _ in 0..default_pass_num {
        zero_drive(drives_vec[umount_idx_vec[user_selection as usize-1]]);
    }
    
}
