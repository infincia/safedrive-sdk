use std;

use ::nom::{IResult, rest, le_u32};

use ::constants::*;

pub trait BinaryWriter {
    fn name(&self) -> String;
    fn as_binary(&self) -> Vec<u8>;
    fn should_include_data(&self) -> bool;
}


#[derive(Debug)]
pub struct BinaryFormat<'a> {
    pub magic: &'a str,
    pub file_type: &'a str,
    pub version: &'a str,
    pub compressed: bool,
    pub channel: Channel,
    pub production: bool,
    pub wrapped_key: &'a [u8],
    pub nonce: &'a [u8],
    pub wrapped_data: &'a [u8],
}

impl<'a> std::fmt::Display for BinaryFormat<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "BinaryFormat <type:{}, version:{}, payload:{}>", self.file_type, self.version, self.wrapped_data.len())
    }

}

pub fn binary_parse<'a>(input: &'a [u8]) -> IResult<&'a [u8], BinaryFormat<'a>> {
    chain!(input,
    magic: map_res!(tag!("sd"), std::str::from_utf8)                             ~
    file_type: map_res!(alt!(tag!("b") | tag!("s")), std::str::from_utf8)        ~
    version: map_res!(take!(2), std::str::from_utf8)                             ~
    flags: bits!(tuple!(take_bits!(u8, 3), take_bits!(u8, 1), take_bits!(u8, 1), take_bits!(u8, 1), take_bits!(u8, 1), take_bits!(u8, 1) ))~
    take!(2)                                                                     ~
    wrapped_key: take!(SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE)                  ~
    nonce: take!(SECRETBOX_NONCE_SIZE)                                           ~
    wrapped_data: rest                                                           ,
    || {
    BinaryFormat {
        magic: magic,
        file_type: file_type,
        version: version,
        compressed: flags.1 == 1,
        channel: {
            if flags.2 == 1 {
                Channel::Nightly
            } else if flags.3 == 1 {
                Channel::Beta
            } else if flags.4 == 1 {
                Channel::Stable
            } else {
                Channel::Nightly
            }
        },
        production: flags.5 == 1,
        wrapped_key: wrapped_key,
        nonce: nonce,
        wrapped_data: wrapped_data,
      }
    }
  )
}

pub fn remove_padding<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    chain!(input,
        length: le_u32      ~
        data: take!(length) ,
    || {
        (data)
    })
}

named!(hmac_parse<&[u8], &[u8]>, do_parse!(
    hmac: take!(HMAC_SIZE) >>
    (hmac)
  )
);

pub fn parse_hmacs<'a, T: AsRef<[u8]>>(input: &'a T) -> IResult<&'a [u8], Vec<&'a [u8]>> {
    let mut hmacs = vec![];
    let (mut d, hmac) = try_parse!(input.as_ref(), hmac_parse);
    hmacs.push(hmac);
    while d.len() > 0 {
        let (remaining, hmac) = try_parse!(d, hmac_parse);
        d = remaining;
        hmacs.push(hmac);
    }
    IResult::Done(d, hmacs)
}


