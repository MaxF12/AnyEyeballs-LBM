use std::net::{UdpSocket, Ipv4Addr, Ipv6Addr};
use anyeyeballs_orchestrator::{Node, send_error, log};
use std::collections::{HashMap, HashSet};
use std::time;
use std::time::SystemTime;
use std::path::Path;
use std::fs::File;
//use quiche::Connection;

const ORCH_ADDR: &str = "127.0.0.1:7722";
const MAX_NODES: u8 = 64;

fn main() {
    let log_path = Path::new("log.txt");
    let mut fp = match File::create(&log_path) {
        Err(err) => panic!("Unable to create log file: {}", err),
        Ok(fp) => fp,
    };
    let mut nodes = HashMap::new();
    let socket = match UdpSocket::bind(ORCH_ADDR) {
        Err(err) => panic!("Unable to bind Orchestrator Socket: {}", err),
        Ok(socket) => socket,
    };
    //let _quic_config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();
    let mut v4_boxes = HashMap::new();
    let mut v6_boxes = HashMap::new();
    //let conn = quiche::accept(&scid, None, &mut quic_config).unwrap();

    let mut buffer = [0; 512];
    let mut ts = SystemTime::now();
    loop {
        let sock = socket.try_clone().unwrap();
        //println!("Orchestrator waiting for message");
        let (_num_bytes, addr) = sock.recv_from(&mut buffer).unwrap();
        match &buffer[0] {
            // New join
            0 => {
                println!("Got new join");
                let mut node_id: u8 = 0;
                while node_id < MAX_NODES {
                    if !nodes.contains_key(&node_id) {
                        break;
                    } else {
                        node_id += 1;
                    }
                }
                if node_id == MAX_NODES {
                    println!("Node list full!");
                    // Send error back
                    send_error(sock,addr, 0_u8);
                } else {
                    let new_node = Node::new(sock, addr, node_id, Ipv4Addr::new(buffer[1], buffer[2], buffer[3], buffer[4]), Ipv6Addr::from([buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15], buffer[16], buffer[17], buffer[18], buffer[19], buffer[20]]));
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
                let old_v4_avg = node.get_avg_v4_load();
                let old_v6_avg = node.get_avg_v6_load();
                let capacity = buffer[2];
                let v4_load = buffer[3];
                let v6_load = buffer[4];
                node.add_new_v4_load(v4_load as f64/capacity as f64);
                node.add_new_v6_load(v6_load as f64/capacity as f64);
                if buffer[5] == 1 {
                    node.set_v4_state(true);
                } else {
                    node.set_v4_state(false);
                }
                if buffer[6] == 1 {
                    node.set_v6_state(true);
                } else {
                    node.set_v6_state(false);
                }
                let new_v4_avg = node.get_avg_v4_load();
                let new_v6_avg = node.get_avg_v6_load();
                println!("{:?}",new_v6_avg);
                match v4_load {
                    0 => {
                        // Threshold
                        if false {
                            if !node.get_v4_state() {
                                node.send_start_v4();
                                *v4_boxes.get_mut(&node.get_v4_addr()).unwrap() -= 1;
                            }
                        }
                    }
                    z => {
                        // Threshold
                        if false {
                            if z >= 5 {
                                if node.get_v4_state() {
                                    //There are already nodes with this ip address that are deactivated, deactivate this one too
                                    if v4_boxes[&node.get_v4_addr()] > 0 {
                                        node.send_shutdown_v4();
                                        *v4_boxes.get_mut(&node.get_v4_addr()).unwrap() += 1;
                                    } else {
                                        let mut other_addr_free = false;
                                        for value in &v4_boxes {
                                            if value.0 != &node.get_v4_addr() && value.1 == &0 {
                                                other_addr_free = true;
                                                break;
                                            }
                                        }
                                        // If there is at least one other set of addresses available, turn this node off
                                        if other_addr_free {
                                            node.send_shutdown_v4();
                                            *v4_boxes.get_mut(&node.get_v4_addr()).unwrap() += 1;
                                        }
                                    }
                                }
                            }
                        };
                        // round-robin
                        if  true {
                            // Only update if one of the averages changed
                            if old_v4_avg != new_v4_avg {
                                println!("Average v4 changed!");
                                let mut min_load = 1 as f64;
                                let mut min_node = 0;
                                for node in &nodes {
                                    if node.1.get_avg_v4_load() < min_load {
                                        min_load = node.1.get_avg_v4_load();
                                        min_node = *node.0;
                                    }
                                }
                                println!("New min node for v4 is {:?} ", min_node);
                                for node in &nodes {
                                    if node.0 == &min_node {
                                        if !node.1.get_v4_state() {
                                            node.1.send_start_v4();
                                        }
                                    } else {
                                        if node.1.get_v4_state() && nodes.get(&min_node).unwrap().get_v4_state() {
                                            println!("Sending shutdown to node {:?}", node.0);
                                            node.1.send_shutdown_v4();
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
                match v6_load {
                    z => {
                        if  true {
                            if old_v6_avg != new_v6_avg {
                                println!("Average v6 changed!");
                                let mut min_load = 1 as f64;
                                let mut min_node = 0;
                                for node in &nodes {
                                    if node.1.get_avg_v6_load() < min_load {
                                        min_load = node.1.get_avg_v6_load();
                                        min_node = *node.0;
                                    }
                                }
                                println!("New min node for v6 is {:?} ", min_node);
                                for node in &nodes {
                                    if node.0 == &min_node {
                                        if !node.1.get_v6_state() {
                                            node.1.send_start_v6();
                                        }
                                    } else {
                                        if node.1.get_v6_state(){
                                            println!("Sending shutdown to node {:?}", node.0);
                                            node.1.send_shutdown_v6();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if ts.elapsed().unwrap().as_secs() >= 2 {
                    log(&nodes, &mut fp);
                    ts = SystemTime::now();
                }
                println!("Node {:?} now has a total load of {:?}, with an v4 load of {:?} and a v6 load of {:?} ", buffer[1], buffer[2], buffer[3], buffer[4])
            }
            _ => {}
        }
    }

}
