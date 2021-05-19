//use quiche::Connection;
use std::net::{Ipv4Addr, Ipv6Addr, UdpSocket, SocketAddr};
use core::fmt;

pub struct Node {
    quic_connection: UdpSocket,
    node_id: u8,
    ipv4: Ipv4Addr,
    ipv6: Ipv6Addr,
    total_loads: Vec<usize>,
    v4_loads: Vec<usize>,
    v6_loads: Vec<usize>
}

impl Node {
    pub fn new(quic_connection: UdpSocket, node_id: u8, ipv4: Ipv4Addr, ipv6: Ipv6Addr) -> Node {
        Node{
            quic_connection,
            node_id,
            ipv4,
            ipv6,
            total_loads: vec![],
            v4_loads: vec![],
            v6_loads: vec![]
        }
    }

    pub fn ok_join (&self, addr: SocketAddr) {
        let mut buf:Vec<u8> = Vec::with_capacity(2);
        // Flag 000 for join
        buf.push(4_u8);
        buf.push(self.node_id);
        self.quic_connection.send_to(&*buf,addr).unwrap();
        println!("Writing buffer");
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