### lock-across-await

Holding a standard synchronous lock across an `.await` point was detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Holding a `std::sync::MutexGuard` or `std::sync::RwLockReadGuard` across `.await` prevents the holding async task from yielding its worker thread safely. This can lead to thread pool starvation, high latency, and deadlocks across concurrent async tasks.

**Rust Idioms & Coaching:**
1. **Narrow the Guard Scope**: Extract the data into an owned or cloned local value inside a block, dropping the guard before awaiting:
   ```rust
   let payload = {
       let guard = state.lock().unwrap();
       guard.cached_item.clone()
   };
   // The mutex guard is dropped here before suspension
   send_request(&payload).await?;
   ```
2. **Use Async Mutex When Necessary**: If the guard must be held across suspension points, use `tokio::sync::Mutex` instead of `std::sync::Mutex`.
3. **Actor Pattern**: If multiple async tasks share mutable state, consider routing operations through an `mpsc` channel to an isolated worker task.
