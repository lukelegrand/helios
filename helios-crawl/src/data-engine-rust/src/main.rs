use std::env;
use std::sync::Arc;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, FixedSizeListArray, Float32Array};
use arrow_schema::{DataType, Field, Schema};
use lance::dataset::{Dataset, WriteParams};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct PageData {
    url: String,
    content: String,
}

// Mock embedding function - in production, this would link to ONNX/Candle
fn mock_embed(_text: &str) -> Vec<f32> {
    vec![0.1; 384] 
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Receive data from Ray (passed as JSON string argument for loose coupling)
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: helios-engine <json_payload>");
        return Ok(());
    }
    
    let payload = &args[1];
    let pages: Vec<PageData> = serde_json::from_str(payload)?;

    // 2. Process Data (Rust speed)
    let mut urls = Vec::new();
    let mut vectors = Vec::new();

    for page in pages {
        // Perform heavy text cleaning here
        let clean_text = page.content.replace("\n", " "); 
        let embedding = mock_embed(&clean_text);
        
        urls.push(page.url);
        vectors.extend(embedding);
    }

    // 3. Prepare Arrow Arrays
    let uri_array = StringArray::from(urls);
    let vector_values = Float32Array::from(vectors);
    let vector_array = FixedSizeListArray::try_new_from_values(vector_values, 384)?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("uri", DataType::Utf8, false),
        Field::new("vector", DataType::FixedSizeList(
            Arc::new(Field::new("item", DataType::Float32, true)), 384), 
        true),
    ]));

    let batch = RecordBatch::try_new(schema.clone(), vec![
        Arc::new(uri_array),
        Arc::new(vector_array),
    ])?;

    // 4. Write to Lance (S3)
    // In a real run, this path is s3://...
    let uri = "/tmp/helios_lance_db"; 
    let params = WriteParams::default();
    
    Dataset::write(
        RecordBatchIterator::new(vec![Ok(batch)], schema),
        uri,
        Some(params),
    ).await?;

    println!("Successfully indexed batch to LanceDB");
    Ok(())
}