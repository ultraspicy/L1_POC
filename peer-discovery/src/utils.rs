use std::net::UdpSocket;
use std::error::Error;

/// Retrieves the local IP address of the machine by creating a dummy UDP socket.
///
/// # Returns
/// A `Result` which is `Ok` containing a string with the IP address,
/// or an `Err` if there was an error retrieving the address.
pub fn get_local_ip() -> Result<String, Box<dyn Error>> {
    // Create a dummy UDP socket to determine the local IP address
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;

    let local_addr = socket.local_addr()?;
    let ip = local_addr.ip();

    match ip {
        std::net::IpAddr::V4(ipv4) => Ok(ipv4.to_string()),
        std::net::IpAddr::V6(ipv6) => Ok(ipv6.to_string()),
    }
}