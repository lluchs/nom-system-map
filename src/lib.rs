use nom::{
    bytes::complete::{is_not, take},
    character::complete::{anychar, multispace1, newline},
    combinator::{map, map_res},
    multi::many1,
    sequence::pair,
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct SymbolMapping<'a> {
    pub address: u64,
    pub typ: char,
    pub symbol: &'a str,
}

fn address_from_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 16)
}

pub(crate) fn address(input: &str) -> IResult<&str, u64> {
    map_res(take(16usize), address_from_hex)(input)
}

fn typ(input: &str) -> IResult<&str, char> {
    anychar(input)
}

pub(crate) fn symbol(input: &str) -> IResult<&str, &str> {
    is_not("\n")(input)
}

/// Parse a single entry from `System.map`.
pub fn symbol_mapping(input: &str) -> IResult<&str, SymbolMapping> {
    let (input, address) = address(input)?;
    let (input, _) = multispace1(input)?;
    let (input, typ) = typ(input)?;
    let (input, _) = multispace1(input)?;
    let (input, symbol) = symbol(input)?;
    Ok((
        input,
        SymbolMapping {
            address,
            typ,
            symbol,
        },
    ))
}

/// Parse a complete `System.map` file.
pub fn system_map(input: &str) -> IResult<&str, Vec<SymbolMapping>> {
    many1(map(pair(symbol_mapping, newline), |(sm, _)| sm))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        assert_eq!(address("0000000000000000 "), Ok((" ", 0)));
        assert_eq!(address("ffffffff83532558 "), Ok((" ", 0xffffffff83532558)));
    }

    #[test]
    fn test_symbol() {
        assert_eq!(symbol("__per_cpu_start\n"), Ok(("\n", "__per_cpu_start")));
    }

    #[test]
    fn test_symbol_mapping() {
        assert_eq!(
            symbol_mapping("0000000000000000 D __per_cpu_start\n"),
            Ok((
                "\n",
                SymbolMapping {
                    address: 0,
                    typ: 'D',
                    symbol: "__per_cpu_start"
                }
            ))
        );
    }

    #[test]
    fn test_system_map() {
        use std::process::Command;

        let uname_output = Command::new("uname")
            .arg("-r")
            .output()
            .expect("failed running uname -r")
            .stdout;
        let uname = String::from_utf8_lossy(&uname_output);
        let system_map_str = std::fs::read_to_string(format!(
            "/usr/lib/modules/{}/build/System.map",
            uname.trim_end()
        ))
        .expect("failed reading System.map");
        let (rest, _sm) = system_map(&system_map_str).expect("failed parsing System.map");
        assert_eq!(rest, "");
    }
}
