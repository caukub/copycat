if (Log.containsLine('Preparing level ""')) {
    return "EmptyWorldName";
}

>>>

[EmptyWoldName]
header = "World does not have a name"
solutions = ["In the `server.properties` file, set the value of `level-name`"]