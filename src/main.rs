#![feature(pattern)]
use std::{
    process::Command,
    str::{
        from_utf8,
        pattern::Pattern,
    },
    io::{
        self, 
        BufRead
    },
    path::Path,
    fs::File,
    fmt::Display
};
use nix::unistd::Uid;
#[macro_use] extern crate scan_fmt;

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
        write!(fmt, "Path: {}, Start: {}, End: {}, Size: {}, File system: {}, Mount state: {}",
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
        write!(fmt, "Path: {}, Size: {} {}, # Unmounted Partitions: {}",
            self.path, self.size, self.size_unit, self.partitions.len()
        )
    }
}


/// prints a welcome message to the user
fn print_welcome(){
    println!("Welcome to Ch3cked W1pe");
}

/// populated `drives_vec` with the currently unmounted available drives
fn get_unmounted(drives_vec: &mut Vec<DiskData>) -> Result<(), String> {
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
                if !partition.is_mounted{
                    match disk.add_partition(partition.clone()){
                        Ok(_) => (),
                        Err(e) => panic!("Failed to add partition to structure: {}", e)
                    };
                }
            }
        }

        // note we disregard all other lines at the moment, including sector
        // information, disklabel types, and other misc information
    }

    Ok(())
}

/* Sample FDISK output


Disk /dev/nvme0n1: 931.51 GiB, 1000204886016 bytes, 1953525168 sectors
Disk model: CT1000P1SSD8                            
Units: sectors of 1 * 512 = 512 bytes
Sector size (logical/physical): 512 bytes / 512 bytes
I/O size (minimum/optimal): 512 bytes / 512 bytes
Disklabel type: gpt
Disk identifier: B381325B-40D1-5643-83B0-4C5C67AC51F5

Device            Start        End    Sectors  Size Type
/dev/nvme0n1p1     2048    1050623    1048576  512M EFI System
/dev/nvme0n1p2  1050624   68159487   67108864   32G Linux swap
/dev/nvme0n1p3 68159488 1953525134 1885365647  899G Linux filesystem


Disk /dev/nvme1n1: 931.51 GiB, 1000204886016 bytes, 1953525168 sectors
Disk model: CT1000P1SSD8                            
Units: sectors of 1 * 512 = 512 bytes
Sector size (logical/physical): 512 bytes / 512 bytes
I/O size (minimum/optimal): 512 bytes / 512 bytes
Disklabel type: gpt
Disk identifier: 5E46032F-79DF-604D-8BF0-1A47542AE033

Device              Start        End    Sectors   Size Type
/dev/nvme1n1p1       2048     206847     204800   100M EFI System
/dev/nvme1n1p2     206848     239615      32768    16M Microsoft reserved
/dev/nvme1n1p3     239616 1952485492 1952245877 930.9G Microsoft basic data
/dev/nvme1n1p4 1952487424 1953521663    1034240   505M Windows recovery environm


Disk /dev/sda: 3.64 TiB, 4000787030016 bytes, 7814037168 sectors
Disk model: WDC WD4005FZBX-0
Units: sectors of 1 * 512 = 512 bytes
Sector size (logical/physical): 512 bytes / 4096 bytes
I/O size (minimum/optimal): 4096 bytes / 4096 bytes
Disklabel type: gpt
Disk identifier: 11B1E966-CA57-CF40-9E7B-602311EBF91C

Device     Start        End    Sectors  Size Type
/dev/sda1   2048 7814037134 7814035087  3.6T Linux filesystem

*/

fn main() {
    print_welcome();
    // check we are running as root
    if !Uid::effective().is_root() {
        panic!("You must run this executable with root permissions");
    }


    let mut drives_vec: Vec<DiskData> = Vec::new();
    get_unmounted(&mut drives_vec).expect("Failed to read drives");

    for drive in drives_vec.iter(){
        println!("{}", drive);
        for partition in drive.partitions.iter(){
            println!("\t{}", partition);
        }
    }
}
