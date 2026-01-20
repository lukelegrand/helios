# Helios: Distributed Ray-Lance Web Crawler

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0-blue?style=flat-square)
![Ray](https://img.shields.io/badge/Orchestrator-Ray-028CF0?style=flat-square)
![Lance](https://img.shields.io/badge/VectorDB-Lance-green?style=flat-square)

**Helios** is a massive-scale distributed web crawler and indexing engine designed to ingest 100M+ pages/day. It leverages **Ray** for distributed orchestration on Kubernetes, **TypeScript/Playwright** for high-fidelity rendering (handling dynamic JS/anti-bot measures), and **Rust/Lance** for high-performance vector storage.

## 🚀 Architecture

The system follows a decoupled micro-processing architecture:

1.  **Orchestrator (Python/Ray)**: Manages the distributed task queue, handles fault tolerance, and autoscales workers on AWS EKS.
2.  **Crawler (TypeScript/Playwright)**: Headless workers communicating via CDP (Chrome DevTools Protocol) to bypass bot detection and render dynamic content.
3.  **Data Engine (Rust)**: A highly optimized binary that cleans HTML, generates embeddings, and writes directly to **LanceDB** formats on S3 without Python overhead.

## 🛠 Tech Stack & Design Choices

* **Ray on Kubernetes**: Chosen over static K8s Jobs to allow dynamic task scheduling and actor-based state management for politeness/rate-limiting across domains.
* **Playwright (TS)**: selected for its robust CDP integration, allowing us to inspect network traffic and modify browser fingerprints on the fly.
* **Rust & Lance**: We bypass standard Python bottlenecks by performing HTML parsing and vector disk I/O strictly in Rust. Lance provides columnar storage optimized for random access and vector search.

## ⚡ Performance

* **Concurrency**: Capable of scaling to 10k+ concurrent browser contexts.
* **Storage**: Direct-to-S3 writes using Arrow IPC/Lance for zero-copy data persistence.
* **Politeness**: Distributed semaphore implementation to ensure domain rate limits are respected globally.

## 📦 Installation & Deployment

### Prerequisites
* Kubernetes Cluster (EKS/GKE)
* S3 Bucket for Lakehouse storage

### Build
```bash
# Build the unified worker image containing Node, Rust binary, and Python
docker build -t helios-worker:latest .