# jwtcrack

This is jwtcrack, which serves three purposes, in order of importance:

- learn / practice Rust
- learn / practice threaded programming
- crack [JSON Web Tokens](https://tools.ietf.org/html/rfc7519) damn quickly

Comments on code, style, approach, architecture, etc. etc. very welcome!

jwtcrack takes the JWT as its only argument, and expects a dictionary from stdin.  This allows you to read a dictionary from a file like so:
```
$ # Read dictionary from file
$ ./jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Fw4maeqOtL8pPwiI2_VzYBo4JQ91P1Ow3X3hNqx2wPg" < rockyou.txt 

Key found:
20 73 61 6D 61 6E 74 68 61 31 (' samantha1')
```

Or, if you're feeling adventurous and have `hashcat` installed, you can use [hashcat rules](https://hashcat.net/wiki/doku.php?id=rule_based_attack) such as [Hob0Rules](https://github.com/praetorian-inc/Hob0Rules) like so:
```
$ # Use hashcat rules
$ hashcat -r hob064.rule rockyou.txt --stdout | ./jwtcrack "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Uzr5ePfZFgmvhMFYJ9WAYISmGLj7JE7SWO43OrfmcZM" < aux/rockyou.txt

Key found:
62 75 64 40 70 33 24 74 30 37 21 ('bud@p3$t07!')
```

Currently it spawns 4 worker threads (and to tweak this you'll need to edit/recompile), but I plan on making this an option soon.
