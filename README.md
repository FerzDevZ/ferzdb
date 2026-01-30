# FerzDB

**FerzDB** adalah database SQL modern yang dirancang untuk web umum, embedded systems, dan offline-first scenarios.

## ✨ Features

- 🔧 **Embedded & Server Mode**: Gunakan sebagai library atau client-server
- 📴 **Offline-First**: Dirancang untuk bekerja tanpa koneksi permanen
- 🔍 **Transparent Storage**: File format yang bisa di-inspect manual
- ⚡ **B-Tree Indexing**: Akses data cepat dengan B-Tree
- 💾 **ACID Transactions**: Transaction dengan WAL & crash recovery
- 🛠️ **Developer-Friendly**: Error messages jelas, dokumentasi lengkap

## 🚀 Quick Start

### Embedded Mode

```rust
use ferzdb::Database;

fn main() -> Result<(), ferzdb::FerzError> {
    // Create/open database
    let mut db = Database::open("myapp.fdb")?;
    
    // Create table
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")?;
    
    // Insert data
    db.execute("INSERT INTO users VALUES (1, 'Alice', 25)")?;
    db.execute("INSERT INTO users VALUES (2, 'Bob', 30)")?;
    
    // Query data
    let results = db.query("SELECT * FROM users WHERE age > 20")?;
    for row in results {
        println!("{:?}", row);
    }
    
    Ok(())
}
```

### CLI Mode

```bash
# Build
cargo build --release

# Run CLI
./target/release/ferzcli myapp.fdb

# Execute SQL
ferzcli> CREATE TABLE users (id INTEGER, name TEXT);
ferzcli> INSERT INTO users VALUES (1, 'Alice');
ferzcli> SELECT * FROM users;
ferzcli> .quit
```

## 📦 Installation

```bash
# Clone repository
git clone https://github.com/yourusername/ferzdb.git
cd ferzdb

# Build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## 📚 Documentation

- [Architecture](docs/architecture.md) - System design & components
- [SQL Reference](docs/sql_reference.md) - Supported SQL syntax
- [API Reference](docs/api_reference.md) - Embedded mode API
- [Tutorial](docs/tutorial.md) - Getting started guide

## 🗺️ Roadmap

- **v0.1** (Current): Foundation - Embedded mode, basic SQL, B-Tree, WAL
- **v0.2**: Server mode, TCP protocol, authentication
- **v0.3**: Advanced SQL (JOIN, subqueries, aggregates)
- **v0.4**: Offline-first sync protocol
- **v0.5**: Performance (MVCC, optimization)
- **v1.0**: Production-ready release

## 💻 Technology

- **Language**: Rust 🦀
- **Storage**: Page-based (8KB pages)
- **Index**: B-Tree
- **Durability**: Write-Ahead Logging (WAL)
- **Concurrency**: Lock-based (v0.1), MVCC (v0.5+)

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Dual-licensed under MIT OR Apache-2.0

## 🙏 Acknowledgments

Inspired by SQLite, PostgreSQL, and modern database research.

---

**Status**: 🚧 **Early Development** (v0.1)

**Author**: FerzDB Team  
**Website**: https://ferzdb.dev (coming soon)
