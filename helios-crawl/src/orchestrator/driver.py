import ray
import subprocess
import json
import time

# Initialize Ray (connects to K8s head)
ray.init(address="auto")

@ray.remote(num_cpus=1)
class HeliosWorker:
    def crawl_and_index(self, urls: list):
        results = []
        
        # 1. Crawl Phase (TypeScript/Playwright)
        for url in urls:
            try:
                # Call the TS binary
                process = subprocess.run(
                    ["node", "/app/src/crawler-ts/dist/worker.js", url],
                    capture_output=True, text=True, timeout=60
                )
                output = json.loads(process.stdout)
                if "content" in output:
                    results.append(output)
            except Exception as e:
                print(f"Failed to crawl {url}: {e}")

        # 2. Indexing Phase (Rust/Lance)
        if results:
            try:
                # Pass batch to Rust for embedding and Lance storage
                subprocess.run(
                    ["/app/target/release/helios-engine", json.dumps(results)],
                    check=True
                )
                return f"Processed {len(results)} pages"
            except Exception as e:
                return f"Rust engine failure: {e}"
        return "No valid data to process"

# Simulation of a 100M page queue
if __name__ == "__main__":
    # In prod, this reads from Kafka or Redis
    batch_queue = [
        ["https://news.ycombinator.com", "https://rust-lang.org"],
        ["https://ray.io", "https://typescriptlang.org"]
    ]

    workers = [HeliosWorker.remote() for _ in range(2)]
    
    # Scatter-Gather
    futures = [w.crawl_and_index.remote(batch) for w, batch in zip(workers, batch_queue)]
    
    print(ray.get(futures))