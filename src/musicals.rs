include!("model.rs");
pub static MUSICALS: once_cell::sync::Lazy<Vec<Musical>> = once_cell::sync::Lazy::new(|| {
    use log::info;
    let musicals_csv = include_str!("musicals.csv");
    let mut musicals: Vec<Musical> = vec![];
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(musicals_csv.as_bytes());
    for record in reader.deserialize::<Musical>() {
        match record {
            Ok(musical) => musicals.push(musical),
            Err(err) => info!("{:?}", err),
        };
    }
    musicals
});
