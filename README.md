# constellation-rust [WIP]
Implementation of Constellation in Rust

JAVA implementation can be found here: [Constellation](https://github.com/NLeSC/Constellation)

## File Structure
All files located in `src/` are part of the API.

For useful files such as pre-made activities, see the directory `src/util/`.

See `installation-guide-das5.org` for a guide on how to install everything necessary and executing an example application.

**Links:**

## Dependencies
* GCC 6.4.0
* Open MPI 1.10.3
* CMake 3.14.0
* Clang 7.0.1

## Examples
See the directory `examples/` for various example implementations. To execute an example implementation run e.g. `cargo run --example vector_add 1 4 125000`, this will run on 1 node using 4 threads. To run distributed using MPI after compilation, in order to e.g. specify mpi flags, run: `mpirun MPI_ARGS path_to_executable ARGS"`.

## Run on DAS-5 with slurm

Create a slurm script similar to this one:
```bash
$ cat $(which mpi_run)
#!/bin/bash
#SBATCH --time=00:15:00
#SBATCH -N 1
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
Compile with `cargo build` and execute with slurm with `sbatch mpi_run /path/to/target/vector_add "<nmr_threads> <vector_lengt>"`. 

**Notes**

* Note that the arguments for vector_add are between quotation marks. 
* The binary will be located in `target/debug/` when building in development (`cargo build --example vector_add`).
* Constellation is not yet implemented for multiple nodes
---

## TODO
- Improve thread_handler to minimize locking of thread data structures
- Steal strategies
- Steal pools
- Add functionality to specify max size of work_queues on each executor thread (currently each thread has unlimited queues and the thread handler tries to load balance it)
- Enhance ConstellationError functionality and insert appropriate error messages where returned
- Simplify API as much as possible, while maintaining expressivness
- Publish on crates, add links to documentation and crate on this github page

##### Possible improvements to increase speedup of multithreading 
- Some elements can perhaps be removed from structs which are copied/passed around a lot with every activity/event (such as identifiers).
- Minimizing methods/variables that are inside the shared structures wrapped in mutexes, to avoid dead-waiting.
- Figuring out a better way for the multi-thread handler to distribute work. Currently, it can be a bottle-neck if a huge amount of activities are submitted in a short time. It currently locks the thread work queues to insert any newly added activity, also it locks them to check the length.