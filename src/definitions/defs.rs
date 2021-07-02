use std::{
    io::{
        self,
        BufRead,
    },
    process::Command,
    str::{
        from_utf8,
        pattern::Pattern
    },
    fmt::Display,
    path::Path,
    fs::File,
    
};

///////// HELPER FUNCTIONS ///////////
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


/// prints all of the top level directories of each partition on `disk`
pub fn print_top_levels(disk: &DiskData) -> Result<(), String>{
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


/// populated `drives_vec` with the currently unmounted available drives
pub fn parse_partitions(drives_vec: &mut Vec<DiskData>) -> Result<(), String> {
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



/// attempts to mount the partition at `PATH` to `/tmp/mnt`
pub fn try_mount(path: String) -> Result<u32, String> {
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
pub fn unmount() -> Result<(), String> {
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





///////// STRUCTURE DEFINTIONS //////////

/// define our structure for partition data
#[derive (Clone)]
pub struct PartitionData{
    id: String,
    start: u64,
    end: u64,
    size: String,
    fstype: String,
    pub is_mounted: bool
}

/// define functions for our structures
impl PartitionData {
    pub fn new(part_line: String) -> Self {
        let fmt: &str;
        if part_line.contains("*") {
            fmt = "{}\t*\t{}\t{}\t{}\t{}\t{}"
        } else {
            fmt = "{}\t{}\t{}\t{}\t{}\t{}";
        }
        let (id, start, end, _sectors, size, fstype) = scan_fmt!(&part_line[..], 
                                                        fmt, 
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


/// define our structure for disk information
#[derive (Clone)]
pub struct DiskData {
    pub path: String,
    pub size: f64,
    pub size_unit: String,
    pub partitions: Vec<PartitionData>
}

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