#![allow(dead_code)]

use std::{error::Error, time::SystemTime};

use clap::Parser;
use csv::{Reader, StringRecord, Writer};

#[derive(Debug)]
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
    /// Reads CSV data from a file.
    /// Returns an error if the file cannot be read.
    fn read_from_file(file_name: &str) -> Result<CSVData, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(file_name)?;
        let data: Vec<StringRecord> = reader.records().collect::<Result<_, _>>()?;
        let records = data.len();
        let fields = data.get(0).map_or(0, |record| record.len());
        let metadata = std::fs::metadata(file_name)?;
        Ok(CSVData {
            data,
            records,
            fields,
            pages: Vec::new(),
            file_name: file_name.to_string(),
            creation_date: metadata.created()?,
            last_modified_date: metadata.modified()?,
            file_size: metadata.len(),
        })
    }

    /// Creates pagination pages for the CSV data.
    /// Each page contains a range of records defined by `records_per_page`.
    pub fn create_pages(&mut self, records_per_page: usize) {
        let records_per_page = if records_per_page == 0 {
            10
        } else {
            records_per_page
        };
        self.pages.clear();
        let mut start = 0;
        while start < self.records {
            let end = std::cmp::min(start + records_per_page, self.records);
            self.pages.push(Page { start, end });
            start = end;
        }
        println!("Created {} pages", self.pages.len());
        println!("pages: {:#?}", self.pages);
    }

    /// Displays the CSV data to the terminal.
    fn display(&self) {
        for record in &self.data {
            println!("{record:#?}");
        }
    }

    /// Paginates the CSV data and writes it to the specified writer.
    pub fn paginate<W: std::io::Write>(
        &self,
        start: usize,
        end: usize,
        writer: &mut W,
    ) -> Result<(), std::io::Error> {
        for record in self.data[start..end].iter() {
            writeln!(writer, "{record:#?}").expect("Failed to write to writer");
        }
        Ok(())
    }

    /// Deletes a row at the specified index.
    /// The row is replaced with a row of empty strings
    /// The length of row matches the number of fields in CSV data
    /// => ensuring that the dimensions are maintained.
    /// Returns an error if the index is out of bounds.
    fn delete_row(&mut self, index: usize) -> Result<(), &'static str> {
        if index < self.records {
            let empty_row = vec!["".to_string(); self.fields]; // Create a row with empty strings
            self.data[index] = StringRecord::from(empty_row); // Replace the row at the specified index
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    /// Modifies a field at the specified row and field index.
    /// Returns an error if the row or field index is out of bounds.
    fn modify_field(&mut self, row: usize, field: usize, value: &str) -> Result<(), &'static str> {
        if row < self.records && field < self.fields {
            if let Some(record) = self.data.get_mut(row) {
                let mut new_row = record
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                if field < new_row.len() {
                    new_row[field] = value.to_string();
                    self.data[row] = StringRecord::from(new_row);
                    Ok(())
                } else {
                    Err("Field index out of bounds")
                }
            } else {
                Err("Row index out of bounds")
            }
        } else {
            Err("Row index or field index out of bounds")
        }
    }

    /// Writes the CSV data to a file.
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
    #[arg(long)]
    dimension: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Sets the number of records per page for pagination
    #[arg(short, long, default_value_t = 10)]
    records_per_page: usize,
}

/// Gets the dimensions of a CSV file if it's not provided by the user.
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

    println!("dbug = {}", cli.debug);

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    let mut csv_data = CSVData::read_from_file(&cli.file)?;

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
    // Paginate the data based on the records_per_page argument
    csv_data.create_pages(cli.records_per_page);

    // Display entire file
    println!("Displaying entire file:");
    csv_data.display();

    // Example of using paginate function
    println!("\nDisplaying paginated data (first page):");
    if let Some(first_page) = csv_data.pages.get(0) {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        csv_data.paginate(first_page.start, first_page.end, &mut handle)?;
    }

    // Example of deleting a row - deleting the first row
    println!("\n=========== Deleting the first row (index 0) ========== ");
    if let Err(e) = csv_data.delete_row(0) {
        println!("Error deleting row: {}", e);
    }
    println!("Data after deleting the first row:");
    csv_data.display();
    println!("========== End of DELETE demonstration ==========");

    // Example of modifying a field - modifying the first field of the second row
    println!("\nModifying a field (first field of the second row):");
    if let Err(e) = csv_data.modify_field(1, 0, "ModifiedValue") {
        println!("Error modifying field: {}", e);
    }
    println!("Data after modifying a field:");
    csv_data.display();
    println!("========== End of MODIFY FIELD demonstration ==========");

    // Example of writing data to a new file
    println!("\nWriting data to a new file 'output.csv' at the same level of project root....");
    if let Err(e) = csv_data.write_to_file("output.csv") {
        println!("Error writing to file: {}", e);
    }
    println!("Writing to file is done. Please check your file 'output.csv'.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> CSVData {
        let file_name = "testdata.csv";
        CSVData::read_from_file(file_name).expect("Failed to read test CSV file")
    }

    #[test]
    fn test_display() {
        let csv_data = setup();
        // Ensure no panic
        let res = std::panic::catch_unwind(|| csv_data.display());
        assert!(res.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_paginate() {
        let csv_data = setup();
        let mut buffer = Vec::new();
        let _ = csv_data.paginate(0, 3, &mut buffer);

        let output = String::from_utf8(buffer).expect("Not UTF-8");
        // println!("{}", output);

        assert!(output
            .contains("\"community\", \"block\", \"along\", \"telephone\", \"jar\", \"play\""));
        assert!(output.contains("StringRecord([\"environment\", \"managed\", \"valley\", \"potatoes\", \"there\", \"century\"])"));
        assert!(output.contains(
            "StringRecord([\"his\", \"soft\", \"breathing\", \"gun\", \"barn\", \"completely\"])"
        ));
    }

    #[test]
    fn test_delete_and_modify() {
        let mut csv_data = setup();
        let original_records = csv_data.records;
        let original_fields = csv_data.fields;

        // Test delete_row
        csv_data.delete_row(0).expect("Failed to delete row");
        assert_eq!(csv_data.records, original_records);
        assert_eq!(csv_data.data[0].len(), original_fields);

        // Test modify_field
        csv_data
            .modify_field(1, 1, "modified")
            .expect("Failed to modify field");
        assert_eq!(csv_data.records, original_records);
        assert_eq!(csv_data.fields, original_fields);
    }

    #[test]
    fn test_write_to_file() {
        let csv_data = setup();
        let output_file = "test_output.csv";
        csv_data
            .write_to_file(output_file)
            .expect("Failed to write to file");

        // Assert the file exists
        let metadata = std::fs::metadata(output_file);
        assert!(metadata.is_ok());
        assert!(metadata.unwrap().is_file());

        // Clean up the test file
        std::fs::remove_file(output_file).expect("Failed to remove test output file");
    }
}
