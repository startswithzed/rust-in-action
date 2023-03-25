use rand::RngCore;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
struct MacAddress([u8; 6]); //array of 6 bytes

impl Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octet = &self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            octet[0], octet[1], octet[2],
            octet[3], octet[4], octet[5]
        ) // convert each byte to hexadecimal notation
    }
}

impl MacAddress {
    fn new() -> MacAddress {
        let mut octets: [u8; 6] = [0; 6];
        rand::thread_rng().fill_bytes(&mut octets);
        octets[0] |= 0b_0000_0011; // sets the MAC address to local and unicast
        MacAddress { 0: octets }
    }

    fn is_local(&self) -> bool {
        (self.0[0] & 0b_0000_0010) == 0b_0000_0010 // 0b_0000_0010 is number 2 in binary
    }

    fn is_unicast(&self) -> bool {
        (self.0[0] & 0b_0000_0001) == 0b_0000_0001 // number 1 in binary
    }
}

fn main() {
    let mac = MacAddress::new();
    assert!(mac.is_local());
    assert!(mac.is_unicast());
    println!("mac: {}", mac);
}
