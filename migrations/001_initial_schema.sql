-- Initial database schema

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Sensors table to store metadata about each sensor
CREATE TABLE sensors (
    sensor_id INTEGER PRIMARY KEY,
    sensor_name TEXT NOT NULL,
    sensor_type TEXT NOT NULL,  -- 'temperature', 'flow', 'power', etc.
    location TEXT,
    unit TEXT,                  -- 'C', 'L/min', 'kW', etc.
    threshold_min REAL,
    threshold_max REAL,
    calibration_date INTEGER,   -- Unix timestamp
    notes TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Create index on sensor_type for filtering
CREATE INDEX idx_sensors_type ON sensors(sensor_type);
CREATE INDEX idx_sensors_location ON sensors(location);

-- Readings table to store time-series data
CREATE TABLE readings (
    reading_id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,  -- Unix timestamp
    sensor_id INTEGER NOT NULL,
    value REAL,                  -- For analog sensors
    state INTEGER,               -- For digital/boolean sensors
    change_type TEXT,            -- 'periodic', 'event', etc.
    FOREIGN KEY (sensor_id) REFERENCES sensors(sensor_id) ON DELETE CASCADE
);

-- Create composite index for efficient time-series queries
CREATE INDEX idx_readings_sensor_time ON readings(sensor_id, timestamp);
-- Create index for time range queries
CREATE INDEX idx_readings_timestamp ON readings(timestamp);

-- Create view for the latest readings from each sensor
CREATE VIEW current_readings AS
SELECT r.*
FROM readings r
JOIN (
    SELECT sensor_id, MAX(timestamp) as max_timestamp
    FROM readings
    GROUP BY sensor_id
) latest ON r.sensor_id = latest.sensor_id AND r.timestamp = latest.max_timestamp;

-- Sessions table to track logging sessions
CREATE TABLE logging_sessions (
    session_id INTEGER PRIMARY KEY,
    sensor_id INTEGER NOT NULL,
    start_time INTEGER NOT NULL,  -- Unix timestamp
    end_time INTEGER,             -- NULL if session is ongoing
    sample_rate INTEGER,          -- Sampling frequency in seconds
    notes TEXT,
    FOREIGN KEY (sensor_id) REFERENCES sensors(sensor_id) ON DELETE CASCADE
);

-- Create index for active sessions
CREATE INDEX idx_sessions_active ON logging_sessions(sensor_id, end_time);

-- Trigger to update the updated_at field in sensors table
CREATE TRIGGER update_sensors_timestamp 
AFTER UPDATE ON sensors
BEGIN
    UPDATE sensors SET updated_at = (strftime('%s', 'now')) WHERE sensor_id = NEW.sensor_id;
END;

-- Trigger to automatically end any active sessions when a sensor is deleted
CREATE TRIGGER end_sessions_on_sensor_delete
BEFORE DELETE ON sensors
BEGIN
    UPDATE logging_sessions 
    SET end_time = (strftime('%s', 'now'))
    WHERE sensor_id = OLD.sensor_id AND end_time IS NULL;
END;