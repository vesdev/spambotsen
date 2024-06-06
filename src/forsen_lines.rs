use csv::Reader;
use rand::{seq::SliceRandom, thread_rng};
use std::{error::Error, path::PathBuf};

fn parse_lines(path: PathBuf, forsen_lines: &mut ForsenLines) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;
    for result in rdr.records() {
        let record = result?;

        if let Some(line) = record.get(1) {
            let rarity = record.get(4).unwrap_or("undefined");
            forsen_lines.lines.push((
                line.trim().to_string() + &format!(" ({rarity})"),
                match rarity {
                    "Very rare" => 1,
                    "Rare" => 2,
                    "Uncommon" => 3,
                    "Common" => 4,
                    _ => 1,
                },
            ));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct ForsenLines {
    lines: Vec<(String, usize)>,
}

impl ForsenLines {
    pub fn new(path: PathBuf) -> Self {
        let mut result = Self { lines: Vec::new() };
        parse_lines(path, &mut result).expect("Error parsing csv");
        result
    }

    pub fn get_random(&self) -> String {
        let mut rng = thread_rng();
        self.lines
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0
            .clone()
    }
}
