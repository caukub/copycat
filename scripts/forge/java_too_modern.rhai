if server.version matchserver "<= 1.16.5, > 1.8.9" && server.java_version matchver "> 1.8" {
    return "JavaTooModern";
}

///

[JavaTooModern]
header = "Java version is too modern for your version of Forge. Forge 1.16.5 and older requires Java 8"
solutions = ["Set Java version to 8"]