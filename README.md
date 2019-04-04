# constellation-rust
Implementation of Constellation in Rust

## File Structure
All files located in `src/` are part of the API.

For useful files such as pre-made activities, see the directory `src/util/`.

## Examples
See the directory examples for various example implementations. To execute an example implementation run e.g. `cargo run --example vector_add 150000 1`. To run distributed using MPI after compilation: `mpirun MPI_ARGS path_to_executable ARGS"`.

### TODO
- Implement multithreaded base version.
- Go through all mod.rs/lib.r and make all files private, which are not included in the API. (currently pretty much everyting is public)
- Steal strategies
- Steal pools
- Add functionality to specify max size of work_queues on each executor thread
