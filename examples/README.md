These examples show how you can use this program. to follow these examples, you should:

```
cd examples/
# needed for initial compilation as the
# run.rs program will attempt to include_bytes!("include.txt")
echo "" > include.txt

cargo build --example generate

# this will fill the include.txt file with a bunch of 'q's
../target/debug/examples/generate ./include.txt

# build the run executable which will include the include file:
cargo build --example run

# build the modify executable to modify the run binary
cargo build --example modify

# copy the run executable, and then modify it's data section
# (it looks for that section of 'q's)
cp ../target/debug/examples/run ./
../target/debug/examples/modify ./run "this string will be inserted into the run executable"

# run the run file, and you should see it output the data that was entered when modifying
# in this case, the string you entered when running the modify command
./run
```