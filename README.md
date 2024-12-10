# INFO-F404 Uniprocessor Scheduling Project

In this project, we consider synchronous task sets with constrained deadlines. The project includes:
* Implementation of three priority assignment algorithms:
  - **DM** (Deadline Monotonic)
  - **EDF** (Earliest Deadline First)
  - **RR** (Round Robin)
* A scheduler simulator to evaluate and simulate task execution under these scheduling policies.

## Running the program

### Prerequisites
To run this program, you need to have [Rust](https://www.rust-lang.org/tools/install) installed on your system.

### Usage

The program accepts two mandatory arguments:
1. The scheduling algorithm to use: `dm`, `edf`, `rr`
2. The path to the task set file: `<taskset file>`


```bash
cargo run dm|edf|rr <taskset file> 
```

### Example

```bash
cargo run dm path/to/taskset.txt
```
