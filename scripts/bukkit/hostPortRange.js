
/**
 * Let's say on our shared host the assigned (available) ports are
 * are <server port> to <server port + 9>
 */
const serverPort = Ports.server;

if (Server.isBukkitBased) {
    Ports.plugins.forEach((port, index) => {
        if (port < serverPort || port > serverPort + 9) {
            if (port === 8192) {
                return "DefaultVotifierPort";
            } else if (port === 8123) {
                return "DefaultDynmapPort";
            } else if (port === 19132) {
                return "DefaultGeyserPort";
            } else {
                return "PortOutsideOfAssignedRange";
            }
        }
    });
} else if (Server.isModded) {
    Ports.mods.forEach((port, index) => {
        if (port < serverPort || port > serverPort + 9) {
            return "PortOutsideOfAssignedRange";
        }
    })
}

>>>

[DefaultVotifierPort]
header = "Using Votifier on default port which is not assigned to the server"
solutions = ["Set the Votifier port to one of available ports that can be found in the `Information` section"]

[DefaultDynmapPort]
header = "Using Dynmap on default port which is not assigned to the server"
solutions = ["Set the Dynmap port to one of available ports that can be found in the `Information` section"]

[DefaultGeyserPort]
header = "Using Geyser on default port which is not assigned to the server"
solutions = ["Set the Geyser port to one of available ports that can be found in the `Information` section"]

[PortOutsideOfAssignedRange]
header = "Using port which is not assigned to the server thus not available"
solutions = ["Set the port to one of available ports that can be found in the `Information` section"]