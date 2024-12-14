# Load Balancer

A high-performance load balancer built using Rust and asynchronous programming libraries, designed to distribute incoming network traffic across multiple backend services for improved scalability and reliability.

## Features

- **Asynchronous Networking**: Leverages `tokio` for high-performance asynchronous operations.
    
- **Dynamic Backend Management**: Add or remove backend servers on the fly.
    
- **Customizable Load Balancing Algorithms**: Includes implementations for round-robin, random, and weighted algorithms.
    
- **Tracing and Observability**: Integrated with `tracing` and `tracing-subscriber` for comprehensive logging and diagnostics.
    
- **Database Integration**: Supports PostgreSQL with migrations using `sqlx`.
    

## Installation

1. Ensure you have Rust installed. If not, install it via [rustup](https://rustup.rs/):
    
    ```
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    
2. Clone the repository:
    
    ```
    git clone https://github.com/yourusername/load-balancer.git
    cd load-balancer
    ```
    
3. Build the project:
    
    ```
    cargo build --release
    ```
    

## Usage

### Running the Load Balancer

1. Set up your environment variables using a `.env` file. Refer to the Configuration section for required variables.
    
2. Start the load balancer:
    
    ```
    cargo run --bin load_balancer
    ```
    

### Example Configuration

Create a `.env` file in the root directory with the following content:

```
DATABASE_URL=postgres://user:password@localhost/db_name
```

### Using `sqlx-cli` and Migrations

1. Install `sqlx-cli`:
    
    ```
    cargo install sqlx-cli --features postgres
    ```
    
2. Create a new migration:
    
    ```
    sqlx migrate add migration_name
    ```
    
3. Edit the migration files in the `migrations` directory to define the database schema changes.
    Example:

    ```
        CREATE TABLE IF NOT EXISTS workers(
            worker_address VARCHAR(255) NOT NULL PRIMARY KEY
        );

        INSERT INTO workers(worker_address) VALUES 
        ('127.0.0.1:8000'),
        ('127.0.0.1:8001'),
        ('127.0.0.1:8002'),
        ('127.0.0.1:8003'),
        ('127.0.0.1:8004');
    ```
    
4. Apply the migrations:
    
    ```
    sqlx migrate run
    ```
    

## Load Balancing Algorithms

You can customize the algorithm used for distributing traffic by editing the configuration in `src/lib.rs`. Supported algorithms:

- **Round Robin**: Default algorithm for evenly distributing requests.
    
- **Random**: Randomly selects a backend.
    
- **Weighted**: Distributes requests based on predefined weights.
    

## Dependencies

This project leverages the following dependencies:

- `[tokio](https://tokio.rs/)`: Asynchronous runtime for the Rust programming language.
    
- `[futures](https://docs.rs/futures/)`: Asynchronous programming utilities.
    
- `[rand](https://docs.rs/rand/)`: Random number generation for load balancing.
    
- `[tracing](https://docs.rs/tracing/)`: Structured logging for asynchronous Rust applications.
    
- `[color-eyre](https://docs.rs/color-eyre/)`: Enhanced error reporting with colorful output.
    
- `[sqlx](https://docs.rs/sqlx/)`: Asynchronous SQL toolkit and ORM.
    
- `[dotenvy](https://docs.rs/dotenvy/)`: Loads environment variables from `.env`.
    
- `[async-trait](https://docs.rs/async-trait/)`: Allows async functions in traits.
    
- `[lazy_static](https://docs.rs/lazy_static/)`: Defines statics that require complex initialization.
    

## TODOS
- create docker build
- dynamically check for available servers and health check on server changes
- Improve error handling and leveraging 


