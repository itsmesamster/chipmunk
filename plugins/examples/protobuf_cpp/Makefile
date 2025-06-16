WASI_SDK_PATH ?= /Users/sameer.g.srivastava/wasi-sdk-25.0-arm64-macos
CXX = $(WASI_SDK_PATH)/bin/clang++
CC = $(WASI_SDK_PATH)/bin/clang
AR = $(WASI_SDK_PATH)/bin/llvm-ar
WASM_TOOLS ?= /Users/sameer.g.srivastava/.cargo/bin/wasm-tools

# --- PRIMARY CXXFLAGS DEFINITION --
CXXFLAGS = --target=wasm32-wasi \
--sysroot=$(WASI_SDK_PATH)/share/wasi-sysroot \
-nostdlib -fPIC -Wall -O2 -std=c++17 \
-I.

# --- PRIMARY CFLAGS DEFINITION ---
CFLAGS = --target=wasm32-wasi \
--sysroot=$(WASI_SDK_PATH)/share/wasi-sysroot \
-nostdlib -fPIC -Wall -O2 \
-I.

# --- PRIMARY LDFLAGS DEFINITION ---
LDFLAGS = -lc -lc++ -lc++abi \
--target=wasm32-wasi \
--sysroot=$(WASI_SDK_PATH)/share/wasi-sysroot \
-L$(WASI_SDK_PATH)/share/wasi-sysroot/lib/wasm32-wasi/

# Nanopb settings
NANOPB_DIR ?= /opt/homebrew/Cellar/nanopb/0.4.9.1_2

CFLAGS += -I$(NANOPB_DIR)/include/nanopb
CXXFLAGS += -I$(NANOPB_DIR)/include -I./test

PLUGIN = protobuf
PLUGIN_NAME = protobuf_parser
SRC_CXX = protobuf_parser.cpp

PROTO_FILE = test/test.proto

WIT_BINDGEN_GENERATED_C_SRC = parse.c
WIT_BINDGEN_GENERATED_HEADER = parse.h
WIT_BINDGEN_GENERATED_OBJ_TYPE = parse_component_type.o

# The OBJ list should only contain object files (.o)
OBJ = $(WIT_BINDGEN_GENERATED_C_SRC:.c=.o) $(SRC_CXX:.cpp=.o) $(WIT_BINDGEN_GENERATED_OBJ_TYPE) test/test.pb.o test/pb_encode.o test/pb_decode.o test/pb_common.o

WIT_DIR = /Users/sameer.g.srivastava/Desktop/itsmesamster/chipmunk/plugins/plugins_api/wit/v0.1.0/
WIT_FILE = $(WIT_DIR)deps/parser/world.wit

# --- IMPORTANT CHANGE HERE ---
# The core WASM module is still built with the original name,
# but the final component will be renamed to plugin.wasm
CORE_WASM_MODULE = $(PLUGIN_NAME).wasm
FINAL_WASM_COMPONENT = plugin.wasm

WASI_ADAPTER_FILE = wasi_snapshot_preview1.reactor.wasm
WASI_ADAPTER_URL = https://github.com/bytecodealliance/wasmtime/releases/download/dev/wasi_snapshot_preview1.reactor.wasm

.PHONY: all clean download-adapter

all: $(FINAL_WASM_COMPONENT)

# Generate both .cc and .h in one go
test/test.pb.c test/test.pb.h: $(PROTO_FILE)
	@echo "Generating Nanopb C files from $< ..."
	protoc --plugin=protoc-gen-nanopb=$(NANOPB_DIR)/bin/protoc-gen-nanopb \
	       --nanopb_out=. \
	       -I. $<

# Step 1: Link the core WASM module
$(CORE_WASM_MODULE): $(OBJ)
	$(CXX) $(LDFLAGS) -o $@ $^

# Step 2: Convert the core WASM module into a WebAssembly component
# And rename it to plugin.wasm for chipmunk
$(FINAL_WASM_COMPONENT): $(CORE_WASM_MODULE) $(WASI_ADAPTER_FILE)
	$(WASM_TOOLS) component new $< --adapt=$(WASI_ADAPTER_FILE) -o $@

# Rule to download the WASI adapter if it doesn't exist
$(WASI_ADAPTER_FILE):
	@echo "Downloading WASI adapter: $(WASI_ADAPTER_FILE) from $(WASI_ADAPTER_URL)..."
	@curl -L -o $@ $(WASI_ADAPTER_URL)
	@echo "Download complete."

protobuf_parser.o: protobuf_parser.cpp test/test.pb.h
	$(CXX) $(CXXFLAGS) -c $< -o $@

parse.o: parse.c
	$(CC) $(CFLAGS) -c $< -o $@

test/test.pb.o: test/test.pb.c test/test.pb.h
	$(CC) $(CFLAGS) -c $< -o $@

test/pb_encode.o: pb_encode.c
	$(CC) $(CFLAGS) -c $< -o $@

test/pb_decode.o: pb_decode.c
	$(CC) $(CFLAGS) -c $< -o $@

test/pb_common.o: pb_common.c
	$(CC) $(CFLAGS) -c $< -o $@

# Rule to generate C and header files from WIT
$(WIT_BINDGEN_GENERATED_C_SRC) $(WIT_BINDGEN_GENERATED_HEADER) $(WIT_BINDGEN_GENERATED_OBJ_TYPE): $(WIT_FILE)
	wit-bindgen c $(WIT_DIR) -w chipmunk:parser/parse --out-dir .

clean:
	rm -f $(OBJ) $(CORE_WASM_MODULE) $(FINAL_WASM_COMPONENT) $(WASI_ADAPTER_FILE)
	rm -f $(WIT_BINDGEN_GENERATED_C_SRC) $(WIT_BINDGEN_GENERATED_HEADER) $(WIT_BINDGEN_GENERATED_OBJ_TYPE)
	rm -f test/*.o test/*.c test/*.h