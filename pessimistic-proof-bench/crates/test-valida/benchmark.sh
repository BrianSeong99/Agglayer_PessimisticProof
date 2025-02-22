#!/usr/bin/env bash

# Set up variables
BUILD_DIR="./target"
INPUT_DIR="./benchmark_inputs"

# Create necessary directories
mkdir -p $BUILD_DIR
mkdir -p $INPUT_DIR

# Build the program for valida
echo "Building program for Valida..."
/valida-toolchain/bin/clang -c -target delendum \
    -isystem /valida-toolchain/include \
    --sysroot=/valida-toolchain \
    src/bin/ppgen.rs -o $BUILD_DIR/ppgen.o

/valida-toolchain/bin/ld.lld \
    --script=/valida-toolchain/valida.ld \
    -o $BUILD_DIR/ppgen.out \
    $BUILD_DIR/ppgen.o \
    /valida-toolchain/DelendumEntryPoint.o \
    /valida-toolchain/lib/valida-unknown-baremetal-gnu/libc.a \
    /valida-toolchain/lib/valida-unknown-baremetal-gnu/libm.a

# Test cases - different numbers of exits to benchmark
SIZES=(10 25 50)

# Function to run benchmark and extract timing info
run_benchmark() {
    local size=$1
    local mode=$2
    local threads=$3
    local input_file="$INPUT_DIR/benchmark_input_$size.json"
    local results_dir="$INPUT_DIR/results"
    mkdir -p "$results_dir"
    
    echo "Running ${mode} benchmark (size: $size, threads: $threads)"
    
    # Run 10 iterations
    for i in {1..10}; do
        echo "  Iteration $i"
        RAYON_NUM_THREADS=$threads /usr/bin/time -v \
            valida run $BUILD_DIR/ppgen.out $input_file 2>&1 | \
            tee -a "$results_dir/benchmark_results_${size}_${mode}.txt"
        
        # Add small delay between runs
        sleep 1
    done
}

# First prepare all benchmark inputs
echo "Preparing benchmark inputs..."
for n in "${SIZES[@]}"; do
    echo "Generating input for $n exits..."
    # Create JSON input file directly
    cat > "$INPUT_DIR/benchmark_input_$n.json" << EOF
{
    "n_exits": $n,
    "n_imported_exits": $n
}
EOF
done

# Run benchmarks for each size
for n in "${SIZES[@]}"; do
    echo "Benchmarking with $n exits"
    
    # Serial benchmarks
    run_benchmark $n "serial" 1
    
    # Parallel benchmarks
    run_benchmark $n "parallel" 32
done

# Extract and format results
echo "Processing results..."
RESULTS_DIR="$INPUT_DIR/results"
echo "size,mode,user_time,wall_time" > "$RESULTS_DIR/benchmark_results.csv"

for n in "${SIZES[@]}"; do
    for mode in "serial" "parallel"; do
        result_file="$RESULTS_DIR/benchmark_results_${n}_${mode}.txt"
        
        # Extract User time and Wall clock time from GNU time output
        while IFS= read -r line; do
            if [[ $line =~ "User time (seconds):" ]]; then
                user_time=$(echo $line | awk '{print $4}')
            elif [[ $line =~ "Elapsed (wall clock) time (h:mm:ss or m:ss):" ]]; then
                # Convert time format to seconds
                wall_time=$(echo $line | awk '{print $8}' | awk -F: '{print ($1 * 60) + $2}')
                echo "$n,$mode,$user_time,$wall_time" >> "$RESULTS_DIR/benchmark_results.csv"
            fi
        done < "$result_file"
    done
done

echo "Benchmark completed. Results saved in $RESULTS_DIR/benchmark_results.csv"