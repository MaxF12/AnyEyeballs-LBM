//use quiche::Connection;
use std::net::{Ipv4Addr, Ipv6Addr, UdpSocket, SocketAddr};
use core::fmt;
use std::collections::{LinkedList, HashMap};
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::Write;

pub struct Node {
    quic_connection: UdpSocket,
    addr: SocketAddr,
    node_id: u8,
    ipv4: Ipv4Addr,
    ipv6: Ipv6Addr,
    total_loads: Vec<usize>,
    v4_loads: LinkedList<f64>,
    v6_loads: LinkedList<f64>,
    v4_state: bool,
    v6_state: bool,
}

impl Node {
    pub fn new(quic_connection: UdpSocket,addr: SocketAddr ,node_id: u8, ipv4: Ipv4Addr, ipv6: Ipv6Addr) -> Node {
        Node{
            quic_connection,
            addr,
            node_id,
            ipv4,
            ipv6,
            total_loads: vec![],
            v4_loads: LinkedList::new(),
            v6_loads: LinkedList::new(),
            v4_state: true,
            v6_state: true,
        }
    }

    pub fn ok_join (&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(2);
        // Flag 000 for join
        buf.push(4_u8);
        buf.push(self.node_id);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }

    pub fn get_v4_addr (&self) -> Ipv4Addr {
        self.ipv4
    }

    pub fn get_v6_addr (&self) -> Ipv6Addr {
        self.ipv6
    }

    pub fn get_v4_state (&self) -> bool {
        self.v4_state
    }

    pub fn get_v6_state (&self) -> bool {
        self.v6_state
    }

    pub fn set_v4_state (&mut self, state: bool) {
        self.v4_state = state;
    }

    pub fn set_v6_state (&mut self, state: bool) {
        self.v6_state = state;
    }

    pub fn add_new_v4_load (&mut self, load: f64) {
        self.v4_loads.push_back(load);
        if self.v4_loads.len() > 10 {
            self.v4_loads.pop_front();
        }
    }

    pub fn get_avg_v4_load (&self) -> f64 {
        if self.v4_loads.len() == 0 {
            0 as f64
        } else {
            self.v4_loads.iter().sum::<f64>() as f64 / self.v4_loads.len() as f64
        }
    }

    pub fn add_new_v6_load (&mut self, load: f64) {
        self.v6_loads.push_back(load);
        if self.v6_loads.len() > 10 {
            self.v6_loads.pop_front();
        }
    }

    pub fn get_avg_v6_load (&self) -> f64 {
        if self.v6_loads.len() == 0 {
            0 as f64
        } else {
            self.v6_loads.iter().sum::<f64>() as f64 / self.v6_loads.len() as f64
        }
    }

    pub fn send_shutdown_v4(&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(5);
        buf.push(3_u8);
        buf.push(self.node_id);
        buf.push(0_u8);
        buf.push(1_u8);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }

    pub fn send_start_v4(&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(5);
        buf.push(3_u8);
        buf.push(self.node_id);
        buf.push(2_u8);
        buf.push(1_u8);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }

    pub fn send_shutdown_v6(&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(5);
        buf.push(3_u8);
        buf.push(self.node_id);
        buf.push(1_u8);
        buf.push(0_u8);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }

    pub fn send_start_v6(&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(5);
        buf.push(3_u8);
        buf.push(self.node_id);
        buf.push(1_u8);
        buf.push(2_u8);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }

    pub fn send_shutdown_both(&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(5);
        buf.push(3_u8);
        buf.push(self.node_id);
        buf.push(0_u8);
        buf.push(0_u8);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }

    pub fn send_start_both(&self) {
        let mut buf:Vec<u8> = Vec::with_capacity(5);
        buf.push(3_u8);
        buf.push(self.node_id);
        buf.push(2_u8);
        buf.push(2_u8);
        self.quic_connection.send_to(&*buf,self.addr).unwrap();
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node ID: {}, IPv4: {:?},  IPv6: {:?}, total load: {:?}", self.node_id, self.ipv4, self.ipv6, self.total_loads)
    }
}

pub fn send_error(sock: UdpSocket, addr: SocketAddr, err: u8) {
    let mut buf:Vec<u8> = Vec::with_capacity(2);
    // Flag 000 for join
    buf.push(5_u8);
    buf.push(err);
    let sock = sock.try_clone().unwrap();
    sock.send_to(&*buf, addr).unwrap();
    println!("Writing buffer");

}
pub fn log(nodes: &HashMap<u8,Node>, fp: &mut File) {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut new_log = "{".to_owned()+ &ts.to_string();
    for node in nodes {
        new_log = new_log+",{"+&node.0.to_string()+","+&node.1.v4_loads.back().unwrap().to_string()+","+&node.1.v6_loads.back().unwrap().to_string()+"}";
    }
    new_log = new_log + "}\n";
    fp.write(new_log.as_bytes());
}