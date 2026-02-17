use anyhow::{anyhow, Context, Result};
use clap::Parser;
use csv::{ReaderBuilder, WriterBuilder, StringRecord};
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(default_value = "all.csv")]
    input: String,
    #[arg(default_value = "test.in")]
    output: String,
    #[arg(long)]
    series: String,
    #[arg(long, default_value = "R-T")]
    r_kind: String,
}

fn parse_f64(s: &str) -> Result<f64> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("empty numeric field"));
    }
    let normalized = s.replace(',', ".");
    normalized
        .parse::<f64>()
        .with_context(|| format!("cannot parse float: '{s}'"))
}

fn find_col(
    types_row: &StringRecord,
    series_row: &StringRecord,
    series_query: &str,
    wanted_type: &str,
) -> Result<usize> {
    let q = series_query.to_lowercase();
    let mut matches: Vec<usize> = Vec::new();
    for i in 0..types_row.len() {
        let t = types_row.get(i).unwrap_or("");
        let s = series_row.get(i).unwrap_or("");
        if t == wanted_type && s.to_lowercase().contains(&q) {
            matches.push(i);
        }
    }
    match matches.len() {
        0 => Err(anyhow!(
            "Nie znaleziono kolumny typu '{}' dla serii zawierającej '{}'",
            wanted_type,
            series_query
        )),
        1 => Ok(matches[0]),
        _ => Err(anyhow!(
            "Znaleziono wiele kolumn typu '{}' dla serii '{}': indeksy {:?}. \
             Doprecyzuj --series (bardziej unikalny fragment).",
            wanted_type,
            series_query,
            matches
        )),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = File::open(&args.input)
        .with_context(|| format!("Nie mogę otworzyć pliku wejściowego: {}", args.input))?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(true)
        .from_reader(input);
    let mut rec0 = StringRecord::new();
    let mut rec1 = StringRecord::new();
    let mut rec2 = StringRecord::new();
    rdr.read_record(&mut rec0)
        .context("Brak pierwszej linii nagłówka")?;
    rdr.read_record(&mut rec1)
        .context("Brak drugiej linii nagłówka")?;
    rdr.read_record(&mut rec2)
        .context("Brak trzeciej linii nagłówka")?;
    let col_nm = 0usize;
    let col_ev = 1usize;
    let col_t = find_col(&rec0, &rec2, &args.series, "T")?;
    let col_r = find_col(&rec0, &rec2, &args.series, &args.r_kind)?;
    let output = File::create(&args.output)
        .with_context(|| format!("Nie mogę utworzyć pliku wyjściowego: {}", args.output))?;
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_writer(output);
    let mut data_rec = StringRecord::new();
    while rdr.read_record(&mut data_rec)? {
        if data_rec.len() < 2 {
            continue;
        }
        let nm = parse_f64(data_rec.get(col_nm).unwrap_or(""))?;
        let ev = parse_f64(data_rec.get(col_ev).unwrap_or(""))?;
        let t = parse_f64(data_rec.get(col_t).unwrap_or(""))?;
        let r = parse_f64(data_rec.get(col_r).unwrap_or(""))?;
        wtr.write_record([
            format!("{:.6}", ev),
            format!("{:.3}", nm),
            format!("{:.8}", t),
            format!("{:.8}", r),
        ])?;
    }
    wtr.flush()?;
    eprintln!(
        "OK -> zapisano {} (E[eV], lambda[nm], T, R) dla serii zawierającej '{}' (R z '{}')",
        args.output, args.series, args.r_kind
    );
    eprintln!("Użyte kolumny: E=1, lambda=0, T={}, R={}", col_t, col_r);
    Ok(())
}
