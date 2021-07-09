use std::{
    io::{
        prelude::*,
    },
    os::unix::io::AsRawFd
};
use indicatif::{
    ProgressBar,
    ProgressStyle
};
use nix;

mod defs;
pub use defs::{
    DiskData,
    parse_partitions,
    print_top_levels
};

fn get_drive_size(path: String) -> usize {
    // prep the IOCTL call
    let fs = std::fs::File::open(path).expect("Failed to open disk for reading");
    const SPI_IOC_MAGIC: u8 = 0x12;
    const SPI_IOC_TYPE_MODE: u8 = 114;
    nix::ioctl_read!(blkgetsize64, SPI_IOC_MAGIC, SPI_IOC_TYPE_MODE, u64);
    let mut fssize: u64 = 0;
    
    // run ioctl(path, BLKGETSIZE64, out)
    let out = unsafe { blkgetsize64(fs.as_raw_fd(), &mut fssize) };
    assert_eq!{out.unwrap(), 0};

    fssize as usize 
}


/// zeros the drive referred to by `disk
pub fn zero_drive(disk: &DiskData) -> Result<(), String> {
    // first get the file's size
    let fsize = get_drive_size(disk.path.clone()); 
    let write_loop_ctr = fsize / (1024*1024*1024);
    let final_write = fsize % 1024;

    // open the file and prep variables
    let mut drive_handle = std::fs::File::create(disk.path.clone()).expect("Failed to open disk for writing");
    let write_buf: [u8; 1024*1024] = [0;1024*1024];
    
    // initialize a progress bar
    let bar = ProgressBar::new(fsize as u64);
    bar.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.green/red} {pos:>7}/{len:7} bytes")
                .progress_chars("##-"));
    
    // loop until disk is fully written
    for _ in 0..write_loop_ctr as u64 {
        for _ in 0..1024 {
            match drive_handle.write_all(&write_buf){
                Ok(_) => (),
                Err(e) => println!("[-] Hit write error: {}", e)
            };
            
            // increment the progress bar
            bar.inc(1024 * 1024);
        }
        
        // need to flush the drive file so we dont pretend we 
        // are writing faster than we actually are
        drive_handle.flush().unwrap(); 
    }

    // write whatever last bytes need to be written
    for _ in 0..final_write {
        match drive_handle.write(&[0]) {
            Ok(_) => (),
            Err(e) => println!("[-] Hit write error: {}", e)
        }
        bar.inc(1);
    }

    bar.finish();
    
    Ok(())
}


/// checks to see if a drive was really zeroed out
pub fn assert_check(disk: &DiskData) -> Result<(), String> {
    // first get the file's size
    let fsize = get_drive_size(disk.path.clone()); 
    let mut fs = std::fs::File::open(disk.path.clone()).unwrap();
    let checker: [u8; 1024] = [0; 1024];
    let mut buff: [u8; 1024] = [0; 1024];

    // initialize a progress bar
    let bar = ProgressBar::new(fsize as u64);
    bar.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.green/red} {pos:>7}/{len:7} bytes")
                .progress_chars("##-"));
    

    for _ in 0..(fsize/1024) {
        fs.read(&mut buff[..]).unwrap();
        assert_eq!(checker, buff);
        bar.inc(1024);
    }   
    bar.finish();
    
    Ok(())
}