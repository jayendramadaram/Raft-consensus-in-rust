# Guide to run My Raft Consensus - Node runner

> Simulate new local Nodes

- run `cargo run [port]` to run a node on `http://localhost:800{port}/`
- Note : Only ports `0 , 1 , 2 , 3 , 4` work

Wait until count falls down and one of node Gets elected as leader

Now we can `execute` commands on leader by sending payload to `/execute`

> Route /execute

```json
// Example
{
 "command": "sub",
 "args" : [3 , 2 , 4]
}

command can be on of these : "set" , "add" , "sub , "mul" , "div"

args --> [index where it operation is done , Var1 , Var2 ]
```
