# INFO-F404 Multiprocessor Scheduling Project

In this second project, we consider the case of asynchronous tasks with arbitrary deadlines in the scope of multiprocessor scheduling with *m* identical cores. The objective of this project is to build a multithreaded (or multi-processed) tool that checks the schedulability of task sets according to different real-time multiprocessor scheduling algorithms.

## Running the program

### Prerequisites
To run this program, you need to have [Rust](https://www.rust-lang.org/tools/install) installed on your system.

### Usage

The program accepts the following arguments
* the task set file to consider
* the number of cores *m*
* the version of **EDF** to use (partitioned, global, or EDF(𝑘))
* the number of workers *w* to run the simulation (set the number of cores of the computer by default)
* in the case of partitioned scheduling
  - the heuristic to use (first fit, next fit, …)
  - the ordering of tasks (increasing or decreasing utilization)

```bash
cargo run <task_file> <m> -v global|partitioned|<k> [-w <w>] [-h ff|nf|bf|wf] [-s iu|du]
```

### Examples

To run the program with **global EDF** on a task set file with 4 cores:

```bash
cargo run tasks.csv 4 -v global
```

For partitioned scheduling with **Worst Fit** and **Decreasing Utilization**:

```bash
cargo run tasks.csv 4 -v partitioned -h wf -s du
```

For **EDF(k)** with $k=2$:

```bash
cargo run tasks.csv 4 -v 2
```

## Program Outputs

The program exits with specific codes to represent schedulability results:

| Exit Code | Description                                    |
|-----------|------------------------------------------------|
| `0`       | Schedulable, simulation required.              |
| `1`       | Schedulable, sufficient condition met.         |
| `2`       | Not schedulable, simulation required.          |
| `3`       | Not schedulable, necessary condition failed.   |
| `4`       | Schedulability cannot be determined.           |