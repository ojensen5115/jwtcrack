# jwtcrack

This is jwtcrack, which serves three purposes, in order of importance:

- learn / practice Rust
- learn / practice threaded programming
- crack [JSON Web Tokens](https://tools.ietf.org/html/rfc7519) damn quickly

Comments on code, style, approach, architecture, etc. etc. very welcome!

jwtcrack takes the JWT as its only argument, and expects a dictionary from stdin.  This allows you to read a dictionary from a file like so:
```
$ # Read dictionary from file
$ jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Fw4maeqOtL8pPwiI2_VzYBo4JQ91P1Ow3X3hNqx2wPg" < words/rockyou.txt

Key found:
20 73 61 6D 61 6E 74 68 61 31 (' samantha1')
```

Or, if you're feeling adventurous and have `hashcat` installed, you can use [hashcat rules](https://hashcat.net/wiki/doku.php?id=rule_based_attack) such as [Hob0Rules](https://github.com/praetorian-inc/Hob0Rules) like so:
```
$ # Use hashcat rules
$ hashcat -r words/hob064.rule words/rockyou.txt --stdout | jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM"

Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')
```

Currently it spawns 4 worker threads (and to tweak this you'll need to edit/recompile), but I plan on making this an option soon.

## Performance Notes:

### Macbook Pro (8 cores, 16G ram)
If you're piping data from hashcat and using a reasonable work factor, it doesn't make sense to use more than 4 threads, because you're limited by the speed of the output.
The examples below start 8 threads, but only 4 run at a time (the others block waiting for input).
Increasing work factor increases performance until about 500. 500-1000 is a very small improvement. 10,000 also works well, roughly the same as 1000.

```
$ time hashcat -r words/hob064.rule words/rockyou.txt --stdout | jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM"

Starting 8 threads with work factor 500
Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')

real	1m27.630s
user	2m39.695s
sys	1m56.906s


$ time hashcat -r words/hob064.rule words/rockyou.txt --stdout | jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM"

Starting 8 threads with work factor 1000
Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')

real	1m26.801s
user	2m37.132s
sys	1m57.190s


$ time hashcat -r words/hob064.rule words/rockyou.txt --stdout | jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM"

Starting 8 threads with work factor 10000
Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')

real	1m26.626s
user	2m39.126s
sys	1m56.518s
```

If you're piping a file directly, all 8 cores get used and it runs a lot faster. Again, a work factor of 1000 seems like a decent choice.

```
$ time jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM" < words/rockyou_hobo.txt

Starting 8 threads with work factor 500
Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')

real	0m37.294s
user	4m43.001s
sys	0m1.838s


$ time jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM" < words/rockyou_hobo.txt

Starting 8 threads with work factor 1000
Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')

real	0m36.430s
user	4m34.713s
sys	0m1.282s


$ time jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM" < words/rockyou_hobo.txt

Starting 8 threads with work factor 10000
Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')

real	0m36.976s
user	4m37.329s
sys	0m1.195s
```
