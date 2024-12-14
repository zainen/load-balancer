-- Add up migration script here
CREATE TABLE IF NOT EXISTS workers(
    worker_address VARCHAR(255) NOT NULL PRIMARY KEY
);

INSERT INTO workers(worker_address) VALUES 
('127.0.0.1:8000'),
('127.0.0.1:8001'),
('127.0.0.1:8002'),
('127.0.0.1:8003'),
('127.0.0.1:8004');
