// Rust implementation of reflector.c - a simple DPDK packet reflector
// This program receives packets on a port and sends them back out the same port

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;

// Import DPDK bindings
use dpdk_sys::*;

const RING_SIZE: u16 = 1024;
const NUM_MBUFS: u32 = 1024;
const MBUF_CACHE_SIZE: u32 = 250;
const MAX_PKT_BURST: u16 = 32;

/// Initialize a DPDK port with RX and TX queues
unsafe fn port_init(port: u16) -> Result<(), i32> {
    // Check if port is valid
    if rte_eth_dev_is_valid_port(port) == 0 {
        return Err(-1);
    }

    // Create mbuf pool
    let pool_name = CString::new(format!("MBUF_POOL_{}", port)).unwrap();
    let mbuf_pool = rte_pktmbuf_pool_create(
        pool_name.as_ptr(),
        NUM_MBUFS,
        MBUF_CACHE_SIZE,
        0,
        RTE_MBUF_DEFAULT_BUF_SIZE as u16,
        rte_socket_id() as i32,
    );

    if mbuf_pool.is_null() {
        eprintln!("Cannot create mbuf pool");
        return Err(-1);
    }

    // Initialize port configuration
    let port_conf: rte_eth_conf = std::mem::zeroed();
    let rx_rings: u16 = 1;
    let tx_rings: u16 = 1;
    let mut nb_rxd = RING_SIZE;
    let mut nb_txd = RING_SIZE;

    // Get device info
    let mut dev_info: rte_eth_dev_info = std::mem::zeroed();
    let retval = rte_eth_dev_info_get(port, &mut dev_info);
    if retval != 0 {
        eprintln!("Error getting device info for port {}: {}", port, retval);
        return Err(retval);
    }

    // Configure the Ethernet device
    let retval = rte_eth_dev_configure(port, rx_rings, tx_rings, &port_conf);
    if retval != 0 {
        eprintln!("Error configuring device: {}", retval);
        return Err(retval);
    }

    // Adjust ring sizes
    let retval = rte_eth_dev_adjust_nb_rx_tx_desc(port, &mut nb_rxd, &mut nb_txd);
    if retval != 0 {
        eprintln!("Error adjusting ring sizes: {}", retval);
        return Err(retval);
    }

    // Set up RX queue
    let retval = rte_eth_rx_queue_setup(
        port,
        0,
        nb_rxd,
        rte_eth_dev_socket_id(port) as u32,
        ptr::null(),
        mbuf_pool,
    );
    if retval < 0 {
        eprintln!("Error setting up RX queue: {}", retval);
        return Err(retval);
    }

    // Set up TX queue
    let mut txconf = dev_info.default_txconf;
    txconf.offloads = port_conf.txmode.offloads;
    let retval = rte_eth_tx_queue_setup(
        port,
        0,
        nb_txd,
        rte_eth_dev_socket_id(port) as u32,
        &txconf,
    );
    if retval < 0 {
        eprintln!("Error setting up TX queue: {}", retval);
        return Err(retval);
    }

    // Start the Ethernet port
    let retval = rte_eth_dev_start(port);
    if retval < 0 {
        eprintln!("Error starting device: {}", retval);
        return Err(retval);
    }

    // Get and display MAC address
    let mut addr: rte_ether_addr = std::mem::zeroed();
    let retval = rte_eth_macaddr_get(port, &mut addr);
    if retval != 0 {
        return Err(retval);
    }

    println!(
        "Port {} MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        port,
        addr.addr_bytes[0],
        addr.addr_bytes[1],
        addr.addr_bytes[2],
        addr.addr_bytes[3],
        addr.addr_bytes[4],
        addr.addr_bytes[5]
    );

    // Enable promiscuous mode
    let retval = rte_eth_promiscuous_enable(port);
    if retval != 0 {
        eprintln!("Error enabling promiscuous mode: {}", retval);
        return Err(retval);
    }

    Ok(())
}

/// Main packet forwarding loop
unsafe fn wire_ports(in_port: u16, out_port: u16) {
    let mut bufs: [*mut rte_mbuf; MAX_PKT_BURST as usize] = [ptr::null_mut(); MAX_PKT_BURST as usize];
    let mut total_forwarded: u64 = 0;
    let mut total_dropped: u64 = 0;

    println!("Starting packet forwarding:");
    println!("  IN:  Port {}", in_port);
    println!("  OUT: Port {}", out_port);

    loop {
        // Receive burst of packets
        let nb_rx = rte_eth_rx_burst(in_port, 0, bufs.as_mut_ptr(), MAX_PKT_BURST);

        if nb_rx > 0 {
            // Send burst to out_port
            let nb_tx = rte_eth_tx_burst(out_port, 0, bufs.as_mut_ptr(), nb_rx);

            total_forwarded += nb_tx as u64;
            if nb_tx > 0 {
                println!("Total forwarded packets: {}", total_forwarded);
                println!("Total dropped packets: {}", total_dropped);
            }

            // Free any packets that weren't sent
            if nb_tx < nb_rx {
                total_dropped += (nb_rx - nb_tx) as u64;
                for i in nb_tx..nb_rx {
                    rte_pktmbuf_free(bufs[i as usize]);
                }
            }
        }
    }
}

fn main() {
    unsafe {
        // Collect command line arguments as CStrings
        // We need to keep the CStrings alive for the duration of rte_eal_init
        let args: Vec<CString> = std::env::args()
            .map(|arg| CString::new(arg).unwrap())
            .collect();

        // Create a vector of raw pointers
        let mut c_args: Vec<*mut c_char> = args
            .iter()
            .map(|arg| arg.as_ptr() as *mut c_char)
            .collect();

        // Add NULL terminator (some systems expect it)
        c_args.push(ptr::null_mut());

        let mut argc = (c_args.len() - 1) as c_int;  // Exclude NULL terminator from count
        let mut argv = c_args.as_mut_ptr();

        // Initialize DPDK EAL
        // Note: rte_eal_init modifies argc and argv to consume EAL arguments
        let ret = rte_eal_init(argc, argv);
        if ret < 0 {
            eprintln!("Error with EAL initialization");
            std::process::exit(1);
        }

        // Calculate how many arguments remain after EAL consumed some
        let remaining_argc = (argc - ret) as usize;

        // Get the remaining arguments (application-specific args after --)
        if remaining_argc != 2 {
            println!("Usage: reflector [EAL options] -- <port_id>");
            println!("Example: sudo ./reflector -l 0 --no-huge --no-pci --vdev 'net_pcap0,rx_pcap=test.pcap,tx_pcap=out.pcap' -- 0");
            std::process::exit(1);
        }

        // Parse port_id from the remaining arguments
        // argv now points to the remaining arguments after EAL processing
        let remaining_argv = std::slice::from_raw_parts(argv.offset(ret as isize), remaining_argc);
        let port_str = std::ffi::CStr::from_ptr(remaining_argv[1]).to_str().unwrap();
        let port_id: u16 = port_str.parse().unwrap_or_else(|_| {
            eprintln!("Invalid port number");
            std::process::exit(1);
        });

        // Initialize the port
        if let Err(e) = port_init(port_id) {
            eprintln!("Cannot init port {}: error {}", port_id, e);
            std::process::exit(1);
        }

        println!("Starting single-port loopback on port {}", port_id);
        println!("Packets received on port {} will be sent back out port {}", port_id, port_id);

        // Run loopback directly in main thread
        wire_ports(port_id, port_id);
    }
}
