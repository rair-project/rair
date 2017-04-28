#! /bin/bash
. assert.sh
echo "Testing:" `../target/release/rafind -v`
#XXX
echo "TODO -v -X -b are untested"
echo "TODO -m is neither implemented nor tested"

#testing without arguments
assert "../target/release/rafind" ""
assert "../target/release/rafind 2>&1" "Usage: ../target/release/rafind [-mXnzZhv] [-a align] [-b sz] [-f/t from/to] [-[m|s|S|e] str] [-x hex] file .."

#testing -h
assert "../target/release/rafind -h" ""
assert "../target/release/rafind -h 2>&1" "Usage: ../target/release/rafind [-mXnzZhv] [-a align] [-b sz] [-f/t from/to] [-[m|s|S|e] str] [-x hex] file ..

Options:
    -a align            only accept aligned hits
    -b size             set block size
    -e regex            search for regular expression string matches
    -f from             start searching from address 'from'
    -h                  show this help
    -m                  magic search, file-type carver
    -M str              set a binary mask to be applied on keywords
    -r                  print as radare2 flag commands
    -s str              search for a specific string (can be used multiple
                        times)
    -t to               stop search at address 'to'
    -v                  print version and exit
    -x hex              search for hexpair string (can be used multiple times)
    -X                  show hexdump of search results
    -z                  search for zero-terminated strings
    -Z                  show string found on each search hit"
#testing -s
assert "../target/release/rafind assert.sh -s aria" "Processing File assert.sh:
Match at 0x000003d9
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3
Match at 0x00000732"

#testing -Z
assert "../target/release/rafind assert.sh -s aria -Z" "Processing File assert.sh:
Match at 0x000003d9: aria
Match at 0x00000464: aria
Match at 0x000005c7: aria
Match at 0x000005f3: aria
Match at 0x00000732: aria"

#testing -r
assert "../target/release/rafind assert.sh -s aria -r" "Processing File assert.sh:
f hit_61726961_3d9 0x3d9
f hit_61726961_464 0x464
f hit_61726961_5c7 0x5c7
f hit_61726961_5f3 0x5f3
f hit_61726961_732 0x732"

#testing -x
assert "../target/release/rafind assert.sh -x 61726961"  "Processing File assert.sh:
Match at 0x000003d9
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3
Match at 0x00000732"

#testing -t
assert "../target/release/rafind assert.sh -s aria -t 0x735" "Processing File assert.sh:
Match at 0x000003d9
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3"
assert "../target/release/rafind assert.sh -s aria -t 0x736" "Processing File assert.sh:
Match at 0x000003d9
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3
Match at 0x00000732"

#testing -f
assert "../target/release/rafind assert.sh -s aria -f 0x3d8" "Processing File assert.sh:
Match at 0x000003d9
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3
Match at 0x00000732"
assert "../target/release/rafind assert.sh -s aria -f 0x3d9" "Processing File assert.sh:
Match at 0x000003d9
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3
Match at 0x00000732"
assert "../target/release/rafind assert.sh -s aria -f 0x3da" "Processing File assert.sh:
Match at 0x00000464
Match at 0x000005c7
Match at 0x000005f3
Match at 0x00000732"

#testing -a
assert "../target/release/rafind assert.sh -s aria -a 0x4" "Processing File assert.sh:
Match at 0x00000464"

#testing -e
assert "../target/release/rafind assert.sh -e \[\[:digit:\]\]\[\[:digit:\]\]\[\[:digit:\]\]\*" "Processing File assert.sh:
Match at 0x0000004a
Match at 0x00000050
Match at 0x00000056
Match at 0x0000005c"

# testing -M
assert "../target/release/rafind assert.sh -x 41414141 -M F0 -Z" "Processing File assert.sh:
Match at 0x00000225: AAAA
Match at 0x0000022a: AAAA"
assert_end examples
