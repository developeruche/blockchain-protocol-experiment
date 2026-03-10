#[cfg(test)]
mod tests {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use bytes::Bytes;

    #[tokio::test]
    async fn test_parquet_schema() {
        let url = "https://data.ethpandaops.io/xatu/mainnet/databases/default/canonical_execution_block/1000/21500000.parquet"\;
        let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
        
        let reader = SerializedFileReader::new(bytes).unwrap();
        let metadata = reader.metadata();
        
        for (i, field) in metadata.file_metadata().schema_descr().columns().iter().enumerate() {
            println!("Column {}: {} - {:?}", i, field.name(), field.physical_type());
        }
    }
}
