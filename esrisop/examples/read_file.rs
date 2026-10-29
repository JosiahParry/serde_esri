use bytes::Bytes;
use esrisop::{decoder::Decoder, query::Query};
use parquet::{
    arrow::arrow_reader::{ArrowReaderMetadata, ArrowReaderOptions},
    file::metadata::{PageIndexPolicy, ParquetMetaDataReader},
    DecodeResult,
};
use serde_esri::enginex::geometry::Envelope2D;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let path = args
        .first()
        .ok_or("usage: read_file <path> [xmin ymin xmax ymax] [resolution]")?;
    let numbers = args[1..]
        .iter()
        .map(|a| a.parse::<f64>())
        .collect::<Result<Vec<_>, _>>()?;
    let query = Query {
        extents: match numbers.get(..4) {
            Some(&[xmin, ymin, xmax, ymax]) => vec![Envelope2D {
                xmin,
                ymin,
                xmax,
                ymax,
            }],
            _ => Vec::new(),
        },
        resolution: numbers.get(4).copied(),
        columns: None,
    };

    let bytes = Bytes::from(std::fs::read(path)?);
    let metadata = ParquetMetaDataReader::new()
        .with_page_index_policy(PageIndexPolicy::Optional)
        .parse_and_finish(&bytes)?;
    let metadata = ArrowReaderMetadata::try_new(Arc::new(metadata), ArrowReaderOptions::new())?;
    let mut decoder = Decoder::try_new(metadata, &query)?;
    for field in decoder.schema().fields() {
        println!("{}: {}", field.name(), field.data_type());
    }

    let (mut rows, mut fetched) = (0, 0);
    loop {
        match decoder.try_decode()? {
            DecodeResult::NeedsData(ranges) => {
                fetched += ranges.iter().map(|r| r.end - r.start).sum::<u64>();
                let data = ranges
                    .iter()
                    .map(|r| bytes.slice(r.start as usize..r.end as usize))
                    .collect();
                decoder.push_ranges(ranges, data)?;
            }
            DecodeResult::Data(batch) => rows += batch.num_rows(),
            DecodeResult::Finished => break,
        }
    }
    println!("{rows} rows from {fetched} of {} bytes", bytes.len());
    Ok(())
}
