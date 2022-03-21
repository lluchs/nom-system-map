use anyhow::anyhow;
use criterion::{criterion_group, criterion_main, Criterion};

use nom_system_map::{system_map, SymbolMapping};

fn parse_with_std(file: &str) -> anyhow::Result<Vec<SymbolMapping>> {
    let mut result = Vec::new();
    for m in file.lines().map(|line| -> anyhow::Result<SymbolMapping> {
        let mut parts = line.split(' ');
        Ok(SymbolMapping {
            address: u64::from_str_radix(
                parts.next().ok_or_else(|| anyhow!("missing address"))?,
                16,
            )?,
            typ: parts
                .next()
                .ok_or_else(|| anyhow!("missing type"))?
                .chars()
                .nth(0)
                .ok_or_else(|| anyhow!("missing type"))?,
            symbol: parts.next().ok_or_else(|| anyhow!("missing symbol"))?,
        })
    }) {
        result.push(m?);
    }
    Ok(result)
}

fn get_system_map_file() -> String {
    use std::process::Command;

    let uname_output = Command::new("uname")
        .arg("-r")
        .output()
        .expect("failed running uname -r")
        .stdout;
    let uname = String::from_utf8_lossy(&uname_output);
    std::fs::read_to_string(format!(
        "/usr/lib/modules/{}/build/System.map",
        uname.trim_end()
    ))
    .expect("failed reading System.map")
}

fn criterion_benchmark(c: &mut Criterion) {
    let file = get_system_map_file();
    assert_eq!(parse_with_std(&file).unwrap(), system_map(&file).unwrap().1);
    c.bench_function("std", |b| b.iter(|| parse_with_std(&file)));
    c.bench_function("nom", |b| b.iter(|| system_map(&file)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
