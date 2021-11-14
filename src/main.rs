use std::net::{UdpSocket, Ipv4Addr, Ipv6Addr};
use anyeyeballs_orchestrator::{Node, send_error, log, Config};
use std::collections::{HashMap};
use std::env;
use std::time::SystemTime;
use std::path::Path;
use std::fs::File;
use std::io::Read;

fn main() {

    // Load config
    let mut config = String::new();
    File::open(&env::args().nth(1).unwrap())
        .and_then(|mut f| f.read_to_string(&mut config))
        .unwrap();

    let config = Config::new(config);

    // Create log file
    let log_path = Path::new(&config.log_file);
    let mut fp = match File::create(&log_path) {
        Err(err) => panic!("Unable to create log file: {}", err),
        Ok(fp) => fp,
    };
    // Create empty hashmap for nodes
    let mut nodes = HashMap::new();
    // Create and bind UDP socket to listen for incoming nodes messages
    let socket = match UdpSocket::bind(config.orch_addr) {
        Err(err) => panic!("Unable to bind Orchestrator Socket: {}", err),
        Ok(socket) => socket,
    };
    // Create hash maps for IPv4 and IPv6 addresses
    let mut v4_boxes = HashMap::new();
    let mut v6_boxes = HashMap::new();

    // Create buffer to read incoming messages
    let mut buffer = [0; 512];
    let mut ts = SystemTime::now();
    loop {
        // Clone the socket so we can handle multiple incoming packets at once
        let sock = socket.try_clone().unwrap();
        let (_num_bytes, addr) = sock.recv_from(&mut buffer).unwrap();
        match &buffer[0] {
            // New join
            0 => {
                println!("Got new join");
                let mut node_id: u8 = 0;
                while node_id < config.max_nodes {
                    if !nodes.contains_key(&node_id) {
                        break;
                    } else {
                        node_id += 1;
                    }
                }
                if node_id == config.max_nodes {
                    println!("Node list full!");
                    // Send error back
                    send_error(sock,addr, 0_u8);
                } else {
                    let mut new_node = Node::new(sock, addr, node_id, Ipv4Addr::new(buffer[1], buffer[2], buffer[3], buffer[4]), Ipv6Addr::from([buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15], buffer[16], buffer[17], buffer[18], buffer[19], buffer[20]]));
                    new_node.ok_join();
                    if !v4_boxes.contains_key(&new_node.get_v4_addr()) {
                        v4_boxes.insert(new_node.get_v4_addr(), 0);
                    }
                    if !v6_boxes.contains_key(&new_node.get_v6_addr()) {
                        v6_boxes.insert(new_node.get_v6_addr(), 0);
                    }
                    nodes.insert(node_id, new_node);
                    println!("{:?}", nodes[&node_id]);
                }
            }
            // New leave
            1 => {
                println!("Got new leave for node {:?}", buffer[1]);
                nodes.remove(&buffer[1]);
            }
            // New status
            2 => {
                let node = nodes.get_mut(&buffer[1]).unwrap();
                let old_avg = node.get_avg_total_load();
                println!("{:?}", buffer[2]);
                let total_load = (buffer[2] as f64/ 200 as f64) as f64;
                let v4_load = (buffer[3] as f64/ 200 as f64) as f64;
                let v6_load = (buffer[4] as f64/ 200 as f64) as f64;
                // Add new loads to cache, divide by 2 to get load in percentage
                node.add_new_total_load(total_load);

                if v4_load > 1 as f64 {
                    node.set_v4_state(false);
                    //node.add_new_v4_load(100 as f64);
                } else {
                    node.set_v4_state(true);
                    node.add_new_v4_load(v4_load);
                }
                if v6_load > 1 as f64 {
                    node.set_v6_state(false);
                    //node.add_new_v6_load(100 as f64);
                } else {
                    node.set_v6_state(true);
                    node.add_new_v6_load(v6_load);
                }
                let new_avg = node.get_avg_total_load();
                println!("{:?}",new_avg);

                //Only execute load balancing if the averages changed
                if new_avg != old_avg {
                    println!("Average load changed!");
                    // Threshold
                    if config.lb_mode == 0 {
                        if total_load >= config.load_threshold {
                            println!("Passed threshold!");
                            // Node has at least one interfaces enabled
                            if node.get_v4_state() || node.get_v6_state() {
                                // Check if there are other addresses available
                                let mut other_addr_free = false;
                                for value in &v4_boxes {
                                    if value.0 != &node.get_v4_addr() && value.1 == &0 {
                                        other_addr_free = true;
                                        break;
                                    }
                                }
                                for value in &v6_boxes {
                                    if value.0 != &node.get_v6_addr() && value.1 == &0 {
                                        other_addr_free = true;
                                        break;
                                    }
                                }
                                if other_addr_free {
                                    // Both interfaces are enabled
                                    if node.get_v4_state() || node.get_v6_state() {
                                        node.check_rel_loads_and_shutdown(config.relv_threshold);
                                    } else if node.get_v6_state() {
                                        node.send_shutdown_v6();
                                    } else {
                                        node.send_shutdown_v4();
                                    }
                                }
                            }
                        } else {
                            if !node.get_v6_state() {
                                node.send_start_v6();
                            }
                            if !node.get_v4_state() {
                                node.send_start_v4();
                            }
                        }
                        // Round Robin
                    } else if config.lb_mode == 1 {
                        if new_avg > config.load_threshold {
                            // Find node with maxiumum load
                            let mut max_load = 0 as f64;
                            let mut max_node = 0;
                            for node in &nodes {
                                if node.1.get_avg_total_load() > max_load {
                                    max_load = node.1.get_avg_total_load();
                                    max_node = *node.0;
                                }
                            }
                            println!("New max node is {:?} ", max_node);
                            let mut active_node = false;
                            // Make sure another node is still active before shutting this one down
                            // Mostly relevant in cases where the interface cant be reused quickly enough after a switch
                            for node in &nodes {
                                if node.0 != &max_node && node.1.get_v4_state() {
                                    active_node = true;
                                    break;
                                }
                                if node.0 != &max_node && node.1.get_v6_state() {
                                    active_node = true;
                                    break;
                                }
                            }
                            for node in &nodes {
                                if node.0 == &max_node {
                                    if active_node {
                                        node.1.check_rel_loads_and_shutdown(config.relv_threshold);
                                    }
                                    if node.1.get_v4_state() && active_node {
                                        node.1.send_shutdown_v4();
                                        println!("Sending shutdown for v4!");
                                        // If we are over 80% capacity shut down v6 as well
                                        if v4_load >= 0.8 {
                                            node.1.send_shutdown_v6();
                                        }
                                    }
                                } else {
                                    if !node.1.get_v4_state() {
                                        println!("Sending start to v4 node {:?}", node.0);
                                        node.1.send_start_v4();
                                    }
                                    if !node.1.get_v6_state() {
                                        println!("Sending start to v6 node {:?}", node.0);
                                        node.1.send_start_v6();
                                    }
                                }
                            }
                        }
                    }
                }
                if ts.elapsed().unwrap().as_secs() >= config.log_interval {
                    log(&nodes, &mut fp);
                    ts = SystemTime::now();
                }
                println!("Node {:?} now has a total load of {:?}, with an v4 load of {:?} and a v6 load of {:?} ", buffer[1], total_load, v4_load, v6_load)
            }
            _ => {}
        }
    }

}

