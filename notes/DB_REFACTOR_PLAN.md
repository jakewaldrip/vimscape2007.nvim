# Database Refactor Plan

This document outlines potential improvements to the database handling in `vimscape_backend/src/api.rs` and `vimscape_backend/src/db.rs`.

---

## 1. Connection Pooling / Persistence

### Current State
Each API call (`process_batch`, `get_user_data`, `setup_tables`, `get_skill_details`) opens a new SQLite connection.

### The Challenge
The Rust binary is a shared library loaded by Neovim. It's not a long-running process in the traditional sense — functions are invoked on-demand from Lua. However, the library *does* stay loaded in memory between calls.

### Solution: Lazy Static Connection

Use `once_cell::sync::Lazy` (already a dependency) to maintain a persistent connection:

```rust
use std::sync::Mutex;
use once_cell::sync::Lazy;
use rusqlite::Connection;

// Global connection wrapped in Mutex for thread safety
static DB_CONN: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

fn get_or_create_connection(db_path: &str) -> Result<std::sync::MutexGuard<'_, Option<Connection>>, String> {
    let mut guard = DB_CONN.lock().map_err(|e| e.to_string())?;
    
    if guard.is_none() {
        let conn = Connection::open(Path::new(db_path).join("teste.db"))
            .map_err(|e| e.to_string())?;
        *guard = Some(conn);
    }
    
    Ok(guard)
}
```

**Trade-offs:**
- **Pro:** Single connection reused across all calls
- **Pro:** Connection stays alive as long as Neovim is running
- **Con:** Mutex overhead (minimal for single-threaded Neovim)
- **Con:** Path changes require connection reset logic

**Alternative:** Accept the per-call connection cost. SQLite connections are cheap (~microseconds), and for a fun plugin this may be acceptable. The real bottleneck would be disk I/O, not connection overhead.

---

## 2. Batch Updates with Transactions

### Current State

```rust
// db.rs:25-31
pub fn write_exp_to_table(conn: &Connection, skills: HashMap<String, i32>) {
    for (key, exp) in skills {
        let _ = conn.execute(
            "update skills set exp = exp + ?1 where name = ?2",
            params![exp, key],
        );
    }
}
```

Each UPDATE is a separate transaction (SQLite auto-commits by default).

### Improved Version

```rust
use rusqlite::{params, Connection, Result};

pub fn write_exp_to_table(conn: &Connection, skills: HashMap<String, i32>) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    
    {
        let mut stmt = tx.prepare_cached("UPDATE skills SET exp = exp + ?1 WHERE name = ?2")?;
        for (key, exp) in skills {
            stmt.execute(params![exp, key])?;
        }
    }
    
    tx.commit()
}

pub fn write_levels_to_table(conn: &Connection, levels_diff: &HashMap<String, i32>) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    
    {
        let mut stmt = tx.prepare_cached("UPDATE skills SET level = ?1 WHERE name = ?2")?;
        for (key, level) in levels_diff {
            stmt.execute(params![level, key])?;
        }
    }
    
    tx.commit()
}
```

**Benefits:**
- Single transaction = single disk sync (much faster)
- `prepare_cached` reuses the prepared statement
- All-or-nothing semantics (atomicity)

**Note:** Using `unchecked_transaction()` because we're borrowing `conn` immutably. For a `&mut Connection`, use `transaction()`.

---

## 3. Error Handling Strategy

### Current State

Three different patterns exist:
1. `let _ = conn.execute(...)` — silently ignore errors
2. `.expect("message")` — panic on error
3. `let Ok(...) else { return ...; }` — early return with print

### Suggested Unified Approach

Since this is a fun local plugin, we don't need heavy error machinery. Suggested pattern:

```rust
/// Errors are logged but don't crash the plugin.
/// Failed operations return sensible defaults.

pub fn write_exp_to_table(conn: &Connection, skills: HashMap<String, i32>) -> bool {
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("[vimscape] Transaction start failed: {e}");
            return false;
        }
    };
    
    let mut stmt = match tx.prepare_cached("UPDATE skills SET exp = exp + ?1 WHERE name = ?2") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Prepare failed: {e}");
            return false;
        }
    };
    
    for (key, exp) in skills {
        if let Err(e) = stmt.execute(params![exp, key]) {
            eprintln!("[vimscape] Update failed for {key}: {e}");
            // Continue with other skills rather than aborting
        }
    }
    
    if let Err(e) = tx.commit() {
        eprintln!("[vimscape] Commit failed: {e}");
        return false;
    }
    
    true
}
```

**For read operations**, return empty/default on error:

```rust
pub fn get_skill_data(conn: &Connection) -> Vec<SkillData> {
    let mut statement = match conn.prepare("SELECT name, exp, level FROM skills") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Query prepare failed: {e}");
            return Vec::new();
        }
    };
    
    let skill_data_iter = match statement.query_map([], |row| {
        Ok(SkillData {
            skill_name: row.get(0)?,
            total_exp: row.get(1)?,
            level: row.get(2)?,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("[vimscape] Query failed: {e}");
            return Vec::new();
        }
    };
    
    skill_data_iter.filter_map(|r| r.ok()).collect()
}
```

**Key principles:**
- Never panic (`expect`/`unwrap`) — plugin stays alive
- Log errors to stderr (visible in `:messages` if redirected)
- Return sensible defaults (empty vec, false, etc.)
- Continue processing where possible (don't abort entire batch for one bad skill)

---

## 4. Transaction Boundaries for Atomicity

### Current State

`process_batch` in `api.rs` does:
1. Read skill data
2. Calculate new levels
3. Write levels to table
4. Write XP to table

If interrupted between steps 3 and 4, state is inconsistent.

### Improved Version

Wrap the entire write operation in a single transaction:

```rust
pub fn process_batch((input, db_path): (String, String)) -> bool {
    // ... lexing and skill accumulation unchanged ...
    
    let Ok(conn) = Connection::open(Path::new(&db_path).join("teste.db")) else {
        eprintln!("[vimscape] Failed to connect to database");
        return false;
    };
    
    let skill_data = match get_skill_data(&conn) {
        data if !data.is_empty() => data,
        _ => return false,
    };
    
    let updated_levels = get_updated_levels(&skill_data, &skills);
    let levels_diff = get_levels_diff(&skill_data, &updated_levels);
    
    // Single transaction for all writes
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("[vimscape] Transaction start failed: {e}");
            return false;
        }
    };
    
    // Write both levels and XP within the same transaction
    if !write_levels_to_table_tx(&tx, &levels_diff) {
        return false;
    }
    if !write_exp_to_table_tx(&tx, skills) {
        return false;
    }
    
    if let Err(e) = tx.commit() {
        eprintln!("[vimscape] Commit failed: {e}");
        return false;
    }
    
    // Notifications happen after successful commit
    notify_level_ups(&levels_diff);
    
    true
}
```

**New db.rs functions that take a Transaction:**

```rust
use rusqlite::Transaction;

pub fn write_exp_to_table_tx(tx: &Transaction, skills: HashMap<String, i32>) -> bool {
    let mut stmt = match tx.prepare_cached("UPDATE skills SET exp = exp + ?1 WHERE name = ?2") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Prepare failed: {e}");
            return false;
        }
    };
    
    for (key, exp) in skills {
        if let Err(e) = stmt.execute(params![exp, key]) {
            eprintln!("[vimscape] Update failed for {key}: {e}");
        }
    }
    
    true
}

pub fn write_levels_to_table_tx(tx: &Transaction, levels_diff: &HashMap<String, i32>) -> bool {
    let mut stmt = match tx.prepare_cached("UPDATE skills SET level = ?1 WHERE name = ?2") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Prepare failed: {e}");
            return false;
        }
    };
    
    for (key, level) in levels_diff {
        if let Err(e) = stmt.execute(params![level, key]) {
            eprintln!("[vimscape] Update failed for {key}: {e}");
        }
    }
    
    true
}
```

---

## 5. Summary of Changes

| Area | Current | Proposed |
|------|---------|----------|
| Connection | New per call | Keep per call (acceptable) OR lazy static |
| Transactions | Implicit per statement | Explicit transaction wrapping writes |
| Error handling | Mixed (ignore/panic/return) | Unified: log + return default |
| Write atomicity | XP and levels separate | Single transaction for both |
| Statement prep | Fresh each execution | `prepare_cached` for loops |

---

## 6. Implementation Order

1. **Phase 1: Error handling cleanup**
   - Remove all `let _ =` patterns
   - Replace all `.expect()` with graceful handling
   - Add `eprintln!` logging

2. **Phase 2: Transaction wrapping**
   - Add transaction to `process_batch`
   - Create `_tx` variants of write functions
   - Use `prepare_cached` in loops

3. **Phase 3: (Optional) Connection persistence**
   - Only if profiling shows connection overhead is significant
   - Add lazy static with Mutex
   - Handle path changes gracefully

---

## 7. Testing Considerations

- Test transaction rollback on simulated failure
- Test that partial batch failures don't corrupt state
- Test concurrent access (unlikely but possible with async plugins)
- Verify error messages appear in Neovim's `:messages`
