use std::{error::Error, net::{IpAddr, Ipv4Addr}};

use regex::Regex;

#[derive(Debug)]
struct ErrorWithMessage(&'static str);

impl std::fmt::Display for ErrorWithMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for ErrorWithMessage { }

pub fn ipv4_from_str(ip: &str) -> Result<IpAddr, Box<dyn Error>> {
    let regex = Regex::new(r"^(?P<a>\d+)\.(?P<b>\d+)\.(?P<c>\d+)\.(?P<d>\d+)$")?;
    let captures = regex.captures(ip).ok_or(ErrorWithMessage("Invalid ipv4 string"))?;

    let a = captures.name("a").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;
    let b = captures.name("b").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;
    let c = captures.name("c").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;
    let d = captures.name("d").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;

    Ok(IpAddr::V4(Ipv4Addr::new(a, b, c, d)))
}
