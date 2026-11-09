// Let's say we're hosting company with server port always ending with '0'
const serverPort = Ports.server;

if (serverPort > 0 && !String(serverPort).endsWith("0")) {
    return "NotMeowHost";
}

>>>

[NotMeowHost]
header = "This server is not hosted on MeowHost"
solutions = []
private = true # 🤫 this detection is not shown to users but can be accessed through API