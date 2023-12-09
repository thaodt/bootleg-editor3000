use std::{time::SystemTime, error::Error};

use clap::Parser;
use csv::{Reader, StringRecord, Writer};

struct Page {
    start: usize,
    end: usize,
}

struct CSVData {
    data: Vec<StringRecord>,
    records: usize,
    fields: usize,
    pages: Vec<Page>,
    file_name: String,
    creation_date: SystemTime,
    last_modified_date: SystemTime,
    file_size: u64,
}

impl CSVData {
    fn read_from_file(file_name: &str) -> Result<CSVData, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(file_name)?;
        let data: Vec<StringRecord> = reader.records().collect::<Result<_, _>>()?;
        let metadata = std::fs::metadata(file_name)?;
        Ok(CSVData {
            data,
            records: data.len(),
            fields: data.get(0).map_or(0, |record| record.len()),
            pages: Vec::new(),
            file_name: file_name.to_string(),
            creation_date: metadata.created()?,
            last_modified_date: metadata.modified()?,
            file_size: metadata.len(),
        })
    }

    pub fn create_pages(&mut self, records_per_page: usize) {
        self.pages.clear();
        let mut start = 0;
        while start < self.records {
            let end = std::cmp::min(start + records_per_page, self.records);
            self.pages.push(Page { start, end });
            start = end;
        }
    }

    fn display(&self) {
        for record in &self.data {
            println!("{record:#?}");
        }
    }

    fn paginate(&self, start: usize, end: usize) {
        for record in self.data[start..end].iter() {
            println!("{record:#?}");
        }
    }

    // TODO: need to maitain dimensions
    fn delete_row(&mut self, index: usize) -> Result<(), &'static str> {
        if index < self.records {
            self.data.remove(index);
            self.records -= 1;
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    // TODO: need to maitain dimensions
    fn modify_field(&mut self, row: usize, field: usize, value: &str) -> Result<(), &'static str> {
        if row < self.records && field < self.fields {
            self.data[row][field] = *value;
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    fn write_to_file(&self, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = Writer::from_path(file_name)?;
        for record in &self.data {
            writer.write_record(record)?;
        }
        Ok(())
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets the input CSV file to use
    file: String,

    /// Sets the dimensions (rows, columns) of the CSV file
    #[arg(short, long)]
    dimension: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
    //#[command(subcommand)]
    //command: Option<Commands>,
}



fn get_dimensions(file_name: &str) -> Result<(usize, usize), Box<dyn Error>> {
    let mut reader = Reader::from_path(file_name)?;
    let records = reader.records();
    let rows = records.count();
    let mut reader = Reader::from_path(file_name)?; // Recreate the reader because counting the records consumes the iterator
    let columns = match reader.headers() {
        Ok(headers) => headers.iter().count(),
        Err(_) => 0,
    };
    Ok((rows, columns))
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    let mut csv_data = CSVData::read_from_file(&cli.file)?;

    // You can check the value provided by positional arguments, or option arguments
    if let Some(dimension) = cli.dimension.as_deref() {
        let dimensions: Vec<usize> = dimension
            .split(',')
            .map(|d| d.parse::<usize>().unwrap_or(0))
            .collect();
        if dimensions.len() == 2 {
            csv_data.records = dimensions[0];
            csv_data.fields = dimensions[1];
        }
    } else {
        let (rows, columns) = get_dimensions(&cli.file)?;
        csv_data.records = rows;
        csv_data.fields = columns;
    }

    Ok(())
}
