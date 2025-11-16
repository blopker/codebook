Downloader Cache Specification

1. Overview

This document specifies the behavior of a Downloader struct. The struct is responsible for managing file downloads and maintaining a persistent, on-disk cache manifest to avoid re-downloading.

The primary requirements are:

Lazy Initialization: The on-disk manifest must not be read at program startup. It must be loaded on the first cache access (read or write) to minimize startup time.

Thread Safety: All access to the cache manifest must be thread-safe.

Write-Through Persistence: Any change to the in-memory cache (adding a new entry, removing a stale one) must be immediately and synchronously persisted to the manifest file on disk.

Validate-on-Read: An entry in the manifest is only considered "valid" if the corresponding file exists on the filesystem. Cache reads must validate this existence. Stale entries (where the file is missing) must be purged from the cache and the on-disk manifest.

2. State

The Downloader struct shall manage its state via the following components.

2.1. cache: OnceLock<RwLock<CacheManifest>>

This is the core of the lazy, thread-safe mechanism.

OnceLock: Guarantees that the initialization closure (disk read) is run exactly once, on the first thread that attempts to access cache. All other threads will block until this initialization is complete.

RwLock: Provides interior mutability, allowing multiple concurrent reads (check_cache) and exclusive, blocking writes (add_to_cache, stale entry removal).

CacheManifest: A type alias for HashMap<String, String>, mapping a resource URL to its local file path.

2.2. cache_path: PathBuf

The fully qualified path to the on-disk JSON file representing the cache manifest.

3. Core Behaviors

3.1. Constructor: new(cache_path: PathBuf) -> Self

Behavior: The constructor must be non-blocking and infallible.

Action: It shall store the provided cache_path.

Action: It shall initialize cache with OnceLock::new().

Post-condition: No I/O is performed. The cache is uninitialized.

3.2. Internal: get_cache(&self) -> &RwLock<CacheManifest>

This internal method is the gatekeeper for all cache access.

Behavior: It shall call self.cache.get_or_init(...).

Initialization Closure: The closure provided to get_or_init shall execute exactly once. This closure must:

Attempt to open and read the file at self.cache_path.

Attempt to deserialize the file content as a CacheManifest (e.g., from JSON).

If any step fails (e.g., file not found, permission denied, corrupt JSON), it must log the error and return a new, empty CacheManifest.

If successful, it shall return the deserialized CacheManifest.

The final CacheManifest (loaded or new) shall be wrapped in an RwLock and returned.

Return: Returns a reference to the now-initialized RwLock. Subsequent calls will not execute the closure and will return the reference immediately.

3.3. Internal: persist_cache(&self, manifest: &CacheManifest) -> io::Result<()>

This internal helper enforces the write-through requirement.

Behavior: This operation must be synchronous and blocking.

Action: It shall acquire an exclusive write lock on the file at self.cache_path.

Action: It shall serialize the provided manifest (e.g., to pretty JSON) and write it to the file, truncating any existing content.

Error Handling: I/O or serialization errors shall be propagated to the caller.

4. API Method Specifications

4.1. add_to_cache(&self, url: String, file_path: String)

This method adds a newly downloaded file to the cache. It enforces the write-through requirement.

Pre-condition: The file at file_path has been successfully downloaded.

Action:

Acquire an exclusive write lock from self.get_cache().

Insert the (url, file_path) pair into the CacheManifest.

Call self.persist_cache() with the (now-modified) manifest.

If persist_cache returns an error, this error must be logged. The in-memory cache will be inconsistent with the on-disk cache until the next successful persist_cache call.

Release the write lock.

4.2. check_cache(&self, url: &str) -> Option<String>

This method checks for a valid cache entry. It enforces the validate-on-read requirement.

Action:

Acquire a read lock from self.get_cache().

Check if url exists in the CacheManifest.

Case A: Not Found (Cache Miss)

Release the read lock.

Return None.

Case B: Found (Cache Hit)

Clone the file_path string associated with the url.

Release the read lock.

Validation (No Lock Held)

Check for the existence of the file at file_path on the filesystem (e.g., Path::new(&file_path).exists()).

Case B.1: File Exists (Valid Hit)

Return Some(file_path).

Case B.2: File Missing (Stale Hit)

The entry is stale and must be purged.

Acquire an exclusive write lock from self.get_cache().

Double-Check: After acquiring the lock, check if the entry for url is still the same stale file_path. This prevents a race condition where another thread has already modified the entry.

If (and only if) the entry is the same stale path:

Remove the url entry from the CacheManifest.

Call self.persist_cache() with the modified manifest to persist the removal.

Log any persistence errors.

Release the write lock.

Return None.
