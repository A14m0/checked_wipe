use std::{
    io::{
        prelude::*,
    }
};
use indicatif::{
    ProgressBar,
    ProgressStyle
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
    
    // initialize a progress bar
    let bar = ProgressBar::new(disk.size as u64);
    bar.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.red/cyan} {pos:>7}/{len:7} GB")
                .progress_chars("##-"));
    
    // loop until disk is fully written
    for _ in 0..disk.size as u64 {
        //let now = SystemTime::now();
        // for each gigabyte...
        for _idx2 in 0..1024{
            match drive_handle.write_all(&write_buf){
                Ok(_) => (),
                Err(e) => println!("[-] Hit write error: {}", e)
            };
        }

        // increment the progress bar
        bar.inc(1);

        // need to flush the drive file so we dont pretend we 
        // are writing faster than we actually are
        drive_handle.flush().unwrap(); 
    }
    bar.finish();
    
    Ok(())
}