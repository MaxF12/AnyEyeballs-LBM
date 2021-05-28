use std::net::{UdpSocket, Ipv4Addr, Ipv6Addr};
use anyeyeballs_orchestrator::{Node, send_error};
use std::collections::{HashMap, HashSet};
//use quiche::Connection;

const ORCH_ADDR: &str = "127.0.0.1:7722";
const MAX_NODES: u8 = 64;

fn main() {
    let mut nodes = HashMap::new();
    let socket = UdpSocket::bind(ORCH_ADDR).unwrap();
    let _quic_config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();
    let mut v4_boxes = HashMap::new();
    let mut v6_boxes = HashMap::new();
    //let conn = quiche::accept(&scid, None, &mut quic_config).unwrap();

    let mut buffer = [0; 512];
    loop {
        let sock = socket.try_clone().unwrap();
        println!("Orchestrator waiting for message");
        let (_num_bytes, addr) = sock.recv_from(&mut buffer).unwrap();
        match &buffer[0] {
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
                    let new_node = Node::new(sock, node_id, Ipv4Addr::new(buffer[1], buffer[2], buffer[3], buffer[4]), Ipv6Addr::from([buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15], buffer[16], buffer[17], buffer[18], buffer[19], buffer[20]]));
                    new_node.ok_join(addr);
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

            1 => {
                println!("Got new leave for node {:?}", buffer[1]);
                nodes.remove(&buffer[1]);
            }

            2 => {
                let node = nodes.get_mut(&buffer[1]).unwrap();
                println!("Got new status");
                match buffer[2] {
                    0 => {
                        if !node.get_v4_state() {
                            node.send_start(addr);
                            node.set_v4_state(true);
                            *v4_boxes.get_mut(&node.get_v4_addr()).unwrap() -= 1;
                        }
                    }
                    z => {
                        if z >= 5 {
                            if node.get_v4_state() {
                                //There are already nodes with this ip address that are deactivated, deactivate this one too
                                if v4_boxes[&node.get_v4_addr()] > 0 {
                                    node.send_shutdown(addr);
                                    node.set_v4_state(false);
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
                                        node.send_shutdown(addr);
                                        node.set_v4_state(false);
                                        *v4_boxes.get_mut(&node.get_v4_addr()).unwrap() += 1;
                                    }
                                }
                            }
                        }
                    },
                }
                println!("Node {:?} now has a total load of {:?}, with an v4 load of {:?} and a v6 load of {:?} ", buffer[1], buffer[2], buffer[3], buffer[4])
            }
            _ => {}
        }
    }

}
