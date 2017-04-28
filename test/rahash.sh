#! /bin/bash
. assert.sh
echo "Testing:" `../target/release/rahash -v`
#XXX
echo "TODO -I -E -v are untested"

#testing without arguments
assert "../target/release/rahash" ""
assert "../target/release/rahash 2>&1" "Usage: ../target/release/rahash [-rBhlkvje] [-b S] [-a A] [-c H] [-e A] [-s S] [-f O] [-t O] [file] ..."

#testing -h
assert "../target/release/rahash -h" ""
assert "../target/release/rahash -h 2>&1" "Usage: ../target/release/rahash [-rBhlkvje] [-b S] [-a A] [-c H] [-e A] [-s S] [-f O] [-t O] [file] ...

Options:
    -a algo             comma separated list of algorithms (default is
                        'sha256')
    -B                  show per-block hash
    -b bsize            specify the size of the block (instead of full file)
    -c hash             compare with this hash
    -d algo             decrypt. Use -S to set key and -I to set IV
    -E                  use as little endian
    -e algo             encrypt. Use -S to set key and -I to set IV
    -f from             start hashing at given address
    -h                  print this help message
    -I iv               use give initialization vector (IV) (hexa or s:string)
    -i num              repeat hash N iterations
    -j                  output in JSON format
    -k                  show hash using the openssh's randomkey algorithm
    -l                  list all available algorithms (see -a)
    -q                  run in quiet mode (-qq to show only the hash)
    -r                  output radare commands
    -S seed             use given seed for hasing or key for encryption /
                        decryption (hexa or s:string) use ^ to use seed as
                        prefix (key for -E) (- will slurp the key from stdin.
    -s string           hash this string instead of files
    -t to               stop hashing at given address
    -v                  show version information
    -x hexpair          hash this hexpair string instead of files"
# testing -l
assert "../target/release/rahash -l" "Available Hashes:
  md5
  sha1
  sha256
  sha384
  sha512
  crc16
  crc32
  md4
  xor
  xorpair
  parity
  entropy
  hamdist
  pcprint
  mod255
  xxhash
  adler32
  luhn
  crc8smbus
  crc15can
  crc16hdlc
  crc16usb
  crc16citt
  crc24
  crc32c
  crc32ecma267
Available Encoders/Decoders:
  base64
  base91
  punycode
Available Crypto Algos:
  rc2
  rc4
  rc6
  aes-ecb
  aes-cbc
  ror
  rol
  rot
  blowfish
  cps2
  des-ecb
  xor"

# testing -s
assert "../target/release/rahash -s hello_world" "0x00000000-0x0000000a sha256: 35072c1ae546350e0bfa7ab11d49dc6f129e72ccd57ec7eb671225bbd197c8f1"

# testing -t
assert "../target/release/rahash -s hello_worldAAAAAAA -t 0xa" "0x00000000-0x0000000a sha256: 35072c1ae546350e0bfa7ab11d49dc6f129e72ccd57ec7eb671225bbd197c8f1"

#testing -f
assert "../target/release/rahash -s AAAAhello_world -f 0x4" "0x00000004-0x0000000e sha256: 35072c1ae546350e0bfa7ab11d49dc6f129e72ccd57ec7eb671225bbd197c8f1"

#testing -f and -t all together
assert "../target/release/rahash -s AAAAAAhello_worldAAAAAAAA -f 0x6 -t 0x10" "0x00000006-0x00000010 sha256: 35072c1ae546350e0bfa7ab11d49dc6f129e72ccd57ec7eb671225bbd197c8f1"

#testing -x
assert "../target/release/rahash -x 68656c6c6f" "0x00000000-0x00000004 sha256: 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"

#testing -r
assert "../target/release/rahash -s hello -r" "e file.sha256=2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"

#testing -e
assert "../target/release/rahash -e base64 -s hello_world" "aGVsbG9fd29ybGQ="

#testing -d
assert "../target/release/rahash -d base64 -s aGVsbG9fd29ybGQ=" "hello_world"

# testing -i
assert "../target/release/rahash -s hello_world -i 2" "0x00000000-0x0000000a sha256: dd6543a79024af95cdaad4c2e66a6d683a01cbe3eddfdd1b68ec5737b8fb306d"

#testing -q
assert "../target/release/rahash -s hello -q" "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"

#testing -k
assert "../target/release/rahash -s hello -k" "sha256
+--[0x00000000]---+
|   +.+..         |
|E...o +          |
|+..o.+ o         |
|+.+.o *.         |
| = +.o. S        |
|..* .o = o       |
|.=..o o + .      |
|=.+o . o o       |
|*O+  .+..        |
+-----------------+"

# testing -j
assert "../target/release/rahash -s hello -j" "{\"name\":\"sha256\",\"hash\":\"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\"}"

#testing -a
assert "../target/release/rahash -s hello -a md5" "0x00000000-0x00000004 md5: 5d41402abc4b2a76b9719d911017c592"

#testing -c 
assert "../target/release/rahash -s hello -a md5 -c 5d41402abc4b2a76b9719d911017c592" "0x00000000-0x00000004 md5: 5d41402abc4b2a76b9719d911017c592
Computed hash matches the expected one."
assert "../target/release/rahash -s hello -a md5 -c 5d41402abc4b2a76b9719d911017c593" "0x00000000-0x00000004 md5: 5d41402abc4b2a76b9719d911017c592
Computed hash doesn't match the expected one."

#testing -S 
assert "../target/release/rahash -s hello -S 41" "0x00000000-0x00000004 sha256: 105d6b297d17fb03d5ce7489cc466b9e2340f7fe6f28f8fc724c90b0967ffd30"

#testing -S with s:
assert "../target/release/rahash -s hello -S s:A" "0x00000000-0x00000004 sha256: 105d6b297d17fb03d5ce7489cc466b9e2340f7fe6f28f8fc724c90b0967ffd30"

#testing -S with ^
assert "../target/release/rahash -s hello -S ^41" "0x00000000-0x00000004 sha256: e7d9c374ddb268ca1c0ce86141850a434f40f680b816ed70797ac58065c46a40"

#testing -S with ^s:
assert "../target/release/rahash -s hello -S ^s:A" "0x00000000-0x00000004 sha256: e7d9c374ddb268ca1c0ce86141850a434f40f680b816ed70797ac58065c46a40"

#testing hashing file
assert "../target/release/rahash ./assert.sh" "./assert.sh 0x00000000-0x000011c8 sha256: 84cd96140dd20b58a312e37eff0eaee0647b6f27593d30b9d9f8e164897f445c"

#testing -b with -B
#XXX should the 2 options become merged together?
assert "../target/release/rahash ./assert.sh -b 0x100 -B" "0x00000000-0x000000ff sha256: 8be976091be5b728b6194c3aa87cf9843cd23306b317aa95e1592f156cd09ef4
0x00000100-0x000001ff sha256: 58d3c82471d08df3ee6fec7dc1f5cf86293d7bc697b810cdceeef12f960ae873
0x00000200-0x000002ff sha256: 915889156f34d3a282e9b60e414aae1ee150f62b51d31b4bfb0c0f59f36c8f71
0x00000300-0x000003ff sha256: ae7ca5bb876080a138e55a954861f2f2fed059f6158f720a10521983d900209b
0x00000400-0x000004ff sha256: b8c0f8a11cfd479070c15936a591bc9291b4e8df09c3910d67bf1a097e573d55
0x00000500-0x000005ff sha256: 5e1bdf5d42496df6277c66b269734e61dbe89a6efbbcab55d5345628eeab6e89
0x00000600-0x000006ff sha256: 262ff07df414cc70c200ed05c77f547103e8e9496e76d3811463b15eeef81759
0x00000700-0x000007ff sha256: eb53669f82dd7bb5199ab1461ae73daaa3e611e8dbb17629ff5bc84b32f3ef0b
0x00000800-0x000008ff sha256: 750e85074565a42579b13804306edfe15e263d452f72cb056e76ce2e8855a08b
0x00000900-0x000009ff sha256: 98538d3b52a710a23d917168507c6541ec00b0010f9361097907d67f2ed86fc2
0x00000a00-0x00000aff sha256: 1733ea73a1213ec47e493fd7d48e350c80fc7ad71158c2eb31972e0d5c65b781
0x00000b00-0x00000bff sha256: e1546aafb7db0d9a4530b2bf75e7ca1e1084afb4639b06eb7b49de7f0fa88869
0x00000c00-0x00000cff sha256: de9a6e083e643d4d1422c9721f63d8b84b53404a754089a174dd4267a75e49ab
0x00000d00-0x00000dff sha256: 24d8199e3b3ab276d1cfb22d95a02fb501a8dfb284d61f62afe16487fd09b3c3
0x00000e00-0x00000eff sha256: edd057c77ed479a6a02441c98a269dc083e3ffc3d12d34f7744e6e8270670840
0x00000f00-0x00000fff sha256: da5b8683a109fc0e0cac561105721a641a9a6a09725ec7310dbf4983bf06d16f
0x00001000-0x000010ff sha256: 6f23c218e3dfb8956eb1f666fa76fdb5f1fe7a15fc58bca3a19263b59f7839e5
0x00001100-0x000011c8 sha256: 799574bd250cc02cad17bcaecd3f68d71363b313fceb814d2268821cbf065082"

assert_end examples
