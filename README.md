# constellation-rust
Implementation of Constellation in Rust

## File Structure
All files located in `src/` are part of the API.

For useful files such as pre-made activities, see the directory `src/util/`.

**Links:**

## Dependencies
* GCC 6.4.0
* Open MPI 1.10.3
* CMake 3.12.0
* LLVM 7.0.1 (for Clang)

## Examples
See the directory `examples/` for various example implementations. To execute an example implementation run e.g. `cargo run --example vector_add 1 4 125000`, this will run on 1 node using 4 threads. To run distributed using MPI after compilation, in order to e.g. specify mpi flags, run: `mpirun MPI_ARGS path_to_executable ARGS"`.

## Run on DAS-5 with slurm

Create a slurm script similar to this one:
```bash
$ cat $(which mpi_run)
#!/bin/bash
#SBATCH --time=00:15:00
#SBATCH -N 2
# The number of threads per node should ALWAYS be 1.
# Constellation will deal with multithreading depending
# on how many threads are provided as argument.
#SBATCH --ntasks-per-node=1

. /etc/bashrc
. /etc/profile.d/modules.sh
module load openmpi/gcc/64

APP=$1
ARGS="$2"
OMPI_OPTS="--mca btl ^usnic"

$MPI_RUN $OMPI_OPTS $APP $AR
```
Compile with `cargo build` and execute with slurm with `sbatch mpi_run /path/to/target/vector_add "<nmr_nodes> <nmr_threads> <vector_lengt>"`. 

* Note that the arguments for vector_add are between quotation marks. 
* The binary will be located in `target/debug/` when building in development (`cargo build --example vector_add`).

---

## TODO
- Implement multithreaded base version.
- Go through all mod.rs/lib.r and make all files private, which are not included in the API. (currently pretty much everyting is public)
- Steal strategies
- Steal pools
- Add functionality to specify max size of work_queues on each executor thread
- Enhance ConstellationError functionality and insert appropriate error messages where returned
- Simplify API as much as possible, while maintaining expressivness
- Publish on crates, add links to documentation and crate on this github page
