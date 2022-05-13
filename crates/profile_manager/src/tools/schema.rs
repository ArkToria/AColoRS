pub const NODE_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS nodes(
                    ID INTEGER PRIMARY KEY AUTOINCREMENT,
                    Name TEXT NOT NULL,
                    GroupID INTEGER NOT NULL,
                    GroupName TEXT NOT NULL,
                    RoutingID INTEGER NOT NULL,
                    RoutingName TEXT NOT NULL,
                    Protocol TEXT NOT NULL,
                    Address TEXT NOT NULL,
                    Port INTEGER NOT NULL,
                    Password TEXT,
                    Raw TEXT NOT NULL,
                    URL TEXT NOT NULL,
                    Latency INTEGER,
                    Upload INT64,
                    Download INT64,
                    CreatedAt INT64 NOT NULL,
                    ModifiedAt INT64 NOT NULL
                    )";

pub const GROUP_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS groups(
                    ID INTEGER PRIMARY KEY AUTOINCREMENT,
                    Name TEXT UNIQUE NOT NULL,
                    IsSubscription BOOLEAN NOT NULL,
                    Type INTEGER NOT NULL,
                    Url TEXT,
                    CycleTime INTEGER,
                    CreatedAt INT64 NOT NULL,
                    ModifiedAt INT64 NOT NULL
                    )";

pub const RUNTIME_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS runtime(
                    ID INTEGER PRIMARY KEY AUTOINCREMENT,
                    Name TEXT UNIQUE NOT NULL,
                    Type INTEGER NOT NULL,
                    Value TEXT
                    )";
