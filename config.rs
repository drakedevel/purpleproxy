use rustc_serialize::{Decoder, Decodable};
use std::collections::{HashMap, HashSet};
use std::error::FromError;
use std::io;
use std::result;
use toml;

#[derive(Debug)]
pub struct TypeInfo {
    pub ret: String,
}

impl Decodable for TypeInfo {
    fn decode<D: Decoder>(d: &mut D) -> result::Result<TypeInfo, D::Error> {
        d.read_map(|d, n| {
            let mut ret = Err(d.error("missing return field"));

            for i in 0..n {
                match d.read_map_elt_key(i, |d| d.read_str()) {
                    Ok(ref key) if *key == "return" =>
                        ret = d.read_map_elt_val(i, |d| d.read_str()),
                    _ => continue,
                }
            }

            Ok(TypeInfo {
                ret: try!(ret),
            })
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub typeinfo: HashMap<String, TypeInfo>,
    pub proxy: HashSet<String>,
    pub passthrough: HashSet<String>,
    pub skip: HashSet<String>,
}

impl Decodable for Config {
    fn decode<D: Decoder>(d: &mut D) -> result::Result<Config, D::Error> {
        fn decode_seq_to_hash_set<D: Decoder>(d: &mut D, i: usize) -> result::Result<HashSet<String>, D::Error> {
            d.read_map_elt_val(i, |d| {
                d.read_seq(|d, n| {
                    let mut set = HashSet::new();

                    for i in 0..n {
                        set.insert(try!(d.read_seq_elt(i, |d| d.read_str())));
                    }

                    Ok(set)
                })
            })
        }

        fn decode_config<D: Decoder>(d: &mut D, i: usize, config: &mut Config) -> result::Result<(), D::Error> {
            d.read_map_elt_val(i, |d| {
                d.read_map(|d, n| {
                    for i in 0..n {
                        match d.read_map_elt_key(i, |d| d.read_str()) {
                            Ok(ref key) if *key == "proxy" => {
                                config.proxy = try!(decode_seq_to_hash_set(d, i));
                            }
                            Ok(ref key) if *key == "passthrough" => {
                                config.passthrough = try!(decode_seq_to_hash_set(d, i));
                            }
                            Ok(ref key) if *key == "skip" => {
                                config.skip = try!(decode_seq_to_hash_set(d, i));
                            }
                            _ => continue,
                        }
                    }
                    
                    Ok(())
                })
            })
        }

        fn decode_typeinfo<D: Decoder>(d: &mut D, i: usize, config: &mut Config) -> result::Result<(), D::Error> {
            d.read_map_elt_val(i, |d| {
                d.read_map(|d, n| {
                    for i in 0..n {
                        let key = try!(d.read_map_elt_key(i, |d| d.read_str()));

                        let typeinfo = try! {
                            d.read_map_elt_val(i, |d| {
                                <TypeInfo as Decodable>::decode(d)
                            })
                        };

                        config.typeinfo.insert(key, typeinfo);
                    }

                    Ok(())
                })
            })
        }

        d.read_map(|d, n| {
            let mut config = Config {
                typeinfo: HashMap::new(),
                proxy: HashSet::new(),
                passthrough: HashSet::new(),
                skip: HashSet::new(),
            };

            for i in 0..n {
                match d.read_map_elt_key(i, |d| d.read_str()) {
                    // Ok("config") => try!(decode_config(d, i, &mut config)),
                    // Ok("typeinfo") => try!(decode_typeinfo(d, i, &mut config)),
                    Ok(ref key) if *key == "config" =>
                        try!(decode_config(d, i, &mut config)),
                    Ok(ref key) if *key == "typeinfo" =>
                        try!(decode_typeinfo(d, i, &mut config)),
                    _ => continue,
                };
            }

            Ok(config)
        })
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseError(Vec<toml::ParserError>),
    DecodeError(toml::DecodeError),
}

impl FromError<io::Error> for Error {
    fn from_error(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl FromError<Vec<toml::ParserError>> for Error {
    fn from_error(e: Vec<toml::ParserError>) -> Error {
        Error::ParseError(e)
    }
}

impl FromError<toml::DecodeError> for Error {
    fn from_error(e: toml::DecodeError) -> Error {
        Error::DecodeError(e)
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn parse_config<R: io::Read>(mut reader: R) -> Result<Config> {
    let mut s = String::new();
    try!(reader.read_to_string(&mut s));

    let mut parser = toml::Parser::new(&s);
    let toml = try! {
        match parser.parse() {
            Some(toml) => Ok(toml),
            None => Err(parser.errors),
        }
    };

    Ok(try!(Decodable::decode(&mut toml::Decoder::new(toml::Value::Table(toml)))))
}
