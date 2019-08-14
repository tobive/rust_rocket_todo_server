# Simple Todo List Server with RUST and ROCKET

Homework from **RUST class** 8/8/2019

**Todo list**

- GET list?query="string"
- POST data=string
- DELETE by query="string"

**Bonus**

- use json
- save list in file

to start the server:

```
cargo run
```

example query:

_GET_

```
curl -X GET http://localhost:8000/api/list

curl -X GET http://localhost:8000/api/list?query=one
```

_POST_

```
curl -d '{"title":"ten", "content":"data ten"}' -H "Content-Type:application/json" -X POST http://localhost:8000/api/post
```

_DELETE_

```
curl -X DELETE http://localhost:8000/api/delete?query=ten
```
