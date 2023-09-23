// Let's say we're hosting company with server port always ending with '0'
let server_port = ports.server;

if server_port > 0 && !server_port.to_string().ends_with("0") {
    return "NotMeowHost";
}

///

[NotMeowHost]
header = "This server is not hosted on MeowHost"
solutions = []
private = false # 🤫this detection is not shown to users but can be accessed through API