//use quiche::Connection;
use std::net::{Ipv4Addr, Ipv6Addr, UdpSocket, SocketAddr};
use core::fmt;

pub struct Node<'a> {
    quic_connection: &'a UdpSocket,
    node_id: u8,
    ipv4: Ipv4Addr,
    ipv6: Ipv6Addr,
    total_loads: Vec<usize>,
    v4_loads: Vec<usize>,
    v6_loads: Vec<usize>
}

impl Node <'_>{
    pub fn new(quic_connection: &UdpSocket, node_id: u8, ipv4: Ipv4Addr, ipv6: Ipv6Addr) -> Node {
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
        let mut buf:Vec<u8> = Vec::with_capacity(3);
        // Flag 000 for join
        buf.push(4_u8);
        buf.push(self.node_id);
        self.quic_connection.connect(addr).unwrap();
        self.quic_connection.send(&*buf).unwrap();
        println!("Writing buffer");
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node ID: {}, IPv4: {:?},  IPv6: {:?}, total load: {:?}", self.node_id, self.ipv4, self.ipv6, self.total_loads)
    }
}