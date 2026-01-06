// Minimal test to isolate the DPDK initialization issue

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;
use dpdk_sys::*;

fn main() {
    unsafe {
        // Simplest possible EAL init
        let args = vec![
            CString::new("test").unwrap(),
            CString::new("-l").unwrap(),
            CString::new("0").unwrap(),
            CString::new("--no-huge").unwrap(),
            CString::new("--no-pci").unwrap(),
            CString::new("--no-telemetry").unwrap(),
            CString::new("-d").unwrap(),
            CString::new("/usr/local/lib/x86_64-linux-gnu/librte_net_pcap.so").unwrap(),
        ];

        let mut c_args: Vec<*mut c_char> = args
            .iter()
            .map(|arg| arg.as_ptr() as *mut c_char)
            .collect();
        c_args.push(ptr::null_mut());

        let argc = (c_args.len() - 1) as c_int;
        let argv = c_args.as_mut_ptr();

        println!("About to call rte_eal_init...");
        let ret = rte_eal_init(argc, argv);
        println!("rte_eal_init returned: {}", ret);

        if ret < 0 {
            eprintln!("Error with EAL initialization");
            std::process::exit(1);
        }

        println!("SUCCESS! DPDK EAL initialized");
    }
}
