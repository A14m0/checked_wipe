use std::{
    time::{
        Duration, 
        SystemTime
    },
    io::{
        self,
        prelude::*,
    }
};

mod defs;
pub use defs::{
    DiskData,
    parse_partitions,
    print_top_levels
};


/// zeros the drive referred to by `disk
pub fn zero_drive(disk: &DiskData) -> Result<(), String> {
    // open the file and prep variables
    let mut drive_handle = std::fs::File::create(disk.path.clone()).expect("Failed to open disk for writing");
    let write_buf: [u8; 1024*1024] = [0;1024*1024];
    let mut total_written = 0u32;
    let percent_denom = disk.size * ((1024 * 1024) as f64);
    let mbs = 1024*1000;
    
    // loop until disk is fully written
    for idx in 0..disk.size as u64 {
        let now = SystemTime::now();
        // for each gigabyte...
        for _idx2 in 0..1024{
            match drive_handle.write_all(&write_buf){
                Ok(_) => (),
                Err(e) => println!("[-] Hit write error: {}", e)
            };
        }

        // update measured stats
        total_written += 1024*1024; // written 1 mb
        let elapsed = match now.elapsed(){
            Ok(a) => a,
            Err(e) => {
                println!("[-] Failed to time the write transactions: {}", e);
                Duration::from_secs(0)
            }
        };

        // clear line and print updates
        print!("                                                                \r");
        io::stdout().flush().unwrap();
        print!("Written {} GB, {:.2}% completed ({} MB/s)\r", idx, (total_written*100) as f64/percent_denom, mbs/elapsed.as_millis());
        io::stdout().flush().unwrap();

        // need to flush the drive file so we dont pretend we 
        // are writing faster than we actually are
        drive_handle.flush().unwrap(); 
    }
    
    Ok(())
}