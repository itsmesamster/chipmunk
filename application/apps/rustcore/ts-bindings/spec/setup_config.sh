export JASMIN_TEST_CONFIGURATION="./spec/benchmarks.json"
export SH_HOME_DIR="/home/ubuntu"
if [ "$#" -gt 0 ]; then
    export PERFORMANCE_RESULTS="chipmunk_performance_results/Benchmark_$1.json"
else
    echo "No arguments provided."
fi