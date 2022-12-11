extern crate cloudflareddns;

#[test]
fn test_get_external_ipv4() {
    // Call the get_external_ipv4() function
    let result = get_external_ipv4();

    // Verify that the result is an Ok variant containing a valid IPv4 address
    assert!(result.is_ok());
    assert!(result.unwrap().parse::<std::net::Ipv4Addr>().is_ok());
}

#[test]
fn test_get_external_ipv6() {
    // Call the get_external_ipv6() function
    let result = get_external_ipv6();

    // Verify that the result is an Ok variant containing a valid IPv6 address
    assert!(result.is_ok());
    assert!(result.unwrap().parse::<std::net::Ipv6Addr>().is_ok());
}
